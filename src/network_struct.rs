pub trait Graph {
    fn new() -> Self;
    fn add_host(&mut self) -> i32;
    fn add_switch(&mut self) -> i32;
    fn add_edge(&mut self, id_pair: (i32, i32), bandwidth: f64);
    fn del_edge(&mut self, id_pair: (i32, i32)) -> f64;
    fn del_node(&mut self, id: i32) -> bool;
    fn foreach_edge(&self, id: i32, callback: impl FnMut(i32, f64) -> ());
    fn foreach_node(&self, callback: impl FnMut(i32, bool) -> ());
}