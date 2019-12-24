use adams_leaf::network_wrapper::RoutingCost;
use adams_leaf::routing_algos::{AdamsAnt, RoutingAlgo, RO, SPF};
use adams_leaf::{config, read_flows_from_file, read_topo_from_file};
use std::env;

fn main() -> Result<(), String> {
    let (algo_type, topo_file_name, flow_file_name, flow_file_name2, times) = {
        let args: Vec<String> = env::args().collect();
        if args.len() == 6 {
            (
                args[1].clone(),
                args[2].clone(),
                args[3].clone(),
                args[4].clone(),
                args[5].parse::<usize>().unwrap(),
            )
        } else {
            return Err("用法： adams_leaf [algo type] [topo.json] [base_flow.json] [reconf_flow.json] [倍數]".to_owned());
        }
    };
    let (tsns1, avbs1) = read_flows_from_file(&flow_file_name, 1);
    let (tsns2, avbs2) = read_flows_from_file(&flow_file_name2, times);
    let g = read_topo_from_file(&topo_file_name);
    // FIXME 對這個圖作 Yens algo，0->2這條路有時找得到6條，有時只找得到5條

    let mut cost_list = Vec::<RoutingCost>::new();
    for _ in 0..config::Config::get().exp_times {
        if algo_type == "aco" {
            println!("=== round 1 ===");
            let mut algo = AdamsAnt::new(g.clone());
            algo.add_flows(tsns1.clone(), avbs1.clone());
            algo.show_results();
            println!("=== round 2 ===");
            algo.add_flows(tsns2.clone(), avbs2.clone());
            algo.show_results();
            println!("compute time = {} micro sec", algo.get_last_compute_time());
            cost_list.push(algo.get_cost());
        } else if algo_type == "ro" {
            println!("=== round 1 ===");
            let mut algo = RO::new(g.clone());
            algo.add_flows(tsns1.clone(), avbs1.clone());
            algo.show_results();
            println!("=== round 2 ===");
            algo.add_flows(tsns2.clone(), avbs2.clone());
            algo.show_results();
            println!("compute time = {} micro sec", algo.get_last_compute_time());
            cost_list.push(algo.get_cost());
        } else if algo_type == "spf" {
            let mut algo = SPF::new(g.clone());
            algo.add_flows(tsns1.clone(), avbs1.clone());
            algo.add_flows(tsns2.clone(), avbs2.clone());
            algo.show_results();
            cost_list.push(algo.get_cost());
        }
    }
    RoutingCost::show_brief(cost_list);
    Ok(())
}
