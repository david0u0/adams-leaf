use std::collections::HashMap;
use crate::network_struct::Graph;

macro_rules! flow_enum {
    ( $enum_name: ident, $( $name: ident {
        $( $field_name: ident: $field_type: ty ),*
    } ),* ) => {
        pub enum $enum_name {
            $(
                $name {
                    id: i32,
                    src: i32,
                    dst: i32,
                    size: u32,
                    period: u32,
                    max_delay: u32,
                    $( $field_name: $field_type ),*
                }
            ),*
        }
    };
}

pub struct AVBType(bool);
impl AVBType {
    pub fn new_type_a() -> Self {
        return AVBType(true);
    }
    pub fn new_type_b() -> Self {
        return AVBType(false);
    }
    pub fn is_type_a(&self) -> bool {
        return self.0;
    }
}

flow_enum!(Flow,
    TT {
        offset: u32
    },
    AVB {
        avb_type: AVBType
    }
);

pub trait RoutingAlgo {
    fn compute_routes(&mut self, flows: Vec<Flow>);
    fn get_retouted_flows(&self) -> Vec<i32>;
    fn get_route(&self, id: i32) -> Vec<i32>;
}

pub type RouteTable = HashMap<i32, (Flow, Option<(f64, Vec<i32>)>)>;

mod stream_aware_graph;
pub use stream_aware_graph::StreamAwareGraph;

mod shortest_path;
pub use shortest_path::SPF;

mod routing_optimism;
pub use routing_optimism::RO;

pub mod cost_estimate;