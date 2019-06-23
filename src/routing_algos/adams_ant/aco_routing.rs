use crate::MAX_K;
use crate::util::aco::ACO;
use super::{FlowTable, AdamsAnt, compute_avb_cost, compute_all_avb_cost, schedule_online};

const TT_AFFINITY: f64 = 100.0; // 計算能見度時，TT 對舊路徑的
const AVB_AFFINITY: f64 = 10.0; // 計算能見度時，AVB 對舊路徑的

const W1: f64 = 100.0;
const W2: f64 = 1.0;

pub fn do_aco(algo: &mut AdamsAnt, time_limit: u128, changed: FlowTable<usize>) {
    let aco = &mut algo.aco as *mut ACO;
    algo.g.forget_all_flows();
    algo.flow_table.foreach(true, |flow, &route_k| unsafe {
        algo.save_flowid_on_edge(true, *flow.id(), route_k);
    });

    let init_cost = algo.compute_all_avb_cost();
    let vis = compute_visibility(algo, changed);

    let new_state = unsafe {
        (*aco).do_aco(time_limit, &vis, |state| {
            compute_aco_dist(algo, state)
        }, init_cost)
    };
    if let Some(new_state) = new_state {
        for (id, &route_k) in new_state.iter().enumerate() {
            algo.flow_table.update_info(id, route_k);
        }
    }
}

fn compute_visibility(algo: &AdamsAnt, changed: FlowTable<usize>) -> Vec<[f64; MAX_K]> {
    // TODO 好好設計能見度函式！
    // 目前：AVB 為成本的倒數，且選中本來路徑的機率是改路徑機率的10倍
    //      TT 釘死最短路徑
    let len = algo.aco.get_state_len();
    let mut vis = vec![[0.0; MAX_K]; len];
    algo.flow_table.foreach(true, |flow, &route_k| {
        let id = *flow.id();
        for i in 0..algo.get_candidate_count(flow) {
            vis[id][i] = 1.0 / algo.compute_avb_cost(flow, Some(i));
        }
        if !changed.check_flow_exist(id) { // 是舊資料流，調高本來路徑的能見度
            vis[id][route_k] *= AVB_AFFINITY;
        }
    });
    algo.flow_table.foreach(false, |flow, &route_k| {
        let id = *flow.id();
        for i in 0..algo.get_candidate_count(flow) {
            vis[id][i] = 1.0 / algo.get_kth_route(id, route_k).len() as f64;
        }
        if !changed.check_flow_exist(id) { // 是舊資料流，調高本來路徑的能見度
            vis[id][route_k] *= TT_AFFINITY;
        }
    });
    vis
}

unsafe fn compute_aco_dist(algo: &mut AdamsAnt, state: &Vec<usize>) -> f64 {
    let mut table = algo.flow_table.clone();
    let mut tt_changed_table = table.clone_into_changed_table();
    let mut gcl = algo.gcl.clone();
    // 第一輪：處理 TT 重排的問題
    for (id, &route_k) in state.iter().enumerate() {
        if table.check_flow_exist(id) {
            let old_route_k = *table.get_info(id);
            if old_route_k != route_k {
                // 資料流存在，且在蟻群算法途中發生改變
                if table.get_flow(id).is_tt() {
                    let links = algo.get_kth_route(id, old_route_k);
                    gcl.delete_flow(links, id);
                    tt_changed_table.update_info(id, route_k);
                }
            }
        }
    }
    let result = algo.schedule_online(&mut gcl, &mut table, &tt_changed_table);
    if result.is_err() {
        return std::f64::MAX;
    }
    let cost1 = if result.unwrap() { 1.0 } else { 0.0 };

    // 第二輪：計算 AVB 的花費
    for (id, &route_k) in state.iter().enumerate() {
        if table.check_flow_exist(id) {
            let old_route_k = *table.get_info(id);
            if old_route_k != route_k {
                // 資料流存在，且在蟻群算法途中發生改變
                if table.get_flow(id).is_avb() {
                    algo.save_flowid_on_edge(false, id, old_route_k);
                    algo.save_flowid_on_edge(true, id, route_k);
                    // TODO 透過只計算受影響的資料流來加速
                    table.update_info(id, route_k);
                }
            }
        }
    }
    let cost2 = compute_all_avb_cost(algo, &table, &gcl) / algo.avb_count as f64;

    let cost = W1 * cost1 + W2 * cost2;

    #[cfg(not(release))]
    println!("{:?} {}", state, cost * algo.avb_count as f64);

    let base: f64 = 10.0;
    base.powf(cost - 1.0)
    //cost * cost
}