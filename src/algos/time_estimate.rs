use super::{Flow, StreamAwareGraph, GCL, FlowTable};

/// AVB 資料流最多可以佔用的資源百分比
const MAX_AVB_SETTING: f64 = 0.75;
/// BE 資料流最多可以多大
const MAX_BE_SIZE: f64 = 1522.0;

/// 計算 AVB 資料流的端對端延遲（包含 TT、BE 及其它 AVB 所造成的延遲）
/// * `g` - 全局的網路拓撲
/// * `flow` - 該 AVB 資料流的詳細資訊
/// * `route` - 該 AVB 資料流的路徑
/// * `gcl` - 所有 TT 資料流的 Gate Control List
pub fn compute_avb_latency(g: &StreamAwareGraph, flow: &Flow, route: &Vec<usize>, flow_table: &FlowTable<usize>, gcl: &GCL) -> f64 {
    if let Flow::AVB { id, size, .. } = flow {
        let overlap_flow_id = g.get_overlap_flows(route);
        let mut end_to_end_lanency = 0.0;
        for (i, (link_id, bandwidth)) in g.get_edges_id_bandwidth(route).into_iter().enumerate() {
            let wcd = wcd_on_single_link(*id, *size,
                bandwidth, flow_table, &overlap_flow_id[i]);
            end_to_end_lanency += wcd + tt_interfere_avb_single_link(
                link_id, wcd as f64, gcl) as f64;
        }
        end_to_end_lanency
    } else {
        panic!("並非 AVB 資料流！");
    }
}
fn wcd_on_single_link(id: usize, size: u32, bandwidth: f64, flow_table: &FlowTable<usize>, overlap_flow_id: &Vec<usize>) -> f64 {
    let mut wcd = 0.0;
    // MAX None AVB
    wcd += MAX_BE_SIZE / bandwidth;
    // AVB 資料流最多只能佔用這樣的頻寬
    let bandwidth = MAX_AVB_SETTING * bandwidth;
    // On link
    wcd += size as f64 / bandwidth;
    // Ohter AVB
    for &flow_id in overlap_flow_id.iter() {
        if flow_id != id {
            if let Flow::AVB { size, .. } = flow_table.get_flow(flow_id) {
                wcd += *size as f64 / bandwidth;
            }
        }
    }
    wcd
}
fn tt_interfere_avb_single_link(link_id: usize, wcd: f64, gcl: &GCL) -> usize {
    let mut i_max = 0;
    let all_gce = gcl.get_close_event(link_id);
    for mut j in 0..all_gce.len() {
        let (mut i_cur, mut rem) = (0, wcd as i32);
        let gce_ptr = all_gce[j];
        while rem >= 0 {
            i_cur += gce_ptr.1;
            j += 1;
            if j == all_gce.len() {
                break;
            }
            let gce_ptr_next = all_gce[j];
            rem -= (gce_ptr_next.0 - (gce_ptr.0 + gce_ptr.1)) as i32;
        }
        i_max = std::cmp::max(i_max, i_cur);
    }
    return i_max;
}