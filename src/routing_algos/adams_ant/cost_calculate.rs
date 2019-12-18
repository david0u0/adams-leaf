use super::super::time_and_tide::compute_avb_latency;
use super::{AVBFlow, AdamsAnt, FlowTable, GCL};

type FT = FlowTable<usize>;

/// - `0`: 整數，判斷該解有多少條 AVB 不可排程。
/// - `1`: 浮點值，即成本。
pub type AVBCostResult = (u32, f64);

use crate::{W1, W2, W3};

/// 在特定 flow table、GCL 的狀況下，計算一個資料流如果走第 k 條路徑會是如何
/// 若 k 值為空，代表按照 flow table 中記錄的路徑。
pub fn compute_avb_cost(
    algo: &AdamsAnt,
    flow: &AVBFlow,
    k: Option<usize>,
    table: &FT,
    gcl: &GCL,
) -> AVBCostResult {
    let k = match k {
        Some(t) => t,
        None => table.get_info(flow.id).unwrap().clone(),
    };
    let max_delay = flow.max_delay;
    let route = algo.get_kth_route(flow, k);
    let latency = compute_avb_latency(&algo.g, flow, route, table, gcl);
    let c1 = if latency > max_delay { 1.0 } else { 0.0 };
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
    table.foreach_avb(|flow, &k| {
        let (fail_cnt, cost) = compute_avb_cost(algo, flow, Some(k), table, gcl);
        all_fail_cnt += fail_cnt;
        all_cost += cost;
    });
    (all_fail_cnt, all_cost)
}
