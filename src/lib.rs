use std::fs;
extern crate serde_json;
use serde::{Deserialize, Serialize};

pub mod network_struct;
pub mod routing_algos;
pub mod util;
pub const MAX_QUEUE: u8 = 8;
pub const MAX_K: usize = 20;
pub const T_LIMIT: u128 = 1000 * 1000; // micro_sec

pub const W1: f64 = 100.0;
pub const W2: f64 = 1.0;
pub const W3: f64 = 1.0;

pub const FAST_STOP: bool = true;

use routing_algos::{AVBType, Flow};

pub fn read_flows_from_file(base_id: usize, file_name: &str, times: usize) -> Vec<Flow> {
    let mut flows = Vec::<Flow>::new();
    for _ in 0..times {
        read_flows_from_file_once(&mut flows, base_id, file_name);
    }
    flows
}
fn read_flows_from_file_once(flows: &mut Vec<Flow>, base_id: usize, file_name: &str) {
    let txt = fs::read_to_string(file_name).expect(&format!("找不到檔案: {}", file_name));
    let all_flows: AllFlows =
        serde_json::from_str(&txt).expect(&format!("無法解析檔案: {}", file_name));
    for flow in all_flows.tt_flows.iter() {
        flows.push(Flow::TT {
            id: flows.len() + base_id,
            size: flow.size,
            src: flow.src,
            dst: flow.dst,
            period: flow.period,
            max_delay: flow.max_delay,
            offset: flow.offset,
        });
    }
    for flow in all_flows.avb_flows.iter() {
        flows.push(Flow::AVB {
            id: flows.len() + base_id,
            size: flow.size,
            src: flow.src,
            dst: flow.dst,
            period: flow.period,
            max_delay: flow.max_delay,
            avb_type: {
                if flow.avb_type == 'A' {
                    AVBType::new_type_a()
                } else if flow.avb_type == 'B' {
                    AVBType::new_type_b()
                } else {
                    panic!("AVB type 必需為 `A` 或 `B`");
                }
            },
        });
    }
}

use network_struct::Graph;
pub fn read_topo_from_file(file_name: &str) -> network_struct::StreamAwareGraph {
    let txt = fs::read_to_string(file_name).expect(&format!("找不到檔案: {}", file_name));
    let json: GraphJSON =
        serde_json::from_str(&txt).expect(&format!("無法解析檔案: {}", file_name));
    let mut g = network_struct::StreamAwareGraph::new();
    g.add_host(Some(json.host_cnt));
    g.add_switch(Some(json.switch_cnt));
    for (n1, n2, bandwidth) in json.edges.into_iter() {
        g.add_edge((n1, n2), bandwidth);
    }
    g
}

#[derive(Serialize, Deserialize)]
struct AllFlows {
    tt_flows: Vec<TTFlow>,
    avb_flows: Vec<AVBFlow>,
}
#[derive(Serialize, Deserialize)]
struct TTFlow {
    size: usize,
    src: usize,
    dst: usize,
    period: u32,
    max_delay: u32,
    offset: u32,
}
#[derive(Serialize, Deserialize)]
struct AVBFlow {
    size: usize,
    src: usize,
    dst: usize,
    period: u32,
    max_delay: u32,
    avb_type: char,
}

#[derive(Serialize, Deserialize)]
struct GraphJSON {
    host_cnt: usize,
    switch_cnt: usize,
    edges: Vec<(usize, usize, f64)>,
}
