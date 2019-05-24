use std::collections::HashMap;
use std::cell::RefCell;

use crate::network_struct::Graph;
use crate::algos::{StreamAwareGraph, RouteTable, Flow, RoutingAlgo};
use crate::algos::cost_estimate;
use crate::util::Dijkstra;

fn f64_eq(a: f64, b: f64) -> bool {
    return (a - b).abs() < 0.0001;
}

pub struct RO<'a> {
    g: StreamAwareGraph,
    route_table: RouteTable,
    dijkstra_algo: Dijkstra<'a, StreamAwareGraph>
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
    fn get_retouted_flows(&self) -> Vec<i32> {
        panic!("Not implemented!");
    }
    fn get_route(&self, id: i32) -> Vec<i32> {
        if let Some((_, flow)) = self.route_table.get(&id) {
            if let Some((_, vec)) = &flow {
                return vec.clone();
            }
        }
        panic!();
    }
}