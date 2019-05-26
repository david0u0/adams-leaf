use std::hash::Hash;
use std::collections::HashMap;
use std::rc::Rc;

use super::Dijkstra;
use super::MyMinHeap;
use crate::network_struct::OnOffGraph;

type Path<K> = (f64, Vec<K>);

pub struct YensAlgo<'a, K: Hash+Eq+Copy, G: OnOffGraph<K>> {
    g: G,
    k: usize,
    route_table: HashMap<(K, K), Vec<Path<K>>>,
    dijkstra_algo: Dijkstra<'a, K, G>,
}

impl <'a, K: Hash+Eq+Copy, G: OnOffGraph<K>> YensAlgo<'a, K, G> {
    pub fn new(g: &'a G, k: usize) -> Self {
        return YensAlgo {
            k,
            g: g.clone(),
            route_table: HashMap::new(),
            dijkstra_algo: Dijkstra::new(g)
        }
    }
    pub fn get_routes(&mut self, src: K, dst: K) -> &Vec<Path<K>> {
        let pair = (src, dst);
        if !self.route_table.contains_key(&pair) {
            self.compute_routes(src, dst);
        }
        return self.route_table.get(&pair).unwrap();
    }
    #[allow(unused_must_use)]
    pub fn compute_routes(&mut self, src: K, dst: K) {
        if self.route_table.contains_key(&(src, dst)) {
            return;
        }
        let mut paths: HashMap<Rc<Vec<K>>, f64> = HashMap::new();
        let mut min_heap: MyMinHeap<f64, Rc<Vec<K>>, ()> = MyMinHeap::new();
        let shortest = self.dijkstra_algo.get_route(src, dst);
        if self.k == 1 {
            self.route_table.insert((src, dst), vec![shortest]);
            return;
        }
        min_heap.push(Rc::new(shortest.1), shortest.0, ());
        while let Some((cur_path, dist, _)) = min_heap.pop() {
            paths.insert(cur_path.clone(), dist);
            let mut prefix: Vec<K> = vec![];
            for i in 0..cur_path.len()-1 {
                let cur_node = cur_path[i];
                if i >= 1 {
                    self.g.inactivate_node(cur_path[i-1]);
                }
                self.g.inactivate_edge((cur_node, cur_path[i+1]));
                let mut spf = Dijkstra::new(&self.g);
                let (_, postfix) = spf.get_route(cur_node, dst);
                let mut next_path = prefix.clone();
                next_path.extend(postfix);
                let dist = self.g.get_dist(&next_path);
                if !paths.contains_key(&next_path) {
                    let next_path = Rc::new(next_path);
                    if !min_heap.contains_key(&next_path) {
                        min_heap.push(next_path, dist, ());
                    }
                }
                prefix.push(cur_node);
            }
            self.g.reset();

            if paths.len() >= self.k {
                break;
            }
        }
        drop(min_heap);
        let mut vec: Vec<Path<K>> = paths.into_iter().map(|(vec, dist)| {
            if let Ok(vec) = Rc::try_unwrap(vec) {
                (dist, vec)
            } else {
                panic!("取 Rc 值時發生問題");
            }
        }).collect();
        vec.sort_by(|a, b| {
            if a.0 > b.0 {
                return std::cmp::Ordering::Greater;
            } else if a.0 < b.0 {
                return std::cmp::Ordering::Less;
            } else {
                return std::cmp::Ordering::Equal;
            }
        });
        self.route_table.insert((src, dst), vec);
    }
}

#[cfg(test)]
mod test {
    use crate::network_struct::Graph;
    use crate::algos::StreamAwareGraph;
    use super::YensAlgo;
    #[test]
    fn test_yens_algo1() -> Result<(), String> {
        let mut g = StreamAwareGraph::new();
        g.add_host(Some(5));
        g.add_edge((0, 1), 10.0)?;
        g.add_edge((1, 2), 20.0)?;
        g.add_edge((0, 2), 2.0)?;
        g.add_edge((1, 4), 10.0)?;
        g.add_edge((1, 3), 15.0)?;
        g.add_edge((2, 3), 10.0)?;
        g.add_edge((2, 4), 10.0)?;
        let mut algo = YensAlgo::new(&g, 3);
        //println!("{:?}", algo.get_routes(0, 2));
        assert_eq!(vec![0, 1, 2], algo.get_routes(0, 2)[0].1);
        assert_eq!(vec![0, 1, 3, 2], algo.get_routes(0, 2)[1].1);
        assert_eq!(vec![0, 1, 4, 2], algo.get_routes(0, 2)[2].1);
        return Ok(());
    }
}