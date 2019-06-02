use std::rc::Rc;
use super::{Flow};
pub const MAX_FLOW_ID: usize = 9999;
/// 儲存的資料分為兩部份：資料流本身，以及隨附的資訊（T）。
/// 
/// __注意！這個資料結構 clone 的時候並不會把所有資料流複製一次，只會複製資訊的部份。__
/// 
/// 此處隱含的假設為：資料流本身不會時常變化，在演算法執行的過程中應該是唯一不變的，因此用一個 Rc 來記憶即可。
/// TODO 觀察在大資料量下這個改動是否有優化的效果。在小資料量下似乎沒啥差別。
#[derive(Clone)]
pub struct FlowTable<T: Clone> {
    flow_list: Rc<Vec<Option<Flow>>>,
    infos: Vec<Option<T>>
}
impl <T: Clone> FlowTable<T> {
    pub fn new() -> Self {
        return FlowTable {
            infos: vec![None; MAX_FLOW_ID],
            flow_list: Rc::new(vec![None; MAX_FLOW_ID])
        };
    }
    pub fn get_flow(&self, id: usize) -> &Flow {
        if let Some(t) = &self.flow_list[id] {
            return t;
        }
        panic!("該資料流不存在");
    }
    pub fn get_info(&self, id: usize) -> &T {
        if let Some(t) = &self.infos[id] {
            return t;
        }
        panic!("該資料流不存在");
    }
    pub fn delete_flow(&mut self, id: usize) {
        if let Some(_) = &self.infos[id] {
            //self.flow_list[id] = None;
        } else {
            panic!("該資料流不存在");
        }
        unimplemented!();
    }
    pub fn insert(&mut self, flows: Vec<Flow>, info: T) {
        let list = self.flow_list.clone();
        self.flow_list = Rc::new(vec![]);
        if let Ok(mut list) = Rc::try_unwrap(list) {
            for flow in flows.into_iter() {
                let id = *flow.id();
                list[id] = Some(flow);
                self.infos[id] = Some(info.clone());
            }
            self.flow_list = Rc::new(list);
        } else {
            panic!("插入資料流時發生 data race");
        }
    }
    pub fn update_info(&mut self, id: usize, info: T) {
        if let Some(entry) = &mut self.infos[id] {
            *entry = info;
        } else {
            panic!("更新路徑時發現資料流不存在");
        }
    }
    pub fn foreach(&self, is_avb: bool,
        mut callback: impl FnMut(&Flow, &T)
    ) {
        self.foreach_mut(is_avb, |flow, t| {
            callback(flow, t);
        });
    }
    /// __慎用！__ 實現了內部可變性
    pub fn foreach_mut(&self, is_avb: bool,
        mut callback: impl FnMut(&Flow, &mut T)
    ) {
        for (i, info) in self.infos.iter().enumerate() {
            if let Some(info) = &info {
                let flow = self.flow_list[i].as_ref().unwrap();
                let _info = info as *const T as *mut T;
                unsafe {
                    if let Flow::AVB { .. } = flow {
                        if is_avb {
                            callback(flow, &mut *_info);
                        }
                    } else if !is_avb {
                        callback(flow, &mut *_info);
                    }
                }
            } else {
                break;
                // FIXME 應記錄總共有多少資料流，而不是直接跳掉
            }
        }
    }
}

pub struct GCL {
    hyper_p: usize,
    // TODO 這個資料結構有優化的空間
    vec: Vec<Vec<(usize, usize)>>,
}
impl GCL {
    pub fn new(hyper_p: usize, edge_count: usize) -> Self {
        return GCL { vec: vec![vec![]; edge_count], hyper_p };
    }
    /// 回傳 `link_id` 上所有閘門關閉事件。
    /// * `回傳值` - 一個陣列，其內容為 (事件開始時間, 事件持續時間);
    pub fn get_close_event(&self, link_id: usize) -> &Vec<(usize, usize)> {
        assert!(self.vec.len() > link_id, "GCL: 指定了超出範圍的邊");
        return &self.vec[link_id];
    }
    pub fn insert_close_event(&mut self, link_id: usize, start_time: usize, duration: usize) {
        // FIXME: 應該做個二元搜索再插入
        return self.vec[link_id].push((start_time, duration));
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::read_flows_from_file;
    #[test]
    #[should_panic]
    fn flowtable_datarace_should_panic() {
        let mut table = FlowTable::<usize>::new();
        let _flow_list = table.flow_list.clone();
        let flows = read_flows_from_file(0, "flows.json");
        table.insert(flows, 0);
    }
}