use crate::util::YensAlgo;
use super::{StreamAwareGraph, FlowTable, Flow, RoutingAlgo, GCL};
use super::time_and_tide::compute_avb_latency;

const K: usize = 10;
const T_LIMIT: u128 = 1000 * 100; // micro_sec
const C1_EXCEED: f64 = 1000.0;
const W1: f64 = 1.0;
const W2: f64 = 1.0;
const W3: f64 = 1.0;

pub struct RO<'a> {
    g: StreamAwareGraph,
    flow_table: FlowTable<usize>,
    yens_algo: YensAlgo<'a, usize, StreamAwareGraph>,
    gcl: GCL,
}

impl <'a> RO<'a> {
    pub fn new(g: &'a StreamAwareGraph, hyper_p: usize, gcl: GCL) -> Self {
        return RO {
            gcl,
            g: g.clone(),
            flow_table: FlowTable::new(),
            yens_algo: YensAlgo::new(g, K),
        };
    }
}

impl <'a> RO<'a> {
    /// 在所有 TT 都被排定的狀況下去執行爬山算法
    fn grasp(&mut self) {
        let time = std::time::Instant::now();
        let mut iter_times = 0;
        while time.elapsed().as_micros() < T_LIMIT {
            let mut cost = 0.0;
            self.flow_table.foreach_flow(|flow| {
                if let Flow::AVB { id, max_delay, .. } = *flow {
                    let route = self.get_kth_route(flow, *self.flow_table.get_info(id));
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
                    cost += W1*c1 + W2*c2 + W3*c3;
                }
            });
            iter_times += 1;
            println!("{:?}", cost);
        }
        println!("{}", iter_times);
    }
    fn get_kth_route(&self, flow: &Flow, k: usize) -> &Vec<usize> {
        self.yens_algo.get_kth_route(*flow.src(), *flow.dst(), k)
    }
}

impl <'a> RoutingAlgo for RO<'a> {
    fn compute_routes(&mut self, flows: Vec<Flow>) {
        for flow in flows.into_iter() {
            self.yens_algo.compute_routes(*flow.src(), *flow.dst());
            let r = self.yens_algo.get_kth_route(*flow.src(), *flow.dst(), 0);
            self.g.save_flowid_on_edge(true, *flow.id(), r);
            self.flow_table.insert(flow, 0);
        }
        self.grasp();
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