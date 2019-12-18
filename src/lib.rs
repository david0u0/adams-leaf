use std::fs;
extern crate serde_json;
use serde::{Deserialize, Serialize};

pub mod graph_util;
pub mod routing_algos;
pub mod util;
pub const MAX_QUEUE: u8 = 8;
pub const MAX_K: usize = 20;
pub const T_LIMIT: u128 = 1000 * 1000; // micro_sec

pub const W1: f64 = 100.0;
pub const W2: f64 = 1.0;
pub const W3: f64 = 1.0;

pub const FAST_STOP: bool = true;

use routing_algos::{flow, AVBFlow, TSNFlow};

pub fn read_flows_from_file(file_name: &str, times: usize) -> (Vec<TSNFlow>, Vec<AVBFlow>) {
    let mut tsns = Vec::<TSNFlow>::new();
    let mut avbs = Vec::<AVBFlow>::new();
    for _ in 0..times {
        read_flows_from_file_once(&mut tsns, &mut avbs, file_name);
    }
    (tsns, avbs)
}
fn read_flows_from_file_once(tsns: &mut Vec<TSNFlow>, avbs: &mut Vec<AVBFlow>, file_name: &str) {
    let txt = fs::read_to_string(file_name).expect(&format!("找不到檔案: {}", file_name));
    let all_flows: AllFlows =
        serde_json::from_str(&txt).expect(&format!("無法解析檔案: {}", file_name));
    for cur_flow in all_flows.tt_flows.iter() {
        tsns.push(flow::Flow {
            id: 0.into(),
            size: cur_flow.size,
            src: cur_flow.src,
            dst: cur_flow.dst,
            period: cur_flow.period,
            max_delay: cur_flow.max_delay,
            spec_data: flow::TSNData {
                offset: cur_flow.offset,
            },
        });
    }
    for cur_flow in all_flows.avb_flows.iter() {
        avbs.push(flow::Flow {
            id: 0.into(),
            size: cur_flow.size,
            src: cur_flow.src,
            dst: cur_flow.dst,
            period: cur_flow.period,
            max_delay: cur_flow.max_delay,
            spec_data: flow::AVBData {
                avb_class: if cur_flow.avb_type == 'A' {
                    flow::AVBClass::A
                } else if cur_flow.avb_type == 'B' {
                    flow::AVBClass::A
                } else {
                    panic!("AVB type 必需為 `A` 或 `B`");
                },
            },
        });
    }
}

use graph_util::Graph;
pub fn read_topo_from_file(file_name: &str) -> graph_util::StreamAwareGraph {
    let txt = fs::read_to_string(file_name).expect(&format!("找不到檔案: {}", file_name));
    let json: GraphJSON =
        serde_json::from_str(&txt).expect(&format!("無法解析檔案: {}", file_name));
    let mut g = graph_util::StreamAwareGraph::new();
    g.add_host(Some(json.host_cnt));
    g.add_switch(Some(json.switch_cnt));
    for (n1, n2, bandwidth) in json.edges.into_iter() {
        g.add_edge((n1, n2), bandwidth).expect("插入邊失敗");
    }
    g
}

#[derive(Serialize, Deserialize)]
struct AllFlows {
    tt_flows: Vec<RawTSNFlow>,
    avb_flows: Vec<RawAVBFlow>,
}
#[derive(Serialize, Deserialize)]
struct RawTSNFlow {
    size: usize,
    src: usize,
    dst: usize,
    period: u32,
    max_delay: u32,
    offset: u32,
}
#[derive(Serialize, Deserialize)]
struct RawAVBFlow {
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
