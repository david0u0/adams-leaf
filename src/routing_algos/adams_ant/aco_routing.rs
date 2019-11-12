use std::time::Instant;

use super::{compute_all_avb_cost, AVBCostResult, AdamsAnt, FlowTable};
use crate::util::aco::{ShouldStopACO, ACO};
use crate::{MAX_K, W1, W2, FAST_STOP};

const TSN_MEMORY: f64 = 3.0; // 計算能見度時，TSN 對舊路徑的偏好程度
const AVB_MEMORY: f64 = 3.0; // 計算能見度時，AVB 對舊路徑的偏好程度

pub fn do_aco(algo: &mut AdamsAnt, time_limit: u128, reconf: FlowTable<usize>) {
    let time = Instant::now();
    let aco = &mut algo.aco as *mut ACO;
    algo.g.forget_all_flows();
    algo.flow_table.foreach(true, |flow, &route_k| unsafe {
        algo.save_flowid_on_edge(true, *flow.id(), route_k);
    });

    // 算好能見度再把新的 TT 排進去
    let vis = compute_visibility(algo, &reconf);
    // TT 排程
    let _self = algo as *const AdamsAnt;
    unsafe {
        (*_self)
            .schedule_online(&mut algo.gcl, &mut algo.flow_table, &reconf)
            .expect("TT走最短路徑無法排程");
    }

    let mut best_dist = std::f64::MAX;
    let new_state = unsafe {
        (*aco).do_aco(time_limit - time.elapsed().as_micros(), &vis, |state| {
            let res = compute_aco_dist(algo, state, &mut best_dist);
            if res.0 == 0 && FAST_STOP { // 找到可行解，且為快速終止模式
                ShouldStopACO::Yes(res.1)
            } else {
                ShouldStopACO::No(res.1)
            }
        })
    };
}

fn compute_visibility(algo: &AdamsAnt, reconf: &FlowTable<usize>) -> Vec<[f64; MAX_K]> {
    // TODO 好好設計能見度函式！
    // 目前：路徑長的倒數
    let len = algo.aco.get_state_len();
    let mut vis = vec![[0.0; MAX_K]; len];
    algo.flow_table.foreach(true, |flow, &route_k| {
        let id = *flow.id();
        for i in 0..algo.get_candidate_count(flow) {
            //vis[id][i] = 1.0 / algo.compute_avb_cost(flow, Some(i)).1.powf(2.0);
            vis[id][i] = 1.0 / algo.get_kth_route(id, route_k).len() as f64;
        }
        if !reconf.check_flow_exist(id) {
            // 是舊資料流，調高本來路徑的能見度
            vis[id][route_k] *= AVB_MEMORY;
        }
    });
    algo.flow_table.foreach(false, |flow, &route_k| {
        let id = *flow.id();
        for i in 0..algo.get_candidate_count(flow) {
            vis[id][i] = 1.0 / algo.get_kth_route(id, route_k).len() as f64;
        }
        if !reconf.check_flow_exist(id) {
            // 是舊資料流，調高本來路徑的能見度
            vis[id][route_k] *= TSN_MEMORY;
        }
    });
    vis
}

/// 本函式不只會計算距離，如果看見最佳解，還會把該解的 FlowTable 和 GCL 記錄下來
unsafe fn compute_aco_dist(
    algo: &mut AdamsAnt,
    state: &Vec<usize>,
    best_dist: &mut f64,
) -> AVBCostResult {
    let mut table = algo.flow_table.clone();
    let mut tt_changed_table = table.clone_into_changed_table();
    let mut gcl = algo.gcl.clone();
    // 第一輪：處理 TT 重排的問題
    for (id, &route_k) in state.iter().enumerate() {
        if table.check_flow_exist(id) {
            if table.get_flow(id).is_tt() {
                let old_route_k = *table.get_info(id);
                if old_route_k != route_k {
                    // 資料流存在，且在蟻群算法途中發生改變
                    let route = algo.get_kth_route(id, old_route_k);
                    let links = algo
                        .g
                        .get_links_id_bandwidth(route)
                        .iter()
                        .map(|(id, _)| *id)
                        .collect();
                    gcl.delete_flow(&links, id);
                    tt_changed_table.update_info(id, route_k);
                }
            }
        }
    }
    let result = algo.schedule_online(&mut gcl, &mut table, &tt_changed_table);
    if result.is_err() {
        return (std::u32::MAX, std::f64::MAX);
    }
    let cost1 = if result.unwrap() { 1.0 } else { 0.0 };

    // 第二輪：計算 AVB 的花費
    algo.g.forget_all_flows();
    for (id, &route_k) in state.iter().enumerate() {
        if table.check_flow_exist(id) {
            if table.get_flow(id).is_avb() {
                algo.save_flowid_on_edge(true, id, route_k);
                let old_route_k = *table.get_info(id);
                if old_route_k != route_k {
                    // 資料流存在，且在蟻群算法途中發生改變
                    // FIXME 下面兩行應該要有效，但事實上卻達不到預期的效果
                    //algo.save_flowid_on_edge(false, id, old_route_k);
                    //algo.save_flowid_on_edge(true, id, route_k);
                    // TODO 透過只計算受影響的資料流來加速
                    table.update_info(id, route_k);
                }
            }
        }
    }
    let avb_cost_res = compute_all_avb_cost(algo, &table, &gcl);
    let cost2 = avb_cost_res.1 / algo.avb_count as f64;

    let cost = W1 * cost1 + W2 * cost2;

    #[cfg(debug_assertions)]
    println!("{:?} {}", state, cost2 * algo.avb_count as f64);

    let base: f64 = 10.0;
    let dist = base.powf(cost - 1.0);

    if dist < *best_dist {
        *best_dist = dist;
        // 記錄 FlowTable 及 GCL
        algo.gcl = gcl;
        algo.flow_table = table;
    }
    avb_cost_res
}
