use crate::MAX_K;
use crate::util::aco::ACO;
use super::{Flow, AdamsAnt, compute_all_avb_cost, compute_avb_cost};

pub fn do_aco(algo: &mut AdamsAnt, time_limit: u128) {
    let aco = &mut algo.aco as *mut ACO;
    algo.g.forget_all_flows();
    algo.flow_table.foreach(true, |flow, &route_k| unsafe {
        algo.save_flowid_on_edge(true, *flow.id(), route_k);
    });
    let cur_cost = compute_all_avb_cost(algo, &algo.flow_table, &algo.gcl);

    let mut table = algo.flow_table.clone();
    let mut gcl = algo.gcl.clone();
    let vis = compute_visibility(algo);
    let new_state = unsafe {
        (*aco).do_aco(time_limit, &vis, |state| {
            algo.g.forget_all_flows();
            for (id, &route_k) in state.iter().enumerate() {
                if table.check_flow_exist(id) {
                    algo.save_flowid_on_edge(true, id, route_k);
                    let old_route_k = *table.get_info(id);
                    if old_route_k != route_k {
                        // 資料流存在，且在蟻群算法途中發生改變
                        // TODO 透過只計算差異的資料流來加速
                        table.update_info(id, route_k);
                    }
                }

            }
            // FIXME TT 要重排！
            let cost = compute_all_avb_cost(algo, &table, &gcl);
            //println!("{:?} {}", state, cost);
            cost
        }, cur_cost)
    };
    if let Some(new_state) = new_state {
        for (id, &route_k) in new_state.iter().enumerate() {
            algo.flow_table.update_info(id, route_k);
        }
    }
}

fn compute_visibility(algo: &AdamsAnt) -> Vec<[f64; MAX_K]> {
    // TODO 好好設計能見度函式！
    // 目前：AVB 選中本來路徑的機率是改路徑機率的10倍
    //      TT 只會選本來路徑
    let len = algo.aco.get_state_len();
    let mut vis = vec![[0.0; MAX_K]; len];
    algo.flow_table.foreach(true, |flow, &route_k| {
        let id = *flow.id();
        for i in 0..algo.get_candidate_count(flow)-1 {
            vis[id][i] = 1.0;
        }
        vis[id][route_k] = 10.0;
    });
    algo.flow_table.foreach(false, |flow, &route_k| {
        let id = *flow.id();
        vis[id][route_k] = 10.0;
    });
    vis
}