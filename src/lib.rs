use std::fs;
extern crate serde_json;
use serde_json::Value;
use serde::{Serialize, Deserialize};

pub mod network_struct;
pub mod algos;
pub mod util;

use algos::{Flow, AVBType};
const FLOW_FILE: &str = "flows.json";

pub fn read_flows_from_file() -> Vec<Flow> {
    let mut flows = vec![];
    let txt = fs::read_to_string(FLOW_FILE)
        .expect(&format!("無法讀取{}", FLOW_FILE).as_str());
    let all_flows: AllFlows = serde_json::from_str(&txt)
        .expect(&format!("無法解析{}", FLOW_FILE));
    for flow in all_flows.tt_flows.iter() {
        flows.push(Flow::TT {
            id: flows.len(),
            size: flow.size,
            src: flow.src,
            dst: flow.dst,
            period: flow.period,
            max_delay: flow.max_delay,
            offset: flow.offset
        });
    }
    for flow in all_flows.avb_flows.iter() {
        flows.push(Flow::AVB {
            id: flows.len(),
            size: flow.size,
            src: flow.src,
            dst: flow.dst,
            period: flow.period,
            max_delay: flow.max_delay,
            avb_type: {
                if flow.avb_type == 'A' {
                    AVBType::new_type_a()
                } else {
                    AVBType::new_type_b()
                }
            }
        });
    }
    flows
}

#[derive(Serialize, Deserialize)]
struct AllFlows {
    tt_flows: Vec<TTFlow>,
    avb_flows: Vec<AVBFlow>,
}
#[derive(Serialize, Deserialize)]
struct TTFlow {
    size: u32,
    src: usize,
    dst: usize,
    period: u32,
    max_delay: f64,
    offset: u32
}
#[derive(Serialize, Deserialize)]
struct AVBFlow {
    size: u32,
    src: usize,
    dst: usize,
    period: u32,
    max_delay: f64,
    avb_type: char
}