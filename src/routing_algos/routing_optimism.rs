use super::{
    time_and_tide::{compute_avb_latency, schedule_online},
    RoutingAlgo,
};
use crate::flow::{AVBFlow, Flow, FlowID, TSNFlow};
use crate::graph_util::{Graph, MemorizingGraph, StreamAwareGraph};
use crate::recorder::{flow_table::prelude::*, GCL};
use crate::util::YensAlgo;
use crate::{FAST_STOP, W1, W2, W3};
use crate::{MAX_K, T_LIMIT};
use rand::Rng;
use std::time::Instant;

type FT = FlowTable<usize>;

const ALPHA_PORTION: f64 = 0.5;

type AVBCostResult = (u32, f64);

fn gen_n_distinct_outof_k(n: usize, k: usize) -> Vec<usize> {
    let mut vec = Vec::with_capacity(n);
    for i in 0..k {
        vec.push((rand::thread_rng().gen::<usize>(), i));
    }
    vec.sort();
    vec.into_iter().map(|(_, i)| i).take(n).collect()
}

pub struct RO {
    g: MemorizingGraph,
    flow_table: FT,
    yens_algo: YensAlgo<usize, StreamAwareGraph>,
    gcl: GCL,
    compute_time: u128,
}

impl RO {
    pub fn new(g: StreamAwareGraph, flow_table: Option<FT>, gcl: Option<GCL>) -> Self {
        let flow_table = flow_table.unwrap_or(FlowTable::new());
        let gcl = gcl.unwrap_or(GCL::new(1, g.get_edge_cnt()));
        RO {
            g: MemorizingGraph::new(g.clone()),
            yens_algo: YensAlgo::new(g, MAX_K),
            gcl,
            flow_table,
            compute_time: 0,
        }
    }
    pub fn compute_avb_cost(&self, flow: &AVBFlow, k: Option<usize>) -> AVBCostResult {
        let k = match k {
            Some(t) => t,
            None => *self.flow_table.get_info(flow.id).unwrap(),
        };
        let max_delay = flow.max_delay;
        let route = self.get_kth_route(flow, k);
        let latency = compute_avb_latency(&self.g, flow, route, &self.flow_table, &self.gcl);
        let c1 = if latency > max_delay { 1.0 } else { 0.0 };
        let c2 = latency as f64 / max_delay as f64;
        let c3 = 0.0; // TODO 計算 c3
        if c1 > 0.1 {
            (1, W1 * c1 + W2 * c2 + W3 * c3)
        } else {
            (0, W1 * c1 + W2 * c2 + W3 * c3)
        }
    }
    pub fn compute_all_avb_cost(&self) -> AVBCostResult {
        let mut all_cost = 0.0;
        let mut all_fail_cnt = 0;
        for (flow, _) in self.flow_table.iter_avb() {
            let (fail_cnt, cost) = self.compute_avb_cost(flow, None);
            all_fail_cnt += fail_cnt;
            all_cost += cost;
        }
        (all_fail_cnt, all_cost)
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
            unsafe {
                let table = &mut *(&mut self.flow_table as *mut FlowTable<usize>);
                for (flow, route_k) in table.iter_avb_mut() {
                    let candidate_cnt = self.get_candidate_count(flow);
                    let alpha = (candidate_cnt as f64 * ALPHA_PORTION) as usize;
                    let set = Some(gen_n_distinct_outof_k(alpha, candidate_cnt));
                    *route_k = self.find_min_cost_route(flow, set);
                    // NOTE 把資料流的路徑與ID記憶到圖中
                    self.update_flowid_on_route(true, flow, *route_k);
                }
            }
            // PHASE 2
            let (fail_cnt, cost) = self.compute_all_avb_cost();
            if cost < min_cost {
                best_all_routing = self.flow_table.clone();
                min_cost = cost;
                println!("found min_cost = {} at first glance! {}", cost, fail_cnt);
                if fail_cnt == 0 {
                    break;
                }
            }
            println!("start iteration #{}", iter_times);
            let res = self.hill_climbing(&time, min_cost, &mut best_all_routing);
            min_cost = res.1;
            if res.0 == 0 && FAST_STOP {
                // 找到可行解，且為快速終止模式
                break;
            }
        }
        self.flow_table = best_all_routing;
    }
    fn find_min_cost_route(&self, flow: &AVBFlow, set: Option<Vec<usize>>) -> usize {
        let (mut min_cost, mut best_k) = (std::f64::MAX, 0);
        let mut closure = |k: usize| {
            let (_, cost) = self.compute_avb_cost(flow, Some(k));
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
    ) -> AVBCostResult {
        let mut iter_times = 0;
        let mut best_fail_cnt = std::u32::MAX;
        while time.elapsed().as_micros() < T_LIMIT {
            let target_id: FlowID = rand::thread_rng()
                .gen_range(0, self.flow_table.get_flow_cnt())
                .into();
            let target_flow = {
                // TODO 用更好的機制篩選 avb flow
                if let Some(t) = self.flow_table.get_avb(target_id) {
                    t
                } else {
                    continue;
                }
            };
            let old_route = *self.flow_table.get_info(target_id).unwrap();
            // 從圖中忘記舊路徑
            unsafe {
                self.update_flowid_on_route(false, target_flow, old_route);
            }
            let new_route = self.find_min_cost_route(target_flow, None);
            let (fail_cnt, cost) = if old_route == new_route {
                (std::u32::MAX, std::f64::MAX)
            } else {
                // 在圖中記憶新路徑
                let _s = self as *const Self as *mut Self;
                unsafe {
                    self.update_flowid_on_route(true, target_flow, new_route);
                    (*_s).flow_table.update_info(target_id, new_route);
                }
                self.compute_all_avb_cost()
            };
            if cost < min_cost {
                *best_all_routing = self.flow_table.clone();
                self.flow_table.update_info(target_id, new_route);

                min_cost = cost;
                best_fail_cnt = fail_cnt;
                iter_times = 0;
                println!("found min_cost = {}", cost);

                if fail_cnt == 0 {
                    return (0, cost); // 找到可行解，返回
                }
            } else {
                // 恢復上一動
                unsafe {
                    self.update_flowid_on_route(false, target_flow, new_route);
                }
                unsafe {
                    self.update_flowid_on_route(true, target_flow, old_route);
                }
                self.flow_table.update_info(target_id, old_route);

                iter_times += 1;
                if iter_times == self.flow_table.get_flow_cnt() {
                    break;
                }
            }
        }
        (best_fail_cnt, min_cost)
    }
    fn get_kth_route<T: Clone>(&self, flow: &Flow<T>, k: usize) -> &Vec<usize> {
        self.yens_algo.get_kth_route(flow.src, flow.dst, k)
    }
    fn get_candidate_count<T: Clone>(&self, flow: &Flow<T>) -> usize {
        self.yens_algo.get_route_count(flow.src, flow.dst)
    }
    unsafe fn update_flowid_on_route<T: Clone>(&self, remember: bool, flow: &Flow<T>, k: usize) {
        let _g = &self.g as *const MemorizingGraph as *mut MemorizingGraph;
        let route = self.get_kth_route(flow, k);
        (*_g).update_flowid_on_route(remember, flow.id, route);
    }
}
impl RoutingAlgo for RO {
    fn add_flows(&mut self, tsns: Vec<TSNFlow>, avbs: Vec<AVBFlow>) {
        let init_time = Instant::now();
        let new_ids = self.flow_table.insert(tsns, avbs, 0);
        let mut reconf = self.flow_table.clone_as_diff();
        unsafe {
            let _self = &mut *(self as *mut Self);
            for &id in new_ids.iter() {
                reconf.update_info_force(id, 0); // 這裡好像不用管 avb，不過…管他的
                if let Some(flow) = self.flow_table.get_avb(id) {
                    _self.yens_algo.compute_routes(flow.src, flow.dst);
                } else if let Some(flow) = self.flow_table.get_tsn(id) {
                    _self.yens_algo.compute_routes(flow.src, flow.dst);
                }
            }
        }
        let time = Instant::now();
        // TT schedule
        unsafe {
            let _self = self as *mut Self;
            schedule_online(
                &mut (*_self).flow_table,
                &reconf,
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
        for (flow, &r) in self.flow_table.iter_avb() {
            unsafe {
                self.update_flowid_on_route(true, flow, r);
            }
        }

        self.compute_time = init_time.elapsed().as_micros();
    }
    fn del_flows(&mut self, tsns: Vec<TSNFlow>, avbs: Vec<AVBFlow>) {
        unimplemented!();
    }
    fn get_rerouted_flows(&self) -> &Vec<FlowID> {
        unimplemented!();
    }
    fn get_route(&self, id: FlowID) -> &Vec<usize> {
        let k = *self.flow_table.get_info(id).unwrap();
        if let Some(flow) = self.flow_table.get_avb(id) {
            self.get_kth_route(flow, k)
        } else if let Some(flow) = self.flow_table.get_tsn(id) {
            self.get_kth_route(flow, k)
        } else {
            panic!("啥都找不到！");
        }
    }
    fn show_results(&self) {
        println!("TT Flows:");
        for (flow, &route_k) in self.flow_table.iter_tsn() {
            let route = self.get_kth_route(flow, route_k);
            println!("flow id = {:?}, route = {:?}", flow.id, route);
        }
        println!("AVB Flows:");
        for (flow, &route_k) in self.flow_table.iter_avb() {
            let route = self.get_kth_route(flow, route_k);
            let (_, cost) = self.compute_avb_cost(flow, Some(route_k));
            println!(
                "flow id = {:?}, route = {:?} cost = {}",
                flow.id, route, cost
            );
        }
        println!("total avb cost = {}", self.compute_all_avb_cost().1);
    }
    fn get_last_compute_time(&self) -> u128 {
        self.compute_time
    }
}
