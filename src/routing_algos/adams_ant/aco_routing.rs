use super::{compute_all_avb_cost, compute_avb_cost, AVBCostResult, AdamsAnt, OldNew};
use crate::flow::FlowID;
use crate::recorder::flow_table::prelude::*;
use crate::util::aco::{ACOJudgeResult, ACO};
use crate::{FAST_STOP, MAX_K, W0, W1, W2, W3};
use std::time::Instant;

const TSN_MEMORY: f64 = 3.0; // 計算能見度時，TSN 對舊路徑的偏好程度
const AVB_MEMORY: f64 = 3.0; // 計算能見度時，AVB 對舊路徑的偏好程度

pub fn do_aco(algo: &mut AdamsAnt, time_limit: u128, reconf: DiffFlowTable<usize>) {
    let time = Instant::now();
    let aco = &mut algo.aco as *mut ACO;
    algo.g.forget_all_flows();
    let old_new_table = algo.flow_table.map_as(|id, &t| {
        if reconf.check_exist(id) {
            OldNew::New
        } else {
            OldNew::Old(t)
        }
    });

    for (flow, &route_k) in algo.flow_table.iter_avb() {
        unsafe {
            algo.update_flowid_on_route(true, flow, route_k);
        }
    }

    // 算好能見度再把新的 TT 排進去
    let vis = compute_visibility(algo, &old_new_table);
    // TT 排程
    let _self = algo as *const AdamsAnt;
    unsafe {
        (*_self)
            .schedule_online(&mut algo.gcl, &mut algo.flow_table, &reconf)
            .expect("TT走最短路徑無法排程");
        // TODO: 好好處理這個錯誤
    }

    let mut best_dist = std::f64::MAX;
    let new_state = unsafe {
        (*aco).do_aco(time_limit - time.elapsed().as_micros(), &vis, |state| {
            let res = compute_aco_dist(algo, state, &mut best_dist, &old_new_table);
            if res.0 == 0 && FAST_STOP {
                // 找到可行解，且為快速終止模式
                ACOJudgeResult::Stop(res.1)
            } else {
                ACOJudgeResult::KeepOn(res.1)
            }
        })
    };
}

fn compute_visibility(algo: &AdamsAnt, old_new_table: &FlowTable<OldNew>) -> Vec<[f64; MAX_K]> {
    // TODO 好好設計能見度函式！
    // 目前：路徑長的倒數
    let len = algo.aco.get_state_len();
    let mut vis = vec![[0.0; MAX_K]; len];
    for (flow, _) in algo.flow_table.iter_avb() {
        let id = flow.id;
        for i in 0..algo.get_candidate_count(flow) {
            vis[id.0][i] =
                1.0 / compute_avb_cost(algo, flow, Some(i), &algo.flow_table, &algo.gcl, None).1;
        }
        if let Some(OldNew::Old(route_k)) = old_new_table.get_info(flow.id) {
            // 是舊資料流，調高本來路徑的能見度
            vis[id.0][*route_k] *= AVB_MEMORY;
        }
    }
    for (flow, _) in algo.flow_table.iter_tsn() {
        let id = flow.id;
        for i in 0..algo.get_candidate_count(flow) {
            vis[id.0][i] = 1.0 / algo.get_kth_route(flow, i).len() as f64;
        }
        if let Some(OldNew::Old(route_k)) = old_new_table.get_info(flow.id) {
            // 是舊資料流，調高本來路徑的能見度
            vis[id.0][*route_k] *= TSN_MEMORY;
        }
    }
    vis
}

/// 本函式不只會計算距離，如果看見最佳解，還會把該解的 FlowTable 和 GCL 記錄下來
unsafe fn compute_aco_dist(
    algo: &mut AdamsAnt,
    state: &Vec<usize>,
    best_dist: &mut f64,
    old_new_table: &FlowTable<OldNew>,
) -> AVBCostResult {
    let mut table = algo.flow_table.clone();
    let mut tt_changed_table = table.clone_as_diff();
    let mut gcl = algo.gcl.clone();
    // 第零步：處理 TT 重排的問題
    for (id, &route_k) in state.iter().enumerate() {
        let id: FlowID = id.into();
        if let Some(flow) = table.get_tsn(id) {
            let old_route_k = *table.get_info(id).unwrap();
            if old_route_k != route_k {
                // 資料流存在，且在蟻群算法途中發生改變
                let route = algo.get_kth_route(flow, old_route_k);
                // TODO: 從 GCL 拔除舊資料流的工作應可交給排程函式
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
    let result = algo.schedule_online(&mut gcl, &mut table, &tt_changed_table);
    if result.is_err() {
        return (std::u32::MAX, std::f64::MAX, std::f64::MAX);
    }
    let cost0 = if result.unwrap() { 1.0 } else { 0.0 };

    // 第一二三步：計算 AVB 的花費，與排不下的數量，與重排成本
    algo.g.forget_all_flows();
    for (id, &route_k) in state.iter().enumerate() {
        let id: FlowID = id.into();
        if let Some(flow) = table.get_avb(id) {
            algo.update_flowid_on_route(true, flow, route_k);
            let old_route_k = *table.get_info(id).unwrap();
            if old_route_k != route_k {
                // 資料流存在，且在蟻群算法途中發生改變
                // FIXME 下面兩行應該要有效，但事實上卻達不到預期的效果，導致必須在開頭全部忘掉
                //algo.update_flowid_on_route(false, id, old_route_k);
                //algo.update_flowid_on_route(true, id, route_k);
                // TODO 透過只計算受影響的資料流來加速
                table.update_info(id, route_k);
            }
        }
    }
    let (cost1, cost2, cost3) = compute_all_avb_cost(algo, &table, &gcl, Some(old_new_table));

    let cost = W0 * cost0
        + (W1 * cost1 as f64 + W2 * cost2 + W3 * cost3 as f64) / table.get_avb_cnt() as f64;

    let base: f64 = 10.0;
    let dist = base.powf(cost - 1.0);

    if dist < *best_dist {
        *best_dist = dist;
        // 記錄 FlowTable 及 GCL
        algo.gcl = gcl;
        algo.flow_table = table;
    }
    (cost1, cost2, cost3)
}
