use adams_leaf::network_struct::Graph;
use adams_leaf::routing_algos::{RO, GCL, RoutingAlgo, StreamAwareGraph, AdamsAnt};
use adams_leaf::read_flows_from_file;


fn main() -> Result<(), String> {
    let mut g = StreamAwareGraph::new();
    // FIXME 對這個圖作 Yens algo，0->2這條路有時找得到6條，有時只找得到5條
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
    let mut gcl = GCL::new(100, g.get_edge_cnt());
    gcl.insert_gate_evt(9, 0, 0, 100); // 4 -> 2

    let mut algo = AdamsAnt::new(&g, None, None);
    //let mut algo = RO::new(&g, gcl);

    algo.add_flows(flows.clone());
    println!("{:?}", algo.get_route(0));
    println!("{:?}", algo.get_route(1));
    println!("{}", algo.compute_avb_cost(&flows[1], None));
    println!("{:?}", algo.get_route(2));
    println!("{}", algo.compute_avb_cost(&flows[2], None));
    println!("{:?}", algo.get_route(3));
    println!("{}", algo.compute_avb_cost(&flows[3], None));
    println!("{:?}", algo.get_route(4));
    println!("{}", algo.compute_avb_cost(&flows[4], None));
    println!("{:?}", algo.get_route(5));
    println!("{}", algo.compute_avb_cost(&flows[5], None));

    println!("sum = {}", algo.compute_all_avb_cost());

    return Ok(());
}