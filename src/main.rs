extern crate adams_leaf;

use adams_leaf::network_struct::Graph;
use adams_leaf::algos::{RO, GCL, RoutingAlgo, Flow, StreamAwareGraph, AVBType};

fn main() -> Result<(), String> {
    let mut g = StreamAwareGraph::new();
    g.add_host(Some(6));
    g.add_edge((0, 1), 100.0)?;
    g.add_edge((0, 2), 20.0/3.0)?;
    g.add_edge((0, 5), 20.0)?;
    g.add_edge((4, 5), 20.0)?;
    g.add_edge((0, 4), 10.0)?;
    g.add_edge((2, 4), 20.0)?;
    g.add_edge((2, 3), 20.0)?;
    g.add_edge((4, 3), 10.0)?;
    g.add_edge((1, 3), 100.0)?;

    let flow1 = Flow::TT {
        id: 0, src: 0, dst: 1, size: 100, period: 10, max_delay: 10, offset: 10
    };
    let flow2 = Flow::AVB {
        id: 1, src: 0, dst: 2, size: 100, period: 10,
        max_delay: 10, avb_type: AVBType::new_type_a()
    };
    let mut algo = RO::new(&g, 100, GCL::new(100, g.get_edge_cnt() * 2));
    algo.compute_routes(vec![flow1.clone(), flow2.clone()]);
    println!("{:?}", algo.get_route(1));
    println!("{:?}", algo.get_route(0));

    return Ok(());
}