use super::Dijkstra;
use crate::network_struct::OnOffGraph;

pub struct YensAlgo<'a, G: OnOffGraph> {
    g: &'a G,
    k: usize,
    dijkstra_algo: Dijkstra<'a, G>,
}

impl <'a, G: OnOffGraph> YensAlgo<'a, G> {
    pub fn new(g: &'a G, k: usize) -> Self {
        return YensAlgo {
            g, k,
            dijkstra_algo: Dijkstra::new(g)
        }
    }
}