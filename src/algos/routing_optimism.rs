use std::collections::HashMap;
use std::cell::RefCell;

use crate::network_struct::Graph;
use crate::algos::{StreamAwareGraph, RouteTable, Flow, RoutingAlgo};
use crate::algos::util::MyMinHeap;
use crate::algos::cost_estimate;

fn f64_eq(a: f64, b: f64) -> bool {
    return (a - b).abs() < 0.0001;
}

type BackTraceStruct = RefCell<Vec<i32>>;
pub struct RO {
    g: StreamAwareGraph,
    final_dist_map: HashMap<(i32, i32), (f64, BackTraceStruct)>,
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
        let mut min_heap: MyMinHeap<f64, i32, BackTraceStruct> = MyMinHeap::new();
        min_heap.push( src_id, 0.0, RefCell::new(vec![]) );

        // 從優先權佇列中移除，並塞進最終 dist map
        while let Some((cur_id, cur_dist, backtrace)) = min_heap.pop() {
            self.final_dist_map.insert((src_id, cur_id),
                (cur_dist, backtrace));

            self.g.foreach_edge(cur_id, |next_id, bandwidth| {
                let next_pair = (src_id, next_id);
                let next_dist = cur_dist + 1.0 / bandwidth;
                if let Some(entry) = self.final_dist_map.get(&next_pair) {
                    if f64_eq(entry.0, next_dist) {
                        // NOTE: 到底會不會進到這裡？
                        entry.1.borrow_mut().push(cur_id);
                    }
                } else if let Some((og_dist, backtrace)) = min_heap.get(next_id) {
                    if f64_eq(*og_dist, next_dist) {
                        backtrace.borrow_mut().push(cur_id);
                    } else if *og_dist > next_dist {
                        backtrace.borrow_mut().push(cur_id);
                        min_heap.decrease_priority(next_id, next_dist);
                    }
                } else {
                    min_heap.push(next_id, next_dist, RefCell::new(vec![cur_id]));
                }
            });
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
            if let Some((_, backtrace)) = self.final_dist_map.get(&(src_id, dst_id)) {
                for &prev_id in backtrace.borrow().iter() {
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