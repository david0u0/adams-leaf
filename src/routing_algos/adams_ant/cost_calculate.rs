use super::super::{flow_table_prelude::*, time_and_tide::compute_avb_latency, AVBFlow, GCL};
use super::{AdamsAnt, OldNew};

type FT<T> = FlowTable<T>;

/// - `0`: 判斷該解有多少條 AVB 不可排程。
/// - `1`: 即 worst case delay
/// - `3`: 即重排成本
pub type AVBCostResult = (u32, f64, f64);

/// 在特定 flow table、GCL 的狀況下，計算一個資料流如果走第 k 條路徑會是如何
/// 若 k 值為空，代表按照 flow table 中記錄的路徑。
/// - `algo` 傳入僅為了取路徑和取圖
pub(super) fn compute_avb_cost(
    algo: &AdamsAnt,
    flow: &AVBFlow,
    k: Option<usize>,
    table: &FT<usize>,
    gcl: &GCL,
    old_new_table: Option<&FT<OldNew>>,
) -> AVBCostResult {
    let k = match k {
        Some(t) => t,
        None => table.get_info(flow.id).unwrap().clone(),
    };
    let max_delay = flow.max_delay;
    let route = algo.get_kth_route(flow, k);
    let latency = compute_avb_latency(&algo.g, flow, route, table, gcl);
    let c1 = if latency > max_delay { 1 } else { 0 };
    let c2 = latency as f64 / max_delay as f64;
    let c3 = if is_old_route(flow, k, old_new_table) {
        0.0
    } else {
        1.0
    };
    (c1, c2, c3)
}

pub(super) fn compute_all_avb_cost(
    algo: &AdamsAnt,
    table: &FT<usize>,
    gcl: &GCL,
    old_new_table: Option<&FT<OldNew>>,
) -> AVBCostResult {
    let mut all_fail_cnt = 0;
    let mut all_wcd_cost = 0.0;
    let mut all_reroute_cost = 0.0;
    for (flow, &k) in table.iter_avb() {
        let (fail_cnt, wcd_cost, reroute_cost) =
            compute_avb_cost(algo, flow, Some(k), table, gcl, old_new_table);
        all_fail_cnt += fail_cnt;
        all_wcd_cost += wcd_cost;
        all_reroute_cost += reroute_cost;
    }
    (all_fail_cnt, all_wcd_cost, all_reroute_cost)
}

fn is_old_route(flow: &AVBFlow, route: usize, old_new_table: Option<&FT<OldNew>>) -> bool {
    if let Some(old_new_table) = old_new_table {
        if let OldNew::Old(old_route) = old_new_table.get_info(flow.id).unwrap() {
            return *old_route == route;
        }
    }
    true
}
