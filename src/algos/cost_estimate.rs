use super::{Flow, RouteTable, StreamAwareGraph, GCL};
use crate::network_struct::Graph;

/// AVB 資料流最多可以佔用的資源百分比
const MAX_AVB_SETTING: f64 = 0.75;

/// 所有 TT 與 AVB 資料流對單個 AVB 資料流造成的干擾。
/// * `g` - 全局的網路拓撲
/// * `flow` - 該 AVB 資料流的詳細資訊
/// * `route` - 該 AVB 資料流的路徑
/// * `gcl` - 所有 TT 資料流的 Gate Control List
pub fn compute_avb_latency(g: &StreamAwareGraph, avb_id: usize, route_table: &RouteTable, gcl: &GCL) -> usize {
    if let Flow::AVB { .. } = route_table.get_flow(avb_id) {
        let mut end_to_end_lanency = 0;
        let links = g.get_edge_ids(route_table.get_route(avb_id));
        for link in links.into_iter() {
            let wcd = wcd_on_single_link(avb_id, route_table);
            end_to_end_lanency += wcd + tt_interfere_avb_single_link(link, wcd as i32, gcl);
        }
        return end_to_end_lanency;
    } else {
        panic!("並非 AVB 資料流！");
    }
}

fn wcd_on_single_link(avb_id: usize, route_table: &RouteTable) -> usize {
    if let Flow::AVB { .. } = route_table.get_flow(avb_id) {
        panic!("Not implemented");
    } else {
        panic!("並非 AVB 資料流！");
    }
}
fn tt_interfere_avb_single_link(link_id: usize, wcd: i32, gcl: &GCL) -> usize {
    let mut i_max = 0;
    let all_gce = gcl.get_close_event(link_id);
    for (mut j, _) in all_gce.iter().enumerate() {
        let (mut i_cur, mut rem) = (0, wcd);
        let gce_ptr = all_gce[j];
        while rem >= 0 {
            let gce_ptr_next = all_gce[j+1];
            i_cur += gce_ptr.1;
            rem -= (gce_ptr_next.0 - (gce_ptr.0 + gce_ptr.1)) as i32;
            j += 1;
        }
        i_max = std::cmp::max(i_max, i_cur);
    }
    return i_max;
}