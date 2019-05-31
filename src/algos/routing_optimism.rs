use crate::util::Dijkstra;
use super::{StreamAwareGraph, RouteTable, Flow, RoutingAlgo, GCL};

pub struct RO<'a> {
    g: StreamAwareGraph,
    route_table: RouteTable,
    dijkstra_algo: Dijkstra<'a, usize, StreamAwareGraph>,
    gcl: GCL,
}

impl <'a> RO<'a> {
    pub fn new(g: &'a StreamAwareGraph, hyper_p: usize, gcl: GCL) -> Self {
        return RO {
            gcl,
            g: g.clone(),
            route_table: RouteTable::new(),
            dijkstra_algo: Dijkstra::new(g),
        };
    }
}

impl <'a> RoutingAlgo for RO<'a> {
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
        panic!("Not implemented!");
    }
    fn get_route(&self, id: usize) -> &Vec<usize> {
        return self.route_table.get_route(id);
    }
}