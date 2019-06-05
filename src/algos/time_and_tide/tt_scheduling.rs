use super::super::{FlowTable, GCL, Flow, StreamAwareGraph};
use crate::util::YensAlgo;

type FT = FlowTable<usize>;
type Yens<'a> = YensAlgo<'a, usize, StreamAwareGraph>;

const MTU: usize = 1000;
const PROCESS_TIME: f64 = 10.0;

/// 一個大小為 size 的資料流要切成幾個封包才夠？
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

use std::cmp::Ordering;
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

/// 動態計算 TT 資料流的 Gate Control List
/// * `og_table` - 本來的資料流表
/// * `changed_table` - 被改動到的那部份資料流，包含新增與換路徑
/// * `gcl` - 本來的 Gate Control List
/// * `yens` - Yen's algorithm 的物件，因為真正的路徑資訊記錄在這裡面
pub fn schedule_online(og_table: &FT,
    changed_table: &FT, gcl: &mut GCL, yens: &Yens
) -> Result<(), ()> {
    let result = schedule_fixed_og(changed_table, gcl, yens);
    if !result.is_ok() {
        gcl.clear();
        let union_table = og_table.union(true, changed_table);
        schedule_fixed_og(&union_table, gcl, yens)?;
    }
    Ok(())
}

/// 也可以當作離線排程算法來使用
pub fn schedule_fixed_og(changed_table: &FT,
    gcl: &mut GCL, yens: &Yens
) -> Result<(), ()> {
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
    for i in 0..links.len() {
        let trans_time = MTU as f64 / links[i].1;
        let arrive_time = {
            if i == 0 {
                if all_offsets.len() == 0 {
                    flow.offset() as f64
                } else {
                    // #m-1 封包完整送出
                    all_offsets[all_offsets.len()-1][i] + trans_time
                }
            } else {
                // #m 封包送達，且經過處理時間
                offsets[i-1] + MTU as f64 / links[i-1].1 + PROCESS_TIME
            }
        };
        let mut cur_offset = arrive_time;
        let mut time_shift = 0;
        loop { // 考慮 hyper period 中每種狀況
            /*
             * 1. 每個連結一個時間只能傳輸一個封包
             * 2. 同個佇列一個時間只能容納一個資料流（但可能容納該資料流的數個封包）
             * 3. 要符合 max_delay 的需求
             */
            // TODO 搞清楚第二點是為什麼？
            loop {
                let time_shift = time_shift as f64;
                // NOTE 確認沒有其它封包在這個連線上傳輸
                let option = gcl.get_next_empty_time(
                    links[i].0,
                    (time_shift + cur_offset) as usize,
                    (time_shift + cur_offset + trans_time) as usize
                );
                if let Some(time) = option {
                    cur_offset = time as f64 - time_shift;
                    check_deadline(cur_offset, trans_time, flow)?;
                    continue;
                }
                // NOTE 確認傳輸到下個地方時，下個連線的佇列是空的（沒有其它的資料流）
                if i < links.len() { // 還不到最後一個節點
                    let option = gcl.get_next_queue_empty_time(
                        links[i+1].0,
                        ro[i],
                        (time_shift + cur_offset + trans_time) as usize
                    );
                    if let Some(time) = option {
                        cur_offset = time as f64 - time_shift;
                        check_deadline(cur_offset, trans_time, flow)?;
                        continue;
                    }
                }
                // NOTE 檢查 arrive_time ~ cur_offset+trans_time 這段時間中有沒有發生同個佇列被佔用的事件
                // FIXME 應該提到最外面，用最後的 cur_offset 對 hyper period 中所有狀況做一次總檢查才對
                let can_occupy = gcl.check_can_occupy(
                    links[i+1].0,
                    ro[i],
                    (time_shift + arrive_time) as usize,
                    (time_shift + cur_offset + trans_time) as usize,
                );
                if !can_occupy {
                    // TODO 這裡真的應該直接回報無法排程嗎？
                    return Err(());
                }
                break;
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

#[inline(always)]
fn check_deadline(cur_offset: f64, trans_time: f64, flow: &Flow) -> Result<(), ()> {
    if cur_offset + trans_time >= flow.offset() as f64 + *flow.max_delay() as f64 {
        // 死線爆炸！
        Err(())
    } else {
        Ok(())
    }
}