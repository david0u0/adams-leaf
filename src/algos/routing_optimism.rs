use std::collections::HashMap;
use std::cell::RefCell;

use crate::network_struct::Graph;
use crate::algos::{StreamAwareGraph, RouteTable, Flow,RoutingAlgo};
use crate::algos::cost_estimate;

fn f64_eq(a: f64, b: f64) -> bool {
    return (a - b).abs() < 0.0001;
}

type DistStruct = (f64, Vec<i32>);
pub struct RO {
    g: StreamAwareGraph,
    final_dist_map: HashMap<(i32, i32), RefCell<DistStruct>>,
    tt_table: RouteTable,
    avb_table: RouteTable,
    routed_node_table: HashMap<i32, bool>
}

impl RO {
    fn dijkstra(&mut self, src_id: i32) {
        if self.routed_node_table.contains_key(&src_id) {
            return;
        }
        self.routed_node_table.insert(src_id, true);
        let mut cur_id = src_id;
        let mut cur_dist = 0.0;
        let mut tmp_dist_map: HashMap<i32, DistStruct> = HashMap::new();
        self.final_dist_map.insert((src_id, src_id), RefCell::new((0.0, vec![])));
        loop {
            let cur_pair = (src_id, cur_id);
            // 塞進最終 dist map，並從暫存 dist map 中移除
            if let Some(entry) = tmp_dist_map.remove(&cur_id) {
                self.final_dist_map.insert(cur_pair, RefCell::new(entry));
            }

            self.g.foreach_edge(cur_id, |next_id, bandwidth| {
                let next_pair = (src_id, next_id);
                let next_dist = cur_dist + 1.0 / (bandwidth as f64);
                if let Some(rc_entry) = self.final_dist_map.get(&next_pair) {
                    let mut entry = rc_entry.borrow_mut();
                    if f64_eq(entry.0, next_dist) {
                        // NOTE: 到底會不會進到這裡？
                        entry.1.push(cur_id);
                    }
                } else if let Some(entry) = tmp_dist_map.get_mut(&next_id) {
                    if f64_eq(entry.0, next_dist) {
                        entry.1.push(cur_id);
                    } else {
                        tmp_dist_map.insert(next_id, (next_dist, vec![cur_id]));
                    }
                } else {
                    tmp_dist_map.insert(next_id, (next_dist, vec![cur_id]));
                }
            });

            let mut found = false;
            let mut min = std::f64::MAX;
            for (id, entry) in tmp_dist_map.iter() {
                if entry.0 < min {
                    found = true;
                    min = entry.0;
                    cur_dist = min;
                    cur_id = *id;
                }
            }
            if !found {
                break;
            }
        }
    }
    fn _recursive_get_route(&self, src_id: i32, dst_id: i32,
        route: &mut Vec<i32>, all_routes: &mut Vec<Vec<i32>>
    ) {
        route.push(dst_id);
        if src_id == dst_id {
            let route: Vec<i32> = route.iter().rev().map(|x| *x).collect();
            all_routes.push(route);
        } else {
            if let Some(rc_dist) = self.final_dist_map.get(&(src_id, dst_id)) {
                for &prev_id in rc_dist.borrow().1.iter() {
                    self._recursive_get_route(src_id, prev_id, route, all_routes);
                }
            }
        }
        route.pop();
    }
    pub fn get_multi_routes(&self, src_id: i32, dst_id: i32) -> Vec<Vec<i32>> {
        let mut all_routes: Vec<Vec<i32>> = vec![];
        self._recursive_get_route(src_id, dst_id, &mut vec![], &mut all_routes);
        return all_routes;
    }
}

impl RoutingAlgo<StreamAwareGraph> for RO {
    fn new(g: StreamAwareGraph) -> Self {
        return RO {
            g,
            final_dist_map: HashMap::new(),
            avb_table: HashMap::new(),
            tt_table: HashMap::new(),
            routed_node_table: HashMap::new()
        };
    }
    fn compute_routes(&mut self, flows: Vec<Flow>) {
        for flow in flows.into_iter() {
            match flow {
                Flow::AVB(flow) => {
                    self.dijkstra(flow.src);
                    let r = self.get_multi_routes(flow.src, flow.dst);
                    self.avb_table.insert(flow.id, (flow, Some(r[0].clone())));
                },
                Flow::TT(flow) => {
                    self.dijkstra(flow.src);
                    let r = self.get_multi_routes(flow.src, flow.dst);
                    self.tt_table.insert(flow.id, (flow, Some(r[0].clone())));
                },
            }
        }
    }
    fn get_retouted_flows(&self) -> Vec<i32> {
        panic!("Not implemented!");
    }
    fn get_route(&self, id: i32) -> Vec<i32> {
        if let Some(flow) = self.tt_table.get(&id) {
            if let Some(vec) = &flow.1 {
                return vec.clone();
            }
        } else if let Some(flow) = self.avb_table.get(&id) {
            if let Some(vec) = &flow.1 {
                return vec.clone();
            }
        }
        panic!();
    }
}