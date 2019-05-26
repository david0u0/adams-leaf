use std::hash::Hash;

use super::Dijkstra;
use crate::network_struct::OnOffGraph;

pub struct YensAlgo<'a, K: Hash+Eq+Copy, G: OnOffGraph<K>> {
    g: &'a G,
    k: usize,
    dijkstra_algo: Dijkstra<'a, K, G>,
}

impl <'a, K: Hash+Eq+Copy, G: OnOffGraph<K>> YensAlgo<'a, K, G> {
    pub fn new(g: &'a G, k: usize) -> Self {
        return YensAlgo {
            g, k,
            dijkstra_algo: Dijkstra::new(g)
        }
    }
}