use super::super::time_and_tide::compute_avb_latency;
use super::{Flow, AdamsAnt, FlowTable, GCL};

type FT = FlowTable<usize>;

const C1_EXCEED: f64 = 1000.0;
const W1: f64 = 1.0;
const W2: f64 = 1.0;
const W3: f64 = 1.0;

pub fn compute_avb_cost(algo: &AdamsAnt, flow: &Flow, k: Option<usize>, table: &FT, gcl: &GCL) -> f64 {
    let k = match k {
        Some(t) => t,
        None => *table.get_info(*flow.id())
    };
    let max_delay = *flow.max_delay();
    let route = algo.get_kth_route(*flow.id(), *table.get_info(*flow.id()));
    let latency = compute_avb_latency(
        &algo.g,
        flow,
        route,
        &table,
        gcl
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

pub fn compute_all_avb_cost(algo: &AdamsAnt, table: &FT, gcl: &GCL) -> f64 {
    let mut sum = 0.0;
    table.foreach(true, |flow, &k| {
        sum += compute_avb_cost(algo, flow, Some(k), table, gcl);
    });
    sum
}

pub fn find_min_cost_route(algo: &AdamsAnt, flow: &Flow, table: &FT, gcl: &GCL) -> usize {
    let (mut min_cost, mut best_k) = (std::f64::MAX, 0);
    for k in 0..algo.get_candidate_count(flow) {
        let cost = compute_avb_cost(algo, flow, Some(k), table, gcl);
        if cost < min_cost {
            min_cost = cost;
            best_k = k;
        }
    }
    best_k
}