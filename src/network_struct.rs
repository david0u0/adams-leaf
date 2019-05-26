use std::hash::Hash;

pub trait Graph<K: Hash+Eq>: Clone {
    fn add_host(&mut self, cnt: Option<usize>) -> Vec<K>;
    fn add_switch(&mut self, cnt: Option<usize>) -> Vec<K>;
    fn add_edge(&mut self, id_pair: (K, K), bandwidth: f64) -> Result<(), String>;
    fn del_edge(&mut self, id_pair: (K, K)) -> Result<f64, String>;
    fn del_node(&mut self, id: K) -> Result<(), String>;
    fn foreach_edge(&self, id: K, callback: impl FnMut(K, f64) -> ());
    fn foreach_node(&self, callback: impl FnMut(K, bool) -> ());
}

pub trait OnOffGraph<K: Hash+Eq>: Graph<K> {
    fn activate_edge(&mut self, id_pair: (K, K)) -> Result<(), String>;
    fn inactivate_edge(&mut self, id_pair: (K, K)) -> Result<(), String>;
}