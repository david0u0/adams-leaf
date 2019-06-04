use super::super::{Flow, StreamAwareGraph, GCL, FlowTable, AVBType};

/// AVB 資料流最多可以佔用的資源百分比（模擬 Credit Base Shaper 的效果）
const MAX_AVB_SETTING: f64 = 0.75;
/// BE 資料流最多可以多大
const MAX_BE_SIZE: f64 = 1522.0;

/// 計算 AVB 資料流的端對端延遲（包含 TT、BE 及其它 AVB 所造成的延遲）
/// * `g` - 全局的網路拓撲
/// * `flow` - 該 AVB 資料流的詳細資訊
/// * `route` - 該 AVB 資料流的路徑
/// * `gcl` - 所有 TT 資料流的 Gate Control List
pub fn compute_avb_latency<T: Clone>(g: &StreamAwareGraph, flow: &Flow,
    route: &Vec<usize>, flow_table: &FlowTable<T>, gcl: &GCL
) -> f64 {
    if let Flow::AVB { id, size, avb_type, .. } = flow {
        let overlap_flow_id = g.get_overlap_flows(route);
        let mut end_to_end_lanency = 0.0;
        for (i, (link_id, bandwidth)) in g.get_edges_id_bandwidth(route).into_iter().enumerate() {
            let wcd = wcd_on_single_link(*id, *size, *avb_type,
                bandwidth, flow_table, &overlap_flow_id[i]);
            end_to_end_lanency += wcd + tt_interfere_avb_single_link(
                link_id, wcd as f64, gcl) as f64;
        }
        end_to_end_lanency
    } else {
        panic!("並非 AVB 資料流！");
    }
}
fn wcd_on_single_link<T: Clone>(id: usize, size: usize,
    self_type: AVBType, bandwidth: f64,
    flow_table: &FlowTable<T>, overlap_flow_id: &Vec<usize>
) -> f64 {
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
            if let Flow::AVB { size, avb_type, .. } = flow_table.get_flow(flow_id) {
                if self_type.is_type_b() || avb_type.is_type_a() {
                    wcd += *size as f64 / bandwidth;
                }
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
        while rem >= 0 {
            let gce_ptr = all_gce[j];
            i_cur += gce_ptr.1;
            j += 1;
            if j == all_gce.len() {
                // TODO 應該要循環？
                break;
            }
            let gce_ptr_next = all_gce[j];
            rem -= (gce_ptr_next.0 - (gce_ptr.0 + gce_ptr.1)) as i32;
        }
        i_max = std::cmp::max(i_max, i_cur);
    }
    return i_max;
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::*;
    use crate::network_struct::*;
    fn init_settings() -> (StreamAwareGraph, Vec<Flow>, FlowTable<usize>, GCL) {
        let mut g = StreamAwareGraph::new();
        g.add_host(Some(3));
        g.add_edge((0, 1), 100.0).unwrap();
        g.add_edge((1, 2), 100.0).unwrap();
        let flows = vec![
            Flow::AVB {
                id: 0, src: 0, dst: 2, size: 75,
                period: 10000, max_delay: 200.0,
                avb_type: AVBType::new_type_a()
            },
            Flow::AVB {
                id: 1, src: 0, dst: 2, size: 150,
                period: 10000, max_delay: 200.0,
                avb_type: AVBType::new_type_a()
            },
            Flow::AVB {
                id: 2, src: 0, dst: 2, size: 75,
                period: 10000, max_delay: 200.0,
                avb_type: AVBType::new_type_b()
            }
        ];
        let flow_table = FlowTable::new();
        let gcl = GCL::new(10, g.get_edge_cnt());
        (g, flows, flow_table, gcl)
    }
    #[test]
    fn test_single_link_avb() {
        let (_, flows, mut route_table, _) = init_settings();
        let type_a = AVBType::new_type_a();
        let type_b = AVBType::new_type_b();

        route_table.insert(flows, 0);

        assert_eq!(wcd_on_single_link(0, 75, type_a, 100.0, &route_table, &vec![0, 2]),
            (MAX_BE_SIZE/100.0 + 1.0));
        assert_eq!(wcd_on_single_link(0, 75, type_a, 100.0, &route_table, &vec![1, 0, 2]),
            (MAX_BE_SIZE/100.0 + 1.0 + 2.0));
        assert_eq!(wcd_on_single_link(1, 150, type_a, 100.0, &route_table, &vec![1, 0, 2]),
            (MAX_BE_SIZE/100.0 + 1.0 + 2.0));

        assert_eq!(wcd_on_single_link(2, 75, type_b, 100.0, &route_table, &vec![1, 0, 2]),
            (MAX_BE_SIZE/100.0 + 1.0 + 2.0 + 1.0));
    }
    #[test]
    fn test_endtoend_avb_without_gcl() {
        let (mut g, flows, mut flow_table, gcl) = init_settings();
        flow_table.insert(vec![flows[0].clone()], 0);
        g.save_flowid_on_edge(true, 0, &vec![0, 1, 2]);
        assert_eq!(compute_avb_latency(&g, &flows[0], &vec![0, 1, 2], &flow_table, &gcl),
            (MAX_BE_SIZE/100.0 + 1.0) * 2.0);

        flow_table.insert(vec![flows[1].clone()], 0);
        g.save_flowid_on_edge(true, 1, &vec![0, 1, 2]);
        assert_eq!(compute_avb_latency(&g, &flows[0], &vec![0, 1, 2], &flow_table, &gcl),
            (MAX_BE_SIZE/100.0 + 1.0 + 2.0) * 2.0);
    }
    #[test]
    fn test_endtoend_avb_with_gcl() { // 其實已經接近整合測試了 @@
        let (mut g, flows, mut flow_table, mut gcl) = init_settings();

        flow_table.insert(vec![flows[0].clone()], 0);
        g.save_flowid_on_edge(true, 0, &vec![0, 1, 2]);
        flow_table.insert(vec![flows[1].clone()], 0);
        g.save_flowid_on_edge(true, 1, &vec![0, 1, 2]);

        gcl.insert_close_event(0, 0, 10);
        assert_eq!(compute_avb_latency(&g, &flows[0], &vec![0, 1, 2], &flow_table, &gcl),
            (MAX_BE_SIZE/100.0 + 1.0 + 2.0) * 2.0 + 10.0);

        gcl.insert_close_event(0, 15, 5);
        assert_eq!(compute_avb_latency(&g, &flows[0], &vec![0, 1, 2], &flow_table, &gcl),
            (MAX_BE_SIZE/100.0 + 1.0 + 2.0) * 2.0 + 15.0);

        gcl.insert_close_event(2, 100, 100);
        // 雖然這個關閉事件跟前面兩個不可能同時發生，但為了計算快速，還是假裝全部都發生了
        assert_eq!(compute_avb_latency(&g, &flows[0], &vec![0, 1, 2], &flow_table, &gcl),
            (MAX_BE_SIZE/100.0 + 1.0 + 2.0) * 2.0 + 115.0);
        
        gcl.insert_close_event(0, 100, 100);
        // 這個事件與同個埠口上的前兩個事件不可能同時發生，選比較久的（即這個事件）
        assert_eq!(compute_avb_latency(&g, &flows[0], &vec![0, 1, 2], &flow_table, &gcl),
            (MAX_BE_SIZE/100.0 + 1.0 + 2.0) * 2.0 + 200.0);
        assert_eq!(compute_avb_latency(&g, &flows[1], &vec![0, 1, 2], &flow_table, &gcl),
            (MAX_BE_SIZE/100.0 + 2.0 + 1.0) * 2.0 + 200.0);
    }
}