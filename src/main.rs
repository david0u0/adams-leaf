use std::env;

use adams_leaf::network_struct::Graph;
use adams_leaf::routing_algos::{RO, GCL, RoutingAlgo, AdamsAnt};
use adams_leaf::{read_flows_from_file, read_topo_from_file};

fn main() -> Result<(), String> {
    let flow_file_name = {
        let args: Vec<String> = env::args().collect();
        if args.len() > 1 {
            args[args.len()-1].clone()
        } else {
            "test_flows.json".to_owned()
        }
    };
    let mut g = read_topo_from_file("test_graph.json");
    // FIXME 對這個圖作 Yens algo，0->2這條路有時找得到6條，有時只找得到5條

    let flows = read_flows_from_file(0, &flow_file_name);
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

    /*println!("{:?}", algo.get_kth_route(1, 0));
    println!("{:?}", algo.get_kth_route(1, 1));
    println!("{:?}", algo.get_kth_route(1, 2));
    println!("{:?}", algo.get_kth_route(1, 3));
    println!("{:?}", algo.get_kth_route(1, 4));*/

    println!("sum = {}", algo.compute_all_avb_cost());

    return Ok(());
}