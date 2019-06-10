use std::fs;
extern crate serde_json;
use serde::{Serialize, Deserialize};

pub mod network_struct;
pub mod algos;
pub mod util;
pub const MAX_QUEUE: u8 = 8;
pub const MAX_K: usize = 20;

use algos::{Flow, AVBType};

pub fn read_flows_from_file(base_id: usize, file_name: &str) -> Vec<Flow> {
    let mut flows = vec![];
    let txt = fs::read_to_string(file_name)
        .expect(&format!("無法讀取{}", file_name).as_str());
    let all_flows: AllFlows = serde_json::from_str(&txt)
        .expect(&format!("無法解析{}", file_name));
    for flow in all_flows.tt_flows.iter() {
        flows.push(Flow::TT {
            id: flows.len() + base_id,
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
    avb_type: char
}