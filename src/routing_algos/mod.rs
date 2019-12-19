pub mod flow;
pub use flow::{AVBFlow, FlowID, TSNFlow};

pub trait RoutingAlgo {
    fn add_flows(&mut self, tsns: Vec<TSNFlow>, avbs: Vec<AVBFlow>);
    fn del_flows(&mut self, tsns: Vec<TSNFlow>, avbs: Vec<AVBFlow>);
    fn get_rerouted_flows(&self) -> &Vec<FlowID>;
    fn get_route(&self, id: FlowID) -> &Vec<usize>;
    fn show_results(&self);
    fn get_last_compute_time(&self) -> u128;
}

mod shortest_path;
pub use shortest_path::SPF;

mod routing_optimism;
pub use routing_optimism::RO;

mod recorder;
pub use recorder::{flow_table_prelude, GCL};

pub(self) mod time_and_tide;

mod adams_ant;
pub use adams_ant::AdamsAnt;
