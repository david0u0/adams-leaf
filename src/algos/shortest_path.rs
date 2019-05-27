use std::collections::HashMap;

use crate::network_struct::Graph;
use crate::util::Dijkstra;
use super::{StreamAwareGraph, RouteTable, Flow, RoutingAlgo};
use super::cost_estimate;

pub struct SPF<'a> {
    g: StreamAwareGraph,
    route_table: RouteTable,
    dijkstra_algo: Dijkstra<'a, usize, StreamAwareGraph>
}

impl <'a> SPF<'a> {
    pub fn new(g: &'a StreamAwareGraph) -> Self {
        return SPF {
            g: g.clone(),
            route_table: vec![],
            dijkstra_algo: Dijkstra::new(g)
        };
    }
}

impl <'a> RoutingAlgo for SPF<'a> {
    fn compute_routes(&mut self, flows: Vec<Flow>) {
        for flow in flows.into_iter() {
            if let Flow::AVB { id, src, dst, .. } = flow {
                let r = self.dijkstra_algo.get_route(src, dst);
                if id == self.route_table.len() {
                    self.route_table.push((flow, r.0, r.1));
                } else if id < self.route_table.len() {
                    self.route_table[id] = (flow, r.0, r.1);
                } else {
                    panic!("請按順序填入資料流");
                }
            } else if let Flow::TT { id, src, dst, .. } = flow {
                let r = self.dijkstra_algo.get_route(src, dst);
                if id == self.route_table.len() {
                    self.route_table.push((flow, r.0, r.1));
                } else if id < self.route_table.len() {
                    self.route_table[id] = (flow, r.0, r.1);
                } else {
                    panic!("請按順序填入資料流");
                }
            }
        }
    }
    fn get_retouted_flows(&self) -> &Vec<usize> {
        panic!("Not implemented!");
    }
    fn get_route(&self, id: usize) -> &Vec<usize> {
        return &self.route_table[id].2;
    }
}