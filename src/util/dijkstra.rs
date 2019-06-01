use std::collections::HashMap;
use std::hash::Hash;
use std::cell::Cell;

use crate::network_struct::Graph;
use super::MyMinHeap;

pub struct Dijkstra<'a, K: Hash+Eq+Copy, G: Graph<K>> {
    g: &'a G,
    final_dist_map: HashMap<(K, K), (f64, Cell<K>)>,
    routed_node_table: HashMap<K, bool>
}

impl <'a, K: Hash+Eq+Copy, G: Graph<K>> Dijkstra<'a, K, G> {
    pub fn new(g: &'a G) -> Self {
        return Dijkstra {
            g,
            final_dist_map: HashMap::new(),
            routed_node_table: HashMap::new(),
        }
    }
    pub fn compute_route(&mut self, src_id: K) {
        if self.routed_node_table.contains_key(&src_id) {
            return;
        }
        self.routed_node_table.insert(src_id, true);

        let mut min_heap: MyMinHeap<f64, K, Cell<K>> = MyMinHeap::new();
        min_heap.push( src_id, 0.0, Cell::new(src_id) );
        // 從優先權佇列中移除，並塞進最終 dist map
        while let Some((cur_id, cur_dist, backtrace)) = min_heap.pop() {
            self.final_dist_map.insert((src_id, cur_id),
                (cur_dist, backtrace));
            
            self.g.foreach_edge(cur_id, |next_id, bandwidth| {
                let next_pair = (src_id, next_id);
                let next_dist = cur_dist + 1.0 / bandwidth;
                if self.final_dist_map.contains_key(&next_pair) {
                    // DO NOTHING
                } else if let Some((og_dist, backtrace)) = min_heap.get(&next_id) {
                    if *og_dist > next_dist {
                        backtrace.set(cur_id);
                        min_heap.decrease_priority(&next_id, next_dist);
                    }
                } else {
                    min_heap.push(next_id, next_dist, Cell::new(cur_id));
                }
            });
        }
    }
    pub fn get_dist(&mut self, src_id: K, dst_id: K) -> f64 {
        if !self.routed_node_table.contains_key(&src_id) {
            self.compute_route(src_id);
        }
        if let Some(entry) = self.final_dist_map.get(&(src_id, dst_id)) {
            return entry.0;
        } else {
            // NOTE: 路徑無法連通
            return std::f64::MAX;
        }
    }
    pub fn get_route(&mut self, src_id: K, dst_id: K) -> Option<(f64, Vec<K>)> {
        if !self.routed_node_table.contains_key(&src_id) {
            self.compute_route(src_id);
        }
        if let Some(entry) = self.final_dist_map.get(&(src_id, dst_id)) {
            Some((entry.0, self._recursive_get_route(src_id, dst_id)))
        } else {
            // NOTE: 路徑無法連通
            None
        }
    }
    fn _recursive_get_route(&self, src_id: K, dst_id: K) -> Vec<K> {
        if src_id == dst_id {
            vec![src_id]
        } else {
            let prev_id = self.final_dist_map.get(&(src_id, dst_id)).unwrap().1.get();
            let mut path = self._recursive_get_route(src_id, prev_id);
            path.push(dst_id);
            path
        }
    }
}

#[cfg(test)]
mod test {
    use crate::network_struct::Graph;
    use crate::algos::StreamAwareGraph;
    use super::Dijkstra;
    #[test]
    fn test_dijkstra1() -> Result<(), String> {
        let mut g = StreamAwareGraph::new();
        g.add_host(Some(3));
        g.add_edge((0, 1), 10.0)?;
        g.add_edge((0, 1), 10.0)?;
        g.add_edge((1, 2), 20.0)?;
        g.add_edge((0, 2), 2.0)?;
        let mut algo = Dijkstra::new(&g);
        assert_eq!(vec![0, 1, 2], algo.get_route(0, 2).unwrap().1);
        Ok(())
    }
    #[test]
    fn test_dijkstra2() -> Result<(), String> {
        let mut g = StreamAwareGraph::new();
        g.add_host(Some(6));
        g.add_edge((0, 1), 10.0)?;
        g.add_edge((1, 2), 20.0)?;
        g.add_edge((0, 2), 2.0)?;
        g.add_edge((1, 3), 10.0)?;
        g.add_edge((0, 3), 3.0)?;
        g.add_edge((3, 4), 3.0)?;

        let mut algo = Dijkstra::new(&g);
        assert_eq!(vec![0, 1, 3, 4], algo.get_route(0, 4).unwrap().1);
        assert_eq!(vec![2, 1, 3, 4], algo.get_route(2, 4).unwrap().1);
        assert!(algo.get_route(0, 5).is_none());

        let mut g = g.clone();
        g.add_edge((4, 5), 2.0)?;
        let mut algo = Dijkstra::new(&g);
        assert_eq!(vec![0, 1, 3, 4, 5], algo.get_route(0, 5).unwrap().1);
        Ok(())
    }
}
