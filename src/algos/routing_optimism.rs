use crate::util::YensAlgo;
use super::{StreamAwareGraph, RouteTable, Flow, RoutingAlgo, GCL};
use super::time_estimate::compute_avb_latency;

const K: usize = 10;
const T_LIMIT: u128 = 1000 * 100; // micro_sec
const C1_EXCEED: f64 = 1000.0;
const W1: f64 = 1.0;
const W2: f64 = 1.0;
const W3: f64 = 1.0;

pub struct RO<'a> {
    g: StreamAwareGraph,
    route_table: RouteTable,
    yens_algo: YensAlgo<'a, usize, StreamAwareGraph>,
    gcl: GCL,
}

impl <'a> RO<'a> {
    pub fn new(g: &'a StreamAwareGraph, hyper_p: usize, gcl: GCL) -> Self {
        return RO {
            gcl,
            g: g.clone(),
            route_table: RouteTable::new(),
            yens_algo: YensAlgo::new(g, K),
        };
    }
}

impl <'a> RO<'a> {
    /// 在所有 TT 都被排定的狀況下去執行爬山算法
    fn hill_climbing(&self) {
        let time = std::time::Instant::now();
        let mut i = 0;
        while time.elapsed().as_micros() < T_LIMIT {
            let mut cost = 0.0;
            self.route_table.foreach_flow(|flow| {
                if let Flow::AVB { max_delay, .. } = *flow {
                    let latency = compute_avb_latency(
                        &self.g,
                        flow,
                        &self.route_table,
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
            //println!("{:?}", cost);
            i += 1;
        }
        println!("{}", i);
    }
}

impl <'a> RoutingAlgo for RO<'a> {
    fn compute_routes(&mut self, flows: Vec<Flow>) {
        for flow in flows.into_iter() {
            if let Flow::AVB { id, src, dst, .. } = flow {
                let rs = self.yens_algo.get_routes(src, dst);
                self.g.save_flowid_on_edge(true, id, &rs[0].1);
                self.route_table.insert(flow, rs[0].0, rs[0].1.clone());
            } else if let Flow::TT { id, src, dst, .. } = flow {
                let r = self.yens_algo.get_shortest_route(src, dst);
                self.g.save_flowid_on_edge(true, id, &r.1);
                self.route_table.insert(flow, r.0, r.1);
                // TODO 計算GCL
            }
        }
        self.hill_climbing();
    }
    fn get_retouted_flows(&self) -> &Vec<usize> {
        panic!("Not implemented!");
    }
    fn get_route(&self, id: usize) -> &Vec<usize> {
        return self.route_table.get_route(id);
    }
}