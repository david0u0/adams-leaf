mod network_struct;
mod algos;

use algos::{RO, Flow, FlowStruct, RoutingAlgo, Dijkstra};

fn main() {
    let mut g = network_struct::Graph::new();
    println!("{}", g.add_host());
    println!("{}", g.add_host());
    println!("{}", g.add_host());
    println!("=====");
    assert!(g.add_edge((1, 0), 20));
    assert!(g.add_edge((1, 2), 20));
    assert!(g.add_edge((0, 2), 10));

    let flow = Flow::AVB(FlowStruct {
        id: 0, src: 0, dst: 2, size: 100, period: 10, max_delay: 10
    });
    let mut algo = RO::new(g);
    algo.compute_routes(vec![flow]);
    let v = algo.get_route(0);
    println!("{:?}", v);
}