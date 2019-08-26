use rand::Rng;
use std::time::Instant;

use super::time_and_tide::{compute_avb_latency, schedule_online};
use super::{Flow, FlowTable, RoutingAlgo, StreamAwareGraph, GCL};
use crate::network_struct::Graph;
use crate::util::YensAlgo;
use crate::T_LIMIT;

type FT = FlowTable<usize>;

const K: usize = 20;
const ALPHA_PORTION: f64 = 0.5;
const C1_EXCEED: f64 = 100.0;
const W1: f64 = 1.0;
const W2: f64 = 1.0;
const W3: f64 = 1.0;

fn gen_n_distinct_outof_k(n: usize, k: usize) -> Vec<usize> {
    let mut vec = Vec::with_capacity(n);
    for i in 0..k {
        vec.push((rand::thread_rng().gen::<usize>(), i));
    }
    vec.sort();
    vec.into_iter().map(|(_, i)| i).take(n).collect()
}

pub struct RO<'a> {
    g: StreamAwareGraph,
    flow_table: FT,
    yens_algo: YensAlgo<'a, usize, StreamAwareGraph>,
    gcl: GCL,
    avb_count: usize,
    tt_count: usize,
}

impl<'a> RO<'a> {
    pub fn new(g: &'a StreamAwareGraph, flow_table: Option<FT>, gcl: Option<GCL>) -> Self {
        let flow_table = flow_table.unwrap_or(FlowTable::new());
        let gcl = gcl.unwrap_or(GCL::new(1, g.get_edge_cnt()));
        RO {
            g: g.clone(),
            yens_algo: YensAlgo::new(g, K),
            avb_count: flow_table.get_count(true),
            tt_count: flow_table.get_count(false),
            gcl,
            flow_table,
        }
    }
    pub fn compute_avb_cost(&self, flow: &Flow, k: Option<usize>) -> f64 {
        let k = match k {
            Some(t) => t,
            None => *self.flow_table.get_info(*flow.id()),
        };
        let max_delay = *flow.max_delay();
        let route = self.get_kth_route(flow, k);
        let latency = compute_avb_latency(&self.g, flow, route, &self.flow_table, &self.gcl);
        let c1 = if latency > max_delay { C1_EXCEED } else { 0.0 };
        let c2 = latency as f64 / max_delay as f64;
        let c3 = 0.0; // TODO 計算 c3
        W1 * c1 + W2 * c2 + W3 * c3
    }
    pub fn compute_all_avb_cost(&self) -> f64 {
        let mut cost = 0.0;
        self.flow_table.foreach(true, |flow, _| {
            cost += self.compute_avb_cost(flow, None);
        });
        cost
    }
    /// 在所有 TT 都被排定的狀況下去執行 GRASP 優化
    fn grasp(&mut self, time: Instant) {
        let mut iter_times = 0;
        let mut min_cost = std::f64::MAX;
        let mut best_all_routing = FlowTable::<usize>::new();
        while time.elapsed().as_micros() < T_LIMIT {
            iter_times += 1;
            // PHASE 1
            // NOTE 先從圖中把舊的資料流全部忘掉
            self.g.forget_all_flows();
            self.flow_table.foreach_mut(true, |flow, route_k| {
                let candidate_cnt = self.get_candidate_count(flow);
                let alpha = (candidate_cnt as f64 * ALPHA_PORTION) as usize;
                let set = Some(gen_n_distinct_outof_k(alpha, candidate_cnt));
                *route_k = self.find_min_cost_route(flow, set);
                unsafe {
                    // NOTE 把資料流的路徑與ID記憶到圖中
                    self.save_flowid_on_edge(true, flow, *route_k);
                }
            });
            // PHASE 2
            let cost = self.compute_all_avb_cost();
            if cost < min_cost {
                best_all_routing = self.flow_table.clone();
                min_cost = cost;
                println!("found min_cost = {} at first glance!", cost);
            }
            println!("start iteration #{}", iter_times);
            min_cost = self.hill_climbing(&time, min_cost, &mut best_all_routing);
        }
        self.flow_table = best_all_routing;
    }
    fn find_min_cost_route(&self, flow: &Flow, set: Option<Vec<usize>>) -> usize {
        let (mut min_cost, mut best_k) = (std::f64::MAX, 0);
        let mut closure = |k: usize| {
            let cost = self.compute_avb_cost(flow, Some(k));
            if cost < min_cost {
                min_cost = cost;
                best_k = k;
            }
        };
        if let Some(vec) = set {
            for k in vec.into_iter() {
                closure(k);
            }
        } else {
            for k in 0..self.get_candidate_count(flow) {
                closure(k);
            }
        }
        best_k
    }
    fn hill_climbing(
        &mut self,
        time: &std::time::Instant,
        mut min_cost: f64,
        best_all_routing: &mut FT,
    ) -> f64 {
        let mut iter_times = 0;
        while time.elapsed().as_micros() < T_LIMIT {
            let target_id = rand::thread_rng().gen_range(0, self.avb_count + self.tt_count);
            let target_flow = self.flow_table.get_flow(target_id);
            if target_flow.is_tt() {
                // TODO 用更好的機制篩選 avb flow
                continue;
            }
            let old_route = *self.flow_table.get_info(target_id);
            // 從圖中忘記舊路徑
            unsafe {
                self.save_flowid_on_edge(false, target_flow, old_route);
            }
            let new_route = self.find_min_cost_route(target_flow, None);
            let cost = if old_route == new_route {
                std::f64::MAX
            } else {
                // 在圖中記憶新路徑
                let _s = self as *const Self as *mut Self;
                unsafe {
                    self.save_flowid_on_edge(true, target_flow, new_route);
                    (*_s).flow_table.update_info(target_id, new_route);
                }

                self.compute_all_avb_cost()
            };
            if cost < min_cost {
                *best_all_routing = self.flow_table.clone();
                self.flow_table.update_info(target_id, new_route);

                min_cost = cost;
                iter_times = 0;
                println!("found min_cost = {}", cost);
            } else {
                // 恢復上一動
                unsafe {
                    self.save_flowid_on_edge(false, target_flow, new_route);
                }
                unsafe {
                    self.save_flowid_on_edge(true, target_flow, old_route);
                }
                self.flow_table.update_info(target_id, old_route);

                iter_times += 1;
                if iter_times == self.avb_count {
                    break;
                }
            }
        }
        min_cost
    }
    fn get_kth_route(&self, flow: &Flow, k: usize) -> &Vec<usize> {
        self.yens_algo.get_kth_route(*flow.src(), *flow.dst(), k)
    }
    fn get_candidate_count(&self, flow: &Flow) -> usize {
        self.yens_algo.get_route_count(*flow.src(), *flow.dst())
    }
    unsafe fn save_flowid_on_edge(&self, remember: bool, flow: &Flow, k: usize) {
        let _g = &self.g as *const StreamAwareGraph as *mut StreamAwareGraph;
        let route = self.get_kth_route(flow, k);
        (*_g).save_flowid_on_edge(remember, *flow.id(), route);
    }
}
impl<'a> RoutingAlgo for RO<'a> {
    fn add_flows(&mut self, flows: Vec<Flow>) {
        self.flow_table.insert(flows.clone(), 0);
        let mut tt_changed = self.flow_table.clone_into_changed_table();
        for flow in flows.iter() {
            self.yens_algo.compute_routes(*flow.src(), *flow.dst());
            if flow.is_avb() {
                self.avb_count += 1;
            } else {
                self.tt_count += 1;
                tt_changed.update_info(*flow.id(), 0);
            }
        }
        let time = Instant::now();
        // TT schedule
        unsafe {
            let _self = self as *mut Self;
            schedule_online(
                &mut (*_self).flow_table,
                &tt_changed,
                &mut (*_self).gcl,
                |flow, &k| {
                    let r = self.get_kth_route(flow, k);
                    self.g.get_links_id_bandwidth(r)
                },
            )
            .unwrap();
        }

        self.grasp(time);
        self.g.forget_all_flows();
        self.flow_table.foreach(true, |flow, r| unsafe {
            self.save_flowid_on_edge(true, flow, *r);
        });
    }
    fn del_flows(&mut self, flows: Vec<Flow>) {
        unimplemented!();
    }
    fn get_retouted_flows(&self) -> &Vec<usize> {
        unimplemented!();
    }
    fn get_route(&self, id: usize) -> &Vec<usize> {
        let k = *self.flow_table.get_info(id);
        let flow = self.flow_table.get_flow(id);
        self.get_kth_route(flow, k)
    }
    fn show_results(&self) {
        println!("TT Flows:");
        self.flow_table.foreach(false, |flow, &route_k| {
            let route = self.get_kth_route(flow, route_k);
            println!("flow id = {}, route = {:?}", *flow.id(), route);
        });
        println!("AVB Flows:");
        self.flow_table.foreach(true, |flow, &route_k| {
            let route = self.get_kth_route(flow, route_k);
            let cost = self.compute_avb_cost(flow, Some(route_k));
            println!(
                "flow id = {}, route = {:?} cost = {}",
                *flow.id(),
                route,
                cost
            );
        });
        println!("total avb cost = {}", self.compute_all_avb_cost());
    }
}
