use rand::Rng;

use crate::util::YensAlgo;
use super::{StreamAwareGraph, FlowTable, Flow, RoutingAlgo, GCL};
use super::time_and_tide::compute_avb_latency;

const K: usize = 20;
const ALPHA_PORTION: f64 = 0.5;
const T_LIMIT: u128 = 1000 * 10; // micro_sec
const C1_EXCEED: f64 = 1000.0;
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
    flow_table: FlowTable<usize>,
    yens_algo: YensAlgo<'a, usize, StreamAwareGraph>,
    gcl: GCL,
    avb_count: usize,
    tt_count: usize,
}

impl <'a> RO<'a> {
    pub fn new(g: &'a StreamAwareGraph, hyper_p: usize, gcl: GCL) -> Self {
        RO {
            gcl,
            g: g.clone(),
            flow_table: FlowTable::new(),
            yens_algo: YensAlgo::new(g, K),
            avb_count: 0,
            tt_count: 0,
        }
    }
    pub fn compute_avb_cost(&self, flow: &Flow) -> f64 {
        let max_delay = *flow.max_delay();
        let route = self.get_kth_route(flow, *self.flow_table.get_info(*flow.id()));
        let latency = compute_avb_latency(
            &self.g,
            flow,
            route,
            &self.flow_table,
            &self.gcl
        ) as f64;
        let c1 = if latency > max_delay {
            C1_EXCEED
        } else {
            0.0
        };
        let c2 = latency / max_delay;
        let c3 = 0.0; // TODO 計算 c3
        W1*c1 + W2*c2 + W3*c3
    }
    pub fn compute_all_avb_cost(&self) -> f64 {
        let mut cost = 0.0;
        self.flow_table.foreach(true, |flow, _| {
            cost += self.compute_avb_cost(flow);
        });
        cost
    }
    /// 在所有 TT 都被排定的狀況下去執行 GRASP 優化
    fn grasp(&mut self) {
        let _g = &mut self.g as *mut StreamAwareGraph;
        let time = std::time::Instant::now();
        let mut iter_times = 0;
        let mut min_cost = std::f64::MAX;
        let mut best_all_routing = FlowTable::<usize>::new();
        while time.elapsed().as_micros() < T_LIMIT {
            iter_times += 1;
            // PHASE 1
            unsafe {
                // NOTE 從圖中把舊的資料流全部忘掉
                (*_g).forget_all_flows();
            }
            self.flow_table.foreach_mut(true, |flow, route| {
                let mut min_cost = std::f64::MAX;
                let mut best_r = 0;
                let k = self.get_candidate_count(flow);
                let alpha = (k as f64 * ALPHA_PORTION) as usize;
                for r in gen_n_distinct_outof_k(alpha, k).into_iter() {
                    *route = r;
                    let cost = self.compute_avb_cost(flow);
                    if cost < min_cost {
                        min_cost = cost;
                        best_r = r;
                    }
                }
                *route = best_r;
                let route = self.get_kth_route(flow, best_r);
                unsafe {
                    // NOTE 把資料流的路徑與ID記憶到圖中
                    (*_g).save_flowid_on_edge(true, *flow.id(), route);
                }
            });
            // PHASE 2
            let cost = self.compute_all_avb_cost();
            if cost < min_cost {
                best_all_routing = self.flow_table.clone();
                min_cost = cost;
                println!("found min_cost = {} at first glance!", cost);
            }
            min_cost = self.hill_climbing(&time, min_cost, &mut best_all_routing);
        }
        println!("# of iteration = {}", iter_times);
        self.flow_table = best_all_routing;
    }
    fn hill_climbing(&mut self, time: &std::time::Instant,
        mut min_cost: f64, best_all_routing: &mut FlowTable<usize>
    ) -> f64 {
        let mut iter_times = 0;
        let _g = &mut self.g as *mut StreamAwareGraph;
        while time.elapsed().as_micros() < T_LIMIT {
            let target_id = rand::thread_rng().gen_range(0, self.avb_count);
            let target_flow = self.flow_table.get_flow(target_id);
            let r = self.get_candidate_count(&target_flow);
            let new_route = rand::thread_rng().gen_range(0, r);
            let old_route = *self.flow_table.get_info(target_id);
            if old_route == new_route {
                continue;
            } else {
                // NOTE 從圖中忘記舊路徑，記憶新路徑
                let old_route = self.get_kth_route(target_flow, old_route);
                let new_route = self.get_kth_route(target_flow, new_route);
                unsafe {
                    (*_g).save_flowid_on_edge(false, target_id, old_route);
                    (*_g).save_flowid_on_edge(true, target_id, new_route);
                }
            }
            self.flow_table.update_info(target_id, new_route);
            let cost = self.compute_all_avb_cost();
            if cost < min_cost {
                *best_all_routing = self.flow_table.clone();
                min_cost = cost;
                iter_times = 0;
                println!("found min_cost = {}", cost);
            } else {
                iter_times += 1;
                //println!("Nothing found QQ");
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
}
impl <'a> RoutingAlgo for RO<'a> {
    fn compute_routes(&mut self, flows: Vec<Flow>) {
        for flow in flows.iter() {
            self.yens_algo.compute_routes(*flow.src(), *flow.dst());
            if let Flow::AVB { .. } = &flow {
                self.avb_count += 1;
            } else {
                self.tt_count += 1;
            }
        }
        self.flow_table.insert(flows, 0);
        self.grasp();
        let g = &mut self.g as *mut StreamAwareGraph;
        self.g.forget_all_flows();
        self.flow_table.foreach(true, |flow, k| {
            let r = self.get_kth_route(&flow, *k);
            unsafe { (*g).save_flowid_on_edge(true, *flow.id(), r) }
        });
    }
    fn get_retouted_flows(&self) -> &Vec<usize> {
        unimplemented!();
    }
    fn get_route(&self, id: usize) -> &Vec<usize> {
        let k = *self.flow_table.get_info(id);
        let flow = self.flow_table.get_flow(id);
        self.get_kth_route(flow, k)
    }
}