use crate::flow::{AVBFlow, FlowID, TSNFlow};
use crate::network_wrapper::RoutingCost;

pub trait RoutingAlgo {
    fn add_flows(&mut self, tsns: Vec<TSNFlow>, avbs: Vec<AVBFlow>);
    fn del_flows(&mut self, tsns: Vec<TSNFlow>, avbs: Vec<AVBFlow>);
    fn get_rerouted_flows(&self) -> &Vec<FlowID>;
    fn get_route(&self, id: FlowID) -> &Vec<usize>;
    fn show_results(&self);
    fn get_last_compute_time(&self) -> u128;
    fn get_cost(&self) -> RoutingCost;
}

mod shortest_path;
pub use shortest_path::SPF;

mod routing_optimism;
pub use routing_optimism::RO;

mod adams_ant;
pub use adams_ant::AdamsAnt;
