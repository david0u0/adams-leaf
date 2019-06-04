use super::super::{FlowTable, GCL, Flow, StreamAwareGraph};
use crate::util::YensAlgo;

type FT = FlowTable<usize>;
type Yens<'a> = YensAlgo<'a, usize, StreamAwareGraph>;

const MTU: usize = 1000;

pub fn tt_scheduling_offline(flow_table: &FT, gcl: &mut GCL, yens: &Yens) {
    // TODO 離線跟線上應該是兩套算法
    let og_table = FlowTable::new();
    tt_scheduling_online(&og_table, flow_table, gcl, yens);
}

/// 動態計算 TT 資料流的 Gate Control List
/// * `og_table` - 本來的資料流表
/// * `changed_table` - 被改動到的那部份資料流，包含新增與換路徑
/// * `gcl` - 本來的 Gate Control List
/// * `yens` - Yen's algorithm 的物件，因為真正的路徑資訊記錄在這裡面
pub fn tt_scheduling_online(og_table: &FT, changed_table: &FT, gcl: &mut GCL, yens: &Yens) {
    let result = tt_scheduling_fixed_og(changed_table, gcl, yens);
    if !result.is_ok() {
        gcl.clear();
        let union_table = og_table.union(true, changed_table);
        let result = tt_scheduling_fixed_og(&union_table, gcl, yens);
        if !result.is_ok() {
            panic!("GCL 怎麼排都排不下 GG");
        }
    }
}

use std::cmp::Ordering;
fn tt_scheduling_fixed_og(changed_table: &FT, gcl: &mut GCL, yens: &Yens) -> Result<(), ()> {
    let mut tt_flows = Vec::<usize>::new();
    let g = yens.get_graph();
    changed_table.foreach(false, |flow, _| {
        tt_flows.push(*flow.id());
    });
    tt_flows.sort_by(|&id1, &id2| {
        cmp_flow(id1, id2, changed_table, yens)
    });
    for id in tt_flows.into_iter() {
        let flow = changed_table.get_flow(id);
        let route = get_route(flow, changed_table, yens);
        let links = g.get_edges_id_bandwidth(route);
        let ro = vec![0; route.len()];
        let mut m = 0;
        let k = get_frame_cnt(*flow.size());
        // TODO 計算GCL
    }

    unimplemented!();
}

/// 排序的標準：
/// * `deadline` - 時間較緊的要排前面
/// * `period` - 週期短的要排前面
/// * `route length` - 路徑長的要排前面
fn cmp_flow(id1: usize, id2: usize, table: &FT, yens: &Yens) -> Ordering {
    let flow1 = table.get_flow(id1);
    let flow2 = table.get_flow(id2);
    if flow1.max_delay() < flow2.max_delay() {
        Ordering::Less
    } else if flow1.max_delay() > flow2.max_delay() {
        Ordering::Greater
    } else {
        if flow1.period() < flow2.period() {
            Ordering::Less
        } else if flow1.period() > flow2.period() {
            Ordering::Greater
        } else {
            let rlen_1 = get_route(flow1, table, yens).len();
            let rlen_2 = get_route(flow2, table, yens).len();
            if rlen_1 > rlen_2 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
    }
}

fn get_route<'a> (flow: &'a Flow, table: &'a FT, yens: &'a Yens) -> &'a Vec<usize> {
    let k = *table.get_info(*flow.id());
    yens.get_kth_route(*flow.src(), *flow.dst(), k)
}

/// 一個大小為 size 的資料流要切成幾個訊框才夠？
#[inline(always)]
fn get_frame_cnt(size: usize) -> usize {
    if size % MTU == 0 {
        size / MTU
    } else {
        size / MTU + 1
    }
}