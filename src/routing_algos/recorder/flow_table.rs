use std::rc::Rc;
use super::super::Flow;
/// 儲存的資料分為兩部份：資料流本身，以及隨附的資訊（T）。
/// 
/// __注意！這個資料結構 clone 的時候並不會把所有資料流複製一次，只會複製資訊的部份。__
/// 
/// 此處隱含的假設為：資料流本身不會時常變化，在演算法執行的過程中應該是唯一不變的，因此用一個 Rc 來記憶即可。
/// 
/// TODO 觀察在大資料量下這個改動是否有優化的效果。在小資料量下似乎沒啥差別。
#[derive(Clone)]
pub struct FlowTable<T: Clone> {
    changed: Option<Vec<usize>>,
    flow_list: Rc<Vec<Option<Flow>>>,
    infos: Vec<Option<T>>
}
impl <T: Clone> FlowTable<T> {
    pub fn new() -> Self {
        FlowTable {
            changed: None,
            infos: vec![],
            flow_list: Rc::new(vec![])
        }
    }
    /// 建立一個新的資料流表。邏輯上，這個新資料流表為空，但可以執行 update_info。
    /// # 範例
    /// ```
    /// let mut table = FlowTable::<usize>::new();
    /// table.insert(vec![flow0, flow1], 0);
    /// // table 中有兩個資料流，隨附資訊皆為0
    /// let mut changed_table = table;
    /// // changed_table 中有零個資料流
    /// changed_table.update(1, 99);
    /// // changed_table 中有一個 id=1 的資料流，且隨附資訊為99
    /// changed_table.insert(vec![flow2], 0);
    /// // will panic!
    /// ```
    pub fn clone_into_changed_table(&self) -> Self {
        FlowTable {
            changed: Some(vec![]),
            infos: vec![None; self.infos.len()],
            flow_list: self.flow_list.clone()
        }
    }
    pub fn check_flow_exist(&self, id: usize) -> bool {
        self.infos[id].is_some()
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
            // self.flow_list[id] = None;
            self.infos[id] = None;
        } else {
            panic!("該資料流不存在");
        }
    }
    pub fn insert(&mut self, flows: Vec<Flow>, info: T) {
        let list = self.flow_list.clone();
        self.flow_list = Rc::new(vec![]);
        if let Ok(mut list) = Rc::try_unwrap(list) {
            for flow in flows.into_iter() {
                let id = *flow.id();
                if id >= list.len() {
                    for _ in 0..(list.len() - id + 1) {
                        list.push(None);
                        self.infos.push(None);
                    }
                }
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
            self.infos[id] = Some(info);
        }
        if let Some(changed) = &mut self.changed {
            changed.push(id);
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
        if let Some(changed) = &self.changed {
            for &i in changed.iter() {
                self.apply_callback(is_avb, i, |flow, t| {
                    callback(flow, t);
                });
            }
        } else {
            for i in 0..self.infos.len() {
                self.apply_callback(is_avb, i, |flow, t| {
                    callback(flow, t);
                });
            }
        }
    }
    #[inline(always)]
    fn apply_callback(&self, is_avb: bool, index: usize,
        mut callback: impl FnMut(&Flow, &mut T)
    ) {
        if let Some(info) = &self.infos[index] {
            let flow = self.flow_list[index].as_ref().unwrap();
            let _info = info as *const T as *mut T;
            unsafe {
                if flow.is_avb() {
                    if is_avb {
                        callback(flow, &mut *_info);
                    }
                } else if !is_avb {
                    callback(flow, &mut *_info);
                }
            }
        }
    }
    pub fn union(&mut self, is_avb: bool, other: &FlowTable<T>) {
        if !self.is_same_flow_list(other) {
            panic!("試圖合併不相干的 FlowTable");
        }
        other.foreach(is_avb, |flow, info| {
            let id = *flow.id();
            self.update_info(id, info.clone());
        });
    }
    pub fn is_same_flow_list(&self, other: &FlowTable<T>) -> bool {
        let a = &*self.flow_list as *const Vec<Option<Flow>>;
        let b = &*other.flow_list as *const Vec<Option<Flow>>;
        a == b
    }
}

#[cfg(test)]
mod test {
    use crate::read_flows_from_file;
    use super::*;
    #[test]
    #[should_panic]
    fn datarace_should_panic() {
        let mut table = FlowTable::<usize>::new();
        let _table2 = table.clone();
        // drop(_table2);
        table.insert(vec![], 0);
    }
    #[test]
    fn no_datarace_no_panic() {
        let mut table = FlowTable::<usize>::new();
        let _table2 = table.clone();
        drop(_table2);
        table.insert(vec![], 0);
    }
    #[test]
    fn test_changed_flow_table() {
        let mut table = FlowTable::<usize>::new();
        let flows = read_flows_from_file(0, "test_flows.json");
        table.insert(flows, 0);
        assert_eq!(count_flows_inside(&table), 6);

        let mut changed = table.clone_into_changed_table();
        assert_eq!(count_flows_inside(&changed), 0);

        changed.update_info(2, 99);
        assert_eq!(count_flows_inside(&changed), 1);

        changed.update_info(4, 77);
        assert_eq!(count_flows_inside(&changed), 2);

        assert_eq!(*changed.get_info(2), 99);
        assert_eq!(*changed.get_info(4), 77);
        assert_eq!(*table.get_info(2), 0);

        table.union(true, &changed);
        assert_eq!(*table.get_info(2), 99);
        assert_eq!(*table.get_info(4), 77);
        assert_eq!(count_flows_inside(&table), 6);
    }
    #[test]
    #[should_panic]
    fn union_different_flows_should_panic() {
        let mut table = FlowTable::<usize>::new();
        let flows = read_flows_from_file(0, "test_flows.json");
        table.insert(flows.clone(), 0);
        let mut table2 = FlowTable::<usize>::new();
        table2.insert(flows.clone(), 0);
        table.union(true, &table2);
    }
    fn count_flows_inside(table: &FlowTable<usize>) -> usize {
        let mut cnt = 0;
        table.foreach(true, |_, _| {
            cnt += 1;
        });
        table.foreach(false, |_, _| {
            cnt += 1;
        });
        cnt
    }
}