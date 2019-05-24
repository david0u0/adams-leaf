extern crate adams_lib;

use adams_lib::network_struct::Graph;
use adams_lib::algos::{RO, RoutingAlgo, Flow, StreamAwareGraph, AVBType};

fn main() {
        let mut g = StreamAwareGraph::new();
        g.add_host();
        g.add_host();
        g.add_host();
        g.add_host();
        g.add_host();
        g.add_host();
        g.add_edge((0, 1), 100.0);
        g.add_edge((0, 2), 20.0/3.0);
        g.add_edge((0, 5), 20.0);
        g.add_edge((4, 5), 20.0);
        g.add_edge((0, 4), 10.0);
        g.add_edge((2, 4), 20.0);
        g.add_edge((2, 3), 20.0);
        g.add_edge((4, 3), 10.0);
        g.add_edge((1, 3), 100.0);
    
        let flow1 = Flow::TT {
            id: 0, src: 0, dst: 1, size: 100, period: 10, max_delay: 10, offset: 10
        };
        let flow2 = Flow::AVB {
            id: 0, src: 0, dst: 1, size: 100, period: 10,
            max_delay: 10, avb_type: AVBType::new_type_a()
        };

        let mut algo = RO::new(&g);
        algo.compute_routes(vec![flow1, flow2]);
}