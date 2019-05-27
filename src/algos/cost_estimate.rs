use super::{Flow, RouteTable, StreamAwareGraph, GCL};

/// 所有 TT 資料流對單個 AVB 資料流造成的干擾。
/// * `route` - 該 AVB 資料流的路徑
/// * `gcl` - 所有 TT 資料流的 Gate Control List
pub fn tt_interfere_on_avb(route: &Vec<usize>, gcl: &GCL, wcd: f64) {

}