use std::collections::HashMap;
use crate::network_struct::Graph;

pub struct FlowStruct {
    pub id: i32,
    pub src: i32,
    pub dst: i32,
    pub size: u32,
    pub period: u32,
    pub max_delay: u32,
}
pub enum Flow {
    AVB(FlowStruct),
    TT(FlowStruct),
}

pub trait RoutingAlgo {
    fn new(g: Graph) -> Self;
    fn compute_routes(&mut self, flows: Vec<Flow>);
    fn get_retouted_flows(&self) -> Vec<i32>;
    fn get_route(&self, id: i32) -> Vec<i32>;
}

pub type RouteTable = HashMap<i32, (FlowStruct, Option<Vec<i32>>)>;

mod dijkstra;
pub use dijkstra::Dijkstra;

mod routing_optimism;
pub use routing_optimism::RO;

pub mod cost_estimate;