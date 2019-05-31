use crate::util::Dijkstra;
use super::{StreamAwareGraph, RouteTable, Flow, RoutingAlgo};

pub struct SPF<'a> {
    route_table: RouteTable,
    dijkstra_algo: Dijkstra<'a, usize, StreamAwareGraph>,
    rerouted: Vec<usize>,
}

impl <'a> SPF<'a> {
    pub fn new(g: &'a StreamAwareGraph) -> Self {
        return SPF {
            rerouted: vec![],
            route_table: RouteTable::new(),
            dijkstra_algo: Dijkstra::new(g),
        };
    }
}

impl <'a> RoutingAlgo for SPF<'a> {
    fn compute_routes(&mut self, flows: Vec<Flow>) {
        for flow in flows.into_iter() {
            if let Flow::AVB { src, dst, .. } = flow {
                let r = self.dijkstra_algo.get_route(src, dst);
                self.route_table.insert(flow, r.0, r.1);
            } else if let Flow::TT { src, dst, .. } = flow {
                let r = self.dijkstra_algo.get_route(src, dst);
                self.route_table.insert(flow, r.0, r.1);
            }
        }
    }
    fn get_retouted_flows(&self) -> &Vec<usize> {
        return &self.rerouted;
    }
    fn get_route(&self, id: usize) -> &Vec<usize> {
        return self.route_table.get_route(id);
    }
}