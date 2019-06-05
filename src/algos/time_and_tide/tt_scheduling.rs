use super::super::{FlowTable, GCL, Flow, StreamAwareGraph};
use crate::util::YensAlgo;

type FT = FlowTable<usize>;
type Yens<'a> = YensAlgo<'a, usize, StreamAwareGraph>;

const MTU: usize = 1000;
const PROCESS_TIME: f64 = 10.0;

/// 一個大小為 size 的資料流要切成幾個訊框才夠？
#[inline(always)]
fn get_frame_cnt(size: usize) -> u8 {
    if size % MTU == 0 {
        (size / MTU) as u8
    } else {
        (size / MTU + 1) as u8
    }
}

fn get_route<'a> (flow: &'a Flow, table: &'a FT, yens: &'a Yens) -> &'a Vec<usize> {
    let k = *table.get_info(*flow.id());
    yens.get_kth_route(*flow.src(), *flow.dst(), k)
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
        let mut all_offsets: Vec<Vec<f64>> = vec![];
        let mut ro = vec![0; route.len()];
        let k = get_frame_cnt(*flow.size());
        let mut m = 0;
        while m < k {
            let res = calculate_offsets(flow, &all_offsets, &links, &ro, gcl);
            if let Ok(offsets) = res {
                m += 1;
                all_offsets.push(offsets);
            } else {
                m = 0;
                all_offsets.clear();
                let res = assign_new_queues();
                ro = {
                    if let Ok(new_ro) = res {
                        new_ro
                    } else {
                        return Err(());
                    }
                }
            }
            // TODO 把上面算好的結果塞進 GCL
        }
    }
    Ok(())
}

/// 回傳值為若為 Err，代表當前的佇列分配不足以完成排程
fn calculate_offsets(flow: &Flow, all_offsets: &Vec<Vec<f64>>,
    links: &Vec<(usize, f64)>, ro: &Vec<u8>, gcl: &GCL
) -> Result<Vec<f64>, ()> {
    let mut offsets = Vec::<f64>::with_capacity(links.len());
    let hyper_p = gcl.get_hyper_p();
    let flow_offset = {
        if let Flow::TT { offset, .. } = flow {
            *offset as f64
        } else {
            panic!("並非TT資料流！");
        }
    };
    for i in 0..links.len() {
        let trans_time = MTU as f64 / links[i].1;
        let mut cur_offset = {
            if i == 0 {
                if all_offsets.len() == 0 {
                    flow_offset
                } else {
                    // #m-1 封包完整送出
                    all_offsets[all_offsets.len()-1][i] + trans_time
                }
            } else {
                // #m 封包送達，且經過處理時間
                offsets[i-1] + MTU as f64 / links[i-1].1 + PROCESS_TIME
            }
        };
        let mut time_shift = 0;
        loop { // 考慮 hyper period 中每種狀況
            /*
             * 1. 每個連結一個時間只能傳輸一個封包
             * 2. 同個佇列一個時間只能容納一個資料流（但可能容納該資料流的數個封包）
             * 3. 要符合 max_delay 的需求
             */
            // TODO 搞清楚第二點是為什麼？
            loop {
                let mut ok = false;
                let time_shift = time_shift as f64;
                // TODO 確認沒有其它封包在這裡傳輸
                gcl.check_overlap(links[i].0, (time_shift + cur_offset) as usize,
                    (time_shift + cur_offset + trans_time) as usize);

                // TODO 確認傳輸到下個地方後，不會導致一個佇列裝兩個資料流

                if cur_offset >= flow_offset + *flow.max_delay() as f64 {
                    // 死線爆炸！
                    return Err(());
                } else if ok {
                    break;
                }
            }

            time_shift += *flow.period();
            if time_shift >= hyper_p {
                break;
            }
        }

        offsets.push(cur_offset);
    }
    Ok(offsets)
}

fn assign_new_queues() -> Result<Vec<u8>, ()> {
    Err(())
}