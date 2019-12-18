use super::super::{
    flow::{Flow, FlowID},
    AVBFlow, TSNFlow,
};
use std::rc::Rc;

enum FlowType {
    AVB,
    TSN,
}

pub struct FlowArena {
    avbs: Vec<Option<AVBFlow>>,
    tsns: Vec<Option<TSNFlow>>,
    type_pos_list: Vec<Option<(FlowType, usize)>>,
}
impl FlowArena {
    fn new() -> Self {
        FlowArena {
            avbs: vec![],
            tsns: vec![],
            type_pos_list: vec![],
        }
    }
    fn insert_avb(&mut self, mut flow: AVBFlow) -> FlowID {
        let id = FlowID(self.type_pos_list.len());
        let pos = self.avbs.len();
        flow.id = id;
        self.avbs.push(Some(flow));
        self.type_pos_list.push(Some((FlowType::AVB, pos)));
        id
    }
    fn insert_tsn(&mut self, mut flow: TSNFlow) -> FlowID {
        let id = FlowID(self.type_pos_list.len());
        let pos = self.tsns.len();
        flow.id = id;
        self.tsns.push(Some(flow));
        self.type_pos_list.push(Some((FlowType::TSN, pos)));
        id
    }
    fn get_avb(&self, id: FlowID) -> Option<&AVBFlow> {
        if id.0 < self.type_pos_list.len() {
            if let Some((FlowType::AVB, pos)) = self.type_pos_list[id.0] {
                return self.avbs[pos].as_ref();
            }
        }
        None
    }
    fn get_tsn(&self, id: FlowID) -> Option<&TSNFlow> {
        if id.0 < self.type_pos_list.len() {
            if let Some((FlowType::TSN, pos)) = self.type_pos_list[id.0] {
                return self.tsns[pos].as_ref();
            }
        }
        None
    }
}
/// 儲存的資料分為兩部份：資料流本身，以及隨附的資訊（T）。
///
/// __注意！這個資料結構 clone 的時候並不會把所有資料流複製一次，只會複製資訊的部份。__
///
/// 此處隱含的假設為：資料流本身不會時常變化，在演算法執行的過程中應該是唯一不變的，因此用一個 Rc 來記憶即可。
///
/// TODO 觀察在大資料量下這個改動是否有優化的效果。在小資料量下似乎沒啥差別。
#[derive(Clone)]
pub struct FlowTable<T: Clone> {
    arena: Rc<FlowArena>,
    changed: Option<Vec<FlowID>>,
    infos: Vec<Option<T>>,
}
impl<T: Clone> FlowTable<T> {
    pub fn new() -> Self {
        FlowTable {
            changed: None,
            infos: vec![],
            arena: Rc::new(FlowArena::new()),
        }
    }
    /// 建立一個新的資料流表。邏輯上，這個新資料流表為空，但可以執行 update_info。
    /// 遍歷新產生的表時，會自動跳過沒有修改過的資料流，且效能較高。
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
            arena: self.arena.clone(),
        }
    }
    pub fn check_flow_exist(&self, id: FlowID) -> bool {
        self.infos[id.0].is_some()
    }
    pub fn get_avb(&self, id: FlowID) -> Option<&AVBFlow> {
        if self.get_info(id).is_some() {
            self.arena.get_avb(id)
        } else {
            None
        }
    }
    pub fn get_tsn(&self, id: FlowID) -> Option<&TSNFlow> {
        if self.get_info(id).is_some() {
            self.arena.get_tsn(id)
        } else {
            None
        }
    }
    pub fn get_info(&self, id: FlowID) -> Option<&T> {
        if id.0 < self.infos.len() {
            self.infos[id.0].as_ref()
        } else {
            None
        }
    }
    pub fn delete_flow(&mut self, id: FlowID) {
        unimplemented!();
    }
    pub fn insert(&mut self, tsns: Vec<TSNFlow>, avbs: Vec<AVBFlow>, default_info: T) {
        if let Some(arena) = Rc::get_mut(&mut self.arena) {
            for flow in tsns.into_iter() {
                arena.insert_tsn(flow);
                self.infos.push(Some(default_info.clone()));
            }
            for flow in avbs.into_iter() {
                arena.insert_avb(flow);
                self.infos.push(Some(default_info.clone()));
            }
        } else {
            panic!("插入資料流時發生數據爭用");
        }
    }
    pub fn update_info(&mut self, id: FlowID, info: T) {
        self.infos[id.0] = Some(info);
        if let Some(changed) = &mut self.changed {
            changed.push(id);
        }
    }

    fn apply_callback<D: Clone + std::fmt::Debug>(
        &self,
        flow: Option<&Flow<D>>,
        mut callback: impl FnMut(&Flow<D>, &T),
    ) {
        if let Some(flow) = flow {
            if let Some(info) = self.get_info(flow.id) {
                callback(flow, info);
            }
        }
    }

    pub fn foreach_avb(&self, mut callback: impl FnMut(&AVBFlow, &T)) {
        if let Some(changed) = &self.changed {
            for &i in changed.iter() {
                self.apply_callback(self.get_avb(i), |flow, t| callback(flow, t));
            }
        } else {
            for i in 0..self.arena.avbs.len() {
                self.apply_callback(self.arena.avbs[i].as_ref(), |flow, t| callback(flow, t));
            }
        }
    }
    /// __慎用！__ 實現了內部可變性
    pub fn foreach_avb_mut(&self, mut callback: impl FnMut(&AVBFlow, &mut T)) {
        self.foreach_avb(|flow, t| unsafe {
            callback(flow, &mut *(t as *const T as *mut T));
        });
    }

    pub fn foreach_tsn(&self, mut callback: impl FnMut(&TSNFlow, &T)) {
        if let Some(changed) = &self.changed {
            for &i in changed.iter() {
                self.apply_callback(self.get_tsn(i), |flow, t| callback(flow, t));
            }
        } else {
            for i in 0..self.arena.tsns.len() {
                self.apply_callback(self.arena.tsns[i].as_ref(), |flow, t| callback(flow, t));
            }
        }
    }
    /// __慎用！__ 實現了內部可變性
    pub fn foreach_mut_tsn(&self, mut callback: impl FnMut(&TSNFlow, &mut T)) {
        self.foreach_tsn(|flow, t| unsafe {
            callback(flow, &mut *(t as *const T as *mut T));
        });
    }

    pub fn union(&mut self, is_avb: bool, other: &FlowTable<T>) {
        if !self.is_same_flow_list(other) {
            panic!("試圖合併不相干的 FlowTable");
        }
        if is_avb {
            other.foreach_avb(|flow, info| {
                self.update_info(flow.id, info.clone());
            });
        } else {
            other.foreach_tsn(|flow, info| {
                self.update_info(flow.id, info.clone());
            });
        }
    }
    pub fn is_same_flow_list(&self, other: &FlowTable<T>) -> bool {
        let a = &*self.arena as *const FlowArena;
        let b = &*other.arena as *const FlowArena;
        a == b
    }
    pub fn get_count(&self, is_avb: bool) -> usize {
        let mut cnt = 0;
        if is_avb {
            self.foreach_avb(|_, _| cnt += 1);
        } else {
            self.foreach_tsn(|_, _| cnt += 1);
        }
        cnt
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::read_flows_from_file;
    #[test]
    #[should_panic]
    fn datarace_should_panic() {
        let mut table = FlowTable::<usize>::new();
        let _table2 = table.clone();
        // drop(_table2);
        table.insert(vec![], vec![], 0);
    }
    #[test]
    fn no_datarace_no_panic() {
        let mut table = FlowTable::<usize>::new();
        let _table2 = table.clone();
        drop(_table2);
        table.insert(vec![], vec![], 0);
    }
    #[test]
    fn test_changed_flow_table() {
        let mut table = FlowTable::<usize>::new();
        let (tsns, avbs) = read_flows_from_file("test_flow.json", 1);
        table.insert(tsns, avbs, 0);
        assert_eq!(count_flows_inside(&table), 6);

        let mut changed = table.clone_into_changed_table();
        assert_eq!(count_flows_inside(&changed), 0);

        changed.update_info(2.into(), 99);
        assert_eq!(count_flows_inside(&changed), 1);

        changed.update_info(4.into(), 77);
        assert_eq!(count_flows_inside(&changed), 2);

        assert_eq!(changed.get_info(2.into()), Some(&99));
        assert_eq!(changed.get_info(4.into()), Some(&77));
        assert_eq!(table.get_info(2.into()), Some(&0));

        table.union(true, &changed);
        assert_eq!(table.get_info(2.into()), Some(&99));
        assert_eq!(table.get_info(4.into()), Some(&77));
        assert_eq!(count_flows_inside(&table), 6);
    }
    #[test]
    #[should_panic]
    fn union_different_flows_should_panic() {
        let mut table = FlowTable::<usize>::new();
        let (tsns, avbs) = read_flows_from_file("test_flow.json", 1);
        table.insert(tsns.clone(), avbs.clone(), 0);
        let table2 = FlowTable::<usize>::new();
        table.insert(tsns, avbs, 0);
        table.union(true, &table2);
    }
    fn count_flows_inside(table: &FlowTable<usize>) -> usize {
        table.get_count(true) + table.get_count(false)
    }
}
