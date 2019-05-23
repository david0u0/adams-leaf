extern crate adams_lib;

use adams_lib::network_struct::Graph;
use adams_lib::algos::{RO, RoutingAlgo, Flow, FlowStruct, StreamAwareGraph};


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
    
        g.inactivate_edge((0, 1));
        
        let flow = Flow::AVB(FlowStruct {
            id: 0, src: 0, dst: 1, size: 100, period: 10, max_delay: 10
        });
        let mut algo = RO::new(g);
        algo.compute_routes(vec![flow]);
        let v = algo.get_multi_routes(0, 1);
}