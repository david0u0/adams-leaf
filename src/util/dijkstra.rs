use std::collections::HashMap;
use std::cell::RefCell;

use crate::network_struct::Graph;
use crate::util::MyMinHeap;

pub struct Dijkstra<'a, G: Graph> {
    g: &'a G,
    final_dist_map: HashMap<(i32, i32), (f64, RefCell<i32>)>,
    routed_node_table: HashMap<i32, bool>
}

impl <'a, G: Graph> Dijkstra<'a, G> {
    pub fn new(g: &'a G) -> Self {
        return Dijkstra {
            g,
            final_dist_map: HashMap::new(),
            routed_node_table: HashMap::new(),
        }
    }
    pub fn compute_route(&mut self, src_id: i32) {
        if self.routed_node_table.contains_key(&src_id) {
            return;
        }
        self.routed_node_table.insert(src_id, true);

        let mut min_heap: MyMinHeap<f64, i32, RefCell<i32>> = MyMinHeap::new();
        min_heap.push( src_id, 0.0, RefCell::new(-1) );
        // 從優先權佇列中移除，並塞進最終 dist map
        while let Some((cur_id, cur_dist, backtrace)) = min_heap.pop() {
            self.final_dist_map.insert((src_id, cur_id),
                (cur_dist, backtrace));

            self.g.foreach_edge(cur_id, |next_id, bandwidth| {
                let next_pair = (src_id, next_id);
                let next_dist = cur_dist + 1.0 / bandwidth;
                if self.final_dist_map.contains_key(&next_pair) {
                    // DO NOTHING
                } else if let Some((og_dist, backtrace)) = min_heap.get(next_id) {
                    if *og_dist > next_dist {
                        (*backtrace.borrow_mut()) = cur_id;
                        min_heap.decrease_priority(next_id, next_dist);
                    }
                } else {
                    min_heap.push(next_id, next_dist, RefCell::new(cur_id));
                }
            });
        }
    }
    pub fn get_route(&mut self, src_id: i32, dst_id: i32) -> Option<Vec<i32>> {
        if !self.routed_node_table.contains_key(&src_id) {
            self.compute_route(src_id);
        }
        if !self.final_dist_map.contains_key(&(src_id, dst_id)) {
            // NOTE: 路徑無法連通
            return None;
        }
        return Some(self._recursive_get_route(src_id, dst_id));
    }
    fn _recursive_get_route(&self, src_id: i32, dst_id: i32) -> Vec<i32> {
        if src_id == dst_id {
            return vec![src_id];
        } else {
            let prev_id = self.final_dist_map.get(&(src_id, dst_id)).unwrap().1.borrow();
            let mut path = self._recursive_get_route(src_id, *prev_id);
            path.push(dst_id);
            return path;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::network_struct::Graph;
    use crate::algos::StreamAwareGraph;
    use super::Dijkstra;
    #[test]
    fn test_dijkstra1() {
        let mut g = StreamAwareGraph::new();
        g.add_host();
        g.add_host();
        g.add_host();
        g.add_edge((0, 1), 10.0);
        g.add_edge((1, 2), 20.0);
        g.add_edge((0, 2), 2.0);
        let mut algo = Dijkstra::new(&g);
        assert_eq!(vec![0, 1, 2], algo.get_route(0, 2).unwrap());
    }
    #[test]
    fn test_dijkstra2() {
        let mut g = StreamAwareGraph::new();
        g.add_host();
        g.add_host();
        g.add_host();
        g.add_host();
        g.add_host();
        g.add_host();
        g.add_edge((0, 1), 10.0);
        g.add_edge((1, 2), 20.0);
        g.add_edge((0, 2), 2.0);
        g.add_edge((1, 3), 10.0);
        g.add_edge((0, 3), 3.0);
        g.add_edge((3, 4), 3.0);
        let mut algo = Dijkstra::new(&g);
        assert_eq!(vec![0, 1, 3, 4], algo.get_route(0, 4).unwrap());
        assert_eq!(vec![2, 1, 3, 4], algo.get_route(2, 4).unwrap());
        assert_eq!(None, algo.get_route(0, 5));

        let mut g = g.clone();
        g.add_edge((4, 5), 2.0);
        let mut algo = Dijkstra::new(&g);
        assert_eq!(Some(vec![0, 1, 3, 4, 5]), algo.get_route(0, 5));
    }
}
