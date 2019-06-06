use super::super::{FlowTable, GCL, Flow, StreamAwareGraph};
use crate::util::YensAlgo;
use crate::MAX_QUEUE;

type FT = FlowTable<usize>;
type Yens<'a> = YensAlgo<'a, usize, StreamAwareGraph>;

const MTU: usize = 1000;
const PROCESS_TIME: f64 = 10.0;

/// 一個大小為 size 的資料流要切成幾個封包才夠？
#[inline(always)]
fn get_frame_cnt(size: usize) -> usize {
    if size % MTU == 0 {
        size / MTU
    } else {
        size / MTU + 1
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
        // NOTE 一個資料流的每個封包，在單一埠口上必需採用同一個佇列
        let mut ro: Vec<u8> = vec![0; links.len()];
        let k = get_frame_cnt(*flow.size());
        let mut m = 0;
        while m < k {
            let offsets = calculate_offsets(flow, &all_offsets, &links, &ro, gcl);
            if offsets.len() == links.len() {
                m += 1;
                all_offsets.push(offsets);
            } else {
                m = 0;
                all_offsets.clear();
                assign_new_queues(&mut ro)?;
            }
        }
        // 把上面算好的結果塞進 GCL
        for i in 0..links.len() {
            let link_id = links[i].0;
            let queue_id = ro[i];
            let trans_time = ((MTU as f64) / links[i].1) as u32;
            gcl.set_queueid(queue_id, link_id, id);
            // 考慮 hyper period 中每個狀況
            let step = *flow.period() as usize;
            for time_shift in (0..gcl.get_hyper_p()).step_by(step) {
                for m in 0..k {
                    // insert gate evt
                    gcl.insert_gate_evt(
                        link_id,
                        queue_id,
                        time_shift + all_offsets[m][i] as u32, 
                        time_shift + trans_time);
                    // insert queue evt
                    let queue_evt_start = if i == 0 {
                        flow.offset()
                    } else {
                        let prev_trans_time = MTU as f64/ links[i-1].1;
                        (all_offsets[m][i-1] + prev_trans_time) as u32
                    };
                    let queue_evt_duration = queue_evt_start - all_offsets[m][i] as u32;
                    gcl.insert_queue_evt(
                        link_id,
                        queue_id,
                        time_shift + queue_evt_start,
                        queue_evt_duration
                    );
                }
            }
        }
    }
    Ok(())
}

/// 回傳值為為一個陣列，若其長度小於路徑長，代表排一排爆開
fn calculate_offsets(flow: &Flow, all_offsets: &Vec<Vec<f64>>,
    links: &Vec<(usize, f64)>, ro: &Vec<u8>, gcl: &GCL
) -> Vec<f64> {
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
            // QUESTION 搞清楚第二點是為什麼？
            loop {
                // NOTE 確認沒有其它封包在這個連線上傳輸
                let option = gcl.get_next_empty_time(
                    links[i].0,
                    time_shift + cur_offset as u32,
                    trans_time as u32
                );
                if let Some(time) = option {
                    cur_offset = (time - time_shift) as f64;
                    if miss_deadline(cur_offset, trans_time, flow) {
                        return offsets;
                    }
                    continue;
                }
                // NOTE 確認傳輸到下個地方時，下個連線的佇列是空的（沒有其它的資料流）
                if i < links.len() { // 還不到最後一個節點
                    let option = gcl.get_next_queue_empty_time(
                        links[i+1].0,
                        ro[i],
                        time_shift + (cur_offset + trans_time) as u32
                    );
                    if let Some(time) = option {
                        cur_offset = (time - time_shift) as f64;
                        if miss_deadline(cur_offset, trans_time, flow) {
                            return offsets;
                        }
                        continue;
                    }
                }
                break;
            }
            // QUESTION 是否要檢查 arrive_time ~ cur_offset+trans_time 這段時間中有沒有發生同個佇列被佔用的事件？
            time_shift += *flow.period();
            if time_shift >= hyper_p {
                break;
            }
        }
        offsets.push(cur_offset);
    }
    offsets
}

fn assign_new_queues(ro: &mut Vec<u8>) -> Result<(), ()> {
    // TODO 好好實作這個函式（目前一個資料流只安排個佇列，但在不同埠口上應該可以安排給不同佇列）
    if ro[0] == MAX_QUEUE-1 {
        Err(())
    } else {
        for i in 0..ro.len() {
            ro[i] += 1;
        }
        Ok(())
    }
}

#[inline(always)]
fn miss_deadline(cur_offset: f64, trans_time: f64, flow: &Flow) -> bool {
    if cur_offset + trans_time >= (flow.offset() + *flow.max_delay()) as f64 {
        // 死線爆炸！
        true
    } else {
        false
    }
}