use std::env;

use adams_leaf::routing_algos::{RO, RoutingAlgo, AdamsAnt};
use adams_leaf::{read_flows_from_file, read_topo_from_file};

fn main() -> Result<(), String> {
    let (topo_file_name, flow_file_name) = {
        let args: Vec<String> = env::args().collect();
        if args.len() == 3 {
            (args[1].clone(), args[2].clone())
        } else if args.len() == 1 {
            ("test_graph.json".to_owned(), "test_flows.json".to_owned())
        } else {
            println!("用法： adams_leaf [flow.json] [topo.json]");
            panic!();
        }
    };
    let flows = read_flows_from_file(0, &flow_file_name);
    let flows2 = read_flows_from_file(flows.len(), &flow_file_name);
    let g = read_topo_from_file(&topo_file_name);
    // FIXME 對這個圖作 Yens algo，0->2這條路有時找得到6條，有時只找得到5條

    let mut algo = AdamsAnt::new(&g, None, None);
    // let mut algo = RO::new(&g, None, None);

    algo.add_flows(flows.clone());
    //algo.add_flows(flows2.clone());

    algo.show_results();

    return Ok(());
}