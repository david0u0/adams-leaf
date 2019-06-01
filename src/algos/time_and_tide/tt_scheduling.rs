use super::super::{FlowTable, GCL};

type FT = FlowTable<usize>;

pub fn tt_scheduling_offline(flow_table: &FT, gcl: &mut GCL) {
    // TODO 離線跟線上應該是兩套算法
    let og_table = FlowTable::new();
    tt_scheduling_online(&og_table, flow_table, gcl);
}

/// 動態計算 TT 資料流的 Gate Control List
/// * `og_table` - 本來的資料流表
/// * `changed_table` - 被改動到的那部份資料流，包含新增與換路徑
/// * `gcl` - 本來的 Gate Control List
pub fn tt_scheduling_online(og_table: &FT, changed_table: &FT, gcl: &mut GCL) {
    unimplemented!();
}