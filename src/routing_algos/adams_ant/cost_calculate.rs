use super::super::time_and_tide::compute_avb_latency;
use super::{Flow, AdamsAnt};

const C1_EXCEED: f64 = 1000.0;
const W1: f64 = 1.0;
const W2: f64 = 1.0;
const W3: f64 = 1.0;

pub fn compute_avb_cost(algo: &AdamsAnt, flow: &Flow) -> f64 {
    let max_delay = *flow.max_delay();
    let route = algo.get_kth_route(flow, *algo.flow_table.get_info(*flow.id()));
    let latency = compute_avb_latency(
        &algo.g,
        flow,
        route,
        &algo.flow_table,
        &algo.gcl
    );
    let c1 = if latency > max_delay {
        C1_EXCEED
    } else {
        0.0
    };
    let c2 = latency as f64 / max_delay as f64;
    let c3 = 0.0; // TODO 計算 c3
    W1*c1 + W2*c2 + W3*c3
}

pub fn compute_all_avb_cost(algo: &AdamsAnt) -> f64 {
    let mut sum = 0.0;
    algo.flow_table.foreach(false, |flow, _| {
        sum += compute_avb_cost(algo, flow);
    });
    sum
}