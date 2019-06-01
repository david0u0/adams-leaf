use adams_leaf::network_struct::Graph;
use adams_leaf::algos::{RO, GCL, RoutingAlgo, StreamAwareGraph};
use adams_leaf::read_flows_from_file;


fn main() -> Result<(), String> {
    let mut g = StreamAwareGraph::new();
    g.add_host(Some(6));
    g.add_edge((0, 1), 100.0)?;
    //g.add_edge((0, 2), 100.0)?;
    g.add_edge((0, 5), 100.0)?;
    g.add_edge((4, 5), 100.0)?;
    g.add_edge((0, 4), 100.0)?;
    g.add_edge((2, 4), 100.0)?;
    g.add_edge((2, 3), 100.0)?;
    g.add_edge((4, 3), 100.0)?;
    g.add_edge((1, 3), 100.0)?;

    let flows = read_flows_from_file(0, "flows.json");
    let mut algo = RO::new(&g, 100, GCL::new(100, g.get_edge_cnt()));

    algo.compute_routes(flows);
    println!("{:?}", algo.get_route(1));
    println!("{:?}", algo.get_route(0));
    println!("{:?}", algo.get_route(2));
    println!("{:?}", algo.get_route(3));

    return Ok(());
}