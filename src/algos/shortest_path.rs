use crate::util::Dijkstra;
use super::{StreamAwareGraph, FlowTable, Flow, RoutingAlgo};

pub struct SPF<'a> {
    flow_table: FlowTable<Vec<usize>>,
    dijkstra_algo: Dijkstra<'a, usize, StreamAwareGraph>,
    rerouted: Vec<usize>,
}

impl <'a> SPF<'a> {
    pub fn new(g: &'a StreamAwareGraph) -> Self {
        return SPF {
            rerouted: vec![],
            flow_table: FlowTable::new(),
            dijkstra_algo: Dijkstra::new(g),
        };
    }
}

impl <'a> RoutingAlgo for SPF<'a> {
    fn compute_routes(&mut self, flows: Vec<Flow>) {
        for flow in flows.into_iter() {
            if let Flow::AVB { src, dst, .. } = flow {
                let r = self.dijkstra_algo.get_route(src, dst);
                self.flow_table.insert(flow, r.1);
            } else if let Flow::TT { src, dst, .. } = flow {
                let r = self.dijkstra_algo.get_route(src, dst);
                self.flow_table.insert(flow, r.1);
            }
        }
    }
    fn get_retouted_flows(&self) -> &Vec<usize> {
        return &self.rerouted;
    }
    fn get_route(&self, id: usize) -> &Vec<usize> {
        return self.flow_table.get_info(id);
    }
}