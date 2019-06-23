use crate::network_struct::Graph;
use crate::util::Dijkstra;
use super::{StreamAwareGraph, FlowTable, Flow, RoutingAlgo, GCL};
use super::time_and_tide::{schedule_online, compute_avb_latency};

const C1_EXCEED: f64 = 100.0;
const W1: f64 = 1.0;
const W2: f64 = 1.0;
const W3: f64 = 1.0;

pub struct SPF<'a> {
    g: StreamAwareGraph,
    flow_table: FlowTable<Vec<usize>>,
    gcl: GCL,
    dijkstra_algo: Dijkstra<'a, usize, StreamAwareGraph>,
    rerouted: Vec<usize>,
}

impl <'a> SPF<'a> {
    pub fn new(g: &'a StreamAwareGraph) -> Self {
        return SPF {
            g: g.clone(),
            rerouted: vec![],
            gcl: GCL::new(1, g.get_edge_cnt()),
            flow_table: FlowTable::new(),
            dijkstra_algo: Dijkstra::new(g),
        };
    }
}

impl <'a> RoutingAlgo for SPF<'a> {
    fn add_flows(&mut self, flows: Vec<Flow>) {
        self.flow_table.insert(flows.clone(), vec![]);
        let mut tt_changed = self.flow_table.clone_into_changed_table();
        for flow in flows.iter() {
            let r = self.get_shortest_route(flow);
            if flow.is_tt() {
                tt_changed.update_info(*flow.id(), r);
            }
        }
        // TT schedule
        let _self = self as *mut Self;
        unsafe {
            schedule_online(&mut (*_self).flow_table, &tt_changed, &mut (*_self).gcl,
                |flow, _| {
                    let r = self.get_shortest_route(flow);
                    self.g.get_links_id_bandwidth(&r)
                }
            ).unwrap();
        }

        let _g = &mut self.g as *mut StreamAwareGraph;
        self.flow_table.foreach(true, |flow, _| {
            let r = self.get_shortest_route(flow);
            unsafe {
                (*_g).save_flowid_on_edge(true, *flow.id(), &r);
            }
        });
    }
    fn del_flows(&mut self, flows: Vec<Flow>) {
        unimplemented!();
    }
    fn get_retouted_flows(&self) -> &Vec<usize> {
        return &self.rerouted;
    }
    fn get_route(&self, id: usize) -> &Vec<usize> {
        unimplemented!();
    }
    fn show_results(&self) {
        println!("TT Flows:");
        self.flow_table.foreach(false, |flow, _| {
            let route = self.get_shortest_route(flow);
            println!("flow id = {}, route = {:?}", *flow.id(), route);
        });
        println!("AVB Flows:");
        self.flow_table.foreach(true, |flow, _| {
            let route = self.get_shortest_route(flow);
            let cost = self.compute_avb_cost(flow);
            println!("flow id = {}, route = {:?} cost = {}", *flow.id(), route, cost);
        });
        println!("total avb cost = {}", self.compute_all_avb_cost());
    }
}
impl <'a> SPF<'a> {
    fn get_shortest_route(&self, flow: &Flow) -> Vec<usize> {
        let _dij = &self.dijkstra_algo as *const Dijkstra<usize, StreamAwareGraph>
            as *mut Dijkstra<usize, StreamAwareGraph>;
        unsafe { (*_dij).get_route(*flow.src(), *flow.dst()).unwrap().1 }
    }
    pub fn compute_avb_cost(&self, flow: &Flow) -> f64 {
        let max_delay = *flow.max_delay();
        let route = self.get_shortest_route(flow);
        let latency = compute_avb_latency(
            &self.g,
            flow,
            &route,
            &self.flow_table,
            &self.gcl
        );
        let c1 = if latency > max_delay {
            C1_EXCEED
        } else {
            0.0
        };
        let c2 = latency as f64 / max_delay as f64;
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
}