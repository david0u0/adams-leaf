use super::super::time_and_tide::compute_avb_latency;
use super::{AdamsAnt, Flow, FlowTable, GCL};

type FT = FlowTable<usize>;

pub type AVBCostResult = (u32, f64);

const C1_EXCEED: f64 = 100.0;
const W1: f64 = 1.0;
const W2: f64 = 1.0;
const W3: f64 = 1.0;

pub fn compute_avb_cost(
    algo: &AdamsAnt,
    flow: &Flow,
    k: Option<usize>,
    table: &FT,
    gcl: &GCL,
) -> AVBCostResult {
    let k = match k {
        Some(t) => t,
        None => *table.get_info(*flow.id()),
    };
    let max_delay = *flow.max_delay();
    let route = algo.get_kth_route(*flow.id(), k);
    let latency = compute_avb_latency(&algo.g, flow, route, table, gcl);
    let c1 = if latency > max_delay { C1_EXCEED } else { 0.0 };
    let c2 = latency as f64 / max_delay as f64;
    let c3 = 0.0; // TODO 計算 c3
    if c1 > 0.1 {
        (1, W1 * c1 + W2 * c2 + W3 * c3)
    } else {
        (0, W1 * c1 + W2 * c2 + W3 * c3)
    }
}

pub fn compute_all_avb_cost(algo: &AdamsAnt, table: &FT, gcl: &GCL) -> AVBCostResult {
    let mut all_cost = 0.0;
    let mut all_fail_cnt = 0;
    table.foreach(true, |flow, &k| {
        let (fail_cnt, cost) = compute_avb_cost(algo, flow, Some(k), table, gcl);
        all_fail_cnt += fail_cnt;
        all_cost += cost;
    });
    (all_fail_cnt, all_cost)
}
