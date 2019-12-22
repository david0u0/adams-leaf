use super::{
    time_and_tide::{compute_avb_latency, schedule_online},
    RoutingAlgo,
};
use crate::flow::{AVBFlow, Flow, FlowID, TSNFlow};
use crate::graph_util::{Graph, MemorizingGraph, StreamAwareGraph};
use crate::recorder::{flow_table::prelude::*, GCL};
use crate::util::Dijkstra;
use crate::{W1, W2, W3};

pub struct SPF {
    g: MemorizingGraph,
    flow_table: FlowTable<Vec<usize>>,
    gcl: GCL,
    dijkstra_algo: Dijkstra<usize, StreamAwareGraph>,
}

impl SPF {
    pub fn new(g: StreamAwareGraph) -> Self {
        return SPF {
            g: MemorizingGraph::new(g.clone()),
            gcl: GCL::new(1, g.get_edge_cnt()),
            flow_table: FlowTable::new(),
            dijkstra_algo: Dijkstra::new(g),
        };
    }
}

impl RoutingAlgo for SPF {
    fn get_last_compute_time(&self) -> u128 {
        unimplemented!();
    }
    fn add_flows(&mut self, tsns: Vec<TSNFlow>, avbs: Vec<AVBFlow>) {
        self.flow_table.insert(tsns.clone(), avbs.clone(), vec![]);
        let mut tt_changed = self.flow_table.clone_as_diff();
        for (flow, _) in self.flow_table.iter_avb() {
            self.get_shortest_route(flow);
        }
        for (flow, _) in self.flow_table.iter_tsn() {
            let r = self.get_shortest_route(flow);
            tt_changed.update_info_force(flow.id, r);
        }

        // TT schedule
        let _self = self as *mut Self;
        unsafe {
            schedule_online(
                &mut (*_self).flow_table,
                &tt_changed,
                &mut (*_self).gcl,
                |flow, _| {
                    let r = self.get_shortest_route(flow);
                    self.g.get_links_id_bandwidth(&r)
                },
            )
            .unwrap();
        }

        let _g = &mut self.g as *mut MemorizingGraph;
        for (flow, r) in self.flow_table.iter_avb() {
            let r = self.get_shortest_route(flow);
            unsafe {
                (*_g).update_flowid_on_route(true, flow.id, &r);
            }
        }
    }
    fn del_flows(&mut self, tsns: Vec<TSNFlow>, avbs: Vec<AVBFlow>) {
        unimplemented!();
    }
    fn get_rerouted_flows(&self) -> &Vec<FlowID> {
        unimplemented!();
    }
    fn get_route(&self, id: FlowID) -> &Vec<usize> {
        unimplemented!();
    }
    fn show_results(&self) {
        println!("TT Flows:");
        for (flow, _) in self.flow_table.iter_tsn() {
            let route = self.get_shortest_route(flow);
            println!("flow id = {:?}, route = {:?}", flow.id, route);
        }
        println!("AVB Flows:");
        for (flow, _) in self.flow_table.iter_avb() {
            let route = self.get_shortest_route(flow);
            let cost = self.compute_avb_cost(flow);
            println!(
                "flow id = {:?}, route = {:?} cost = {}",
                flow.id, route, cost
            );
        }
        println!("total avb cost = {}", self.compute_all_avb_cost());
    }
}
impl SPF {
    fn get_shortest_route<T: Clone>(&self, flow: &Flow<T>) -> Vec<usize> {
        let _dij = &self.dijkstra_algo as *const Dijkstra<usize, StreamAwareGraph>
            as *mut Dijkstra<usize, StreamAwareGraph>;
        unsafe { (*_dij).get_route(flow.src, flow.dst).unwrap().1 }
    }
    pub fn compute_avb_cost(&self, flow: &AVBFlow) -> f64 {
        let max_delay = flow.max_delay;
        let route = self.get_shortest_route(flow);
        let latency = compute_avb_latency(&self.g, flow, &route, &self.flow_table, &self.gcl);
        let c1 = if latency > max_delay { 1.0 } else { 0.0 };
        let c2 = latency as f64 / max_delay as f64;
        let c3 = 0.0; // TODO 計算 c3
        W1 * c1 + W2 * c2 + W3 * c3
    }
    pub fn compute_all_avb_cost(&self) -> f64 {
        let mut cost = 0.0;
        for (flow, _) in self.flow_table.iter_avb() {
            cost += self.compute_avb_cost(flow);
        }
        cost
    }
}
