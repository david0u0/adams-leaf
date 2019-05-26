use std::collections::HashMap;

use crate::network_struct::Graph;
use crate::util::Dijkstra;
use super::{StreamAwareGraph, RouteTable, Flow, RoutingAlgo};
use super::cost_estimate;

pub struct RO<'a> {
    g: StreamAwareGraph,
    route_table: RouteTable,
    dijkstra_algo: Dijkstra<'a, usize, StreamAwareGraph>
}

impl <'a> RO<'a> {
    pub fn new(g: &'a StreamAwareGraph) -> Self {
        return RO {
            g: g.clone(),
            route_table: HashMap::new(),
            dijkstra_algo: Dijkstra::new(g)
        };
    }
}

impl <'a> RoutingAlgo for RO<'a> {
    fn compute_routes(&mut self, flows: Vec<Flow>) {
        for flow in flows.into_iter() {
            if let Flow::AVB { id, src, dst, .. } = flow {
                let r = self.dijkstra_algo.get_route(src, dst);
                self.route_table.insert(id, (flow, r));
            } else if let Flow::TT { id, src, dst, .. } = flow {
                let r = self.dijkstra_algo.get_route(src, dst);
                self.route_table.insert(id, (flow, r));
            }
        }
    }
    fn get_retouted_flows(&self) -> Vec<usize> {
        panic!("Not implemented!");
    }
    fn get_route(&self, id: usize) -> Vec<usize> {
        if let Some((_, route)) = self.route_table.get(&id) {
            return route.1.clone();
        } else {
            panic!("查無資料流");
        }
    }
}