use super::super::{
    flow::{Flow, FlowID},
    AVBFlow, TSNFlow,
};
use std::rc::Rc;

#[derive(Clone, Copy)]
enum FlowType {
    AVB,
    TSN,
}
pub struct FlowArena {
    avbs: Vec<Option<AVBFlow>>,
    tsns: Vec<Option<TSNFlow>>,
    type_pos_list: Vec<Option<(FlowType, usize)>>,
    max_id: FlowID,
}
impl FlowArena {
    fn new() -> Self {
        FlowArena {
            avbs: vec![],
            tsns: vec![],
            type_pos_list: vec![],
            max_id: 0.into(),
        }
    }
    fn get_flow_type(&self, id: FlowID) -> FlowType {
        self.type_pos_list[id.0].unwrap().0
    }
    fn insert_avb(&mut self, mut flow: AVBFlow) -> FlowID {
        let id = FlowID(self.type_pos_list.len());
        self.max_id = std::cmp::max(self.max_id, id);
        let pos = self.avbs.len();
        flow.id = id;
        self.avbs.push(Some(flow));
        self.type_pos_list.push(Some((FlowType::AVB, pos)));
        id
    }
    fn insert_tsn(&mut self, mut flow: TSNFlow) -> FlowID {
        let id = FlowID(self.type_pos_list.len());
        self.max_id = std::cmp::max(self.max_id, id);
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
fn apply_callback<D: Clone, T: Clone, FT: IFlowTable<INFO = T>>(
    table: &FT,
    flow: Option<&Flow<D>>,
    mut callback: impl FnMut(&Flow<D>, &T),
) {
    if let Some(flow) = flow {
        if let Some(info) = table.get_info(flow.id) {
            callback(flow, info);
        }
    }
}

pub trait IFlowTable {
    type INFO: Clone;
    fn get_inner_arena(&self) -> &Rc<FlowArena>;
    fn get_info(&self, id: FlowID) -> Option<&Self::INFO>;
    fn update_info(&mut self, id: FlowID, info: Self::INFO);
    fn check_exist(&self, id: FlowID) -> bool {
        self.get_info(id).is_some()
    }
    fn get_avb(&self, id: FlowID) -> Option<&AVBFlow> {
        if self.check_exist(id) {
            self.get_inner_arena().get_avb(id)
        } else {
            None
        }
    }
    fn get_tsn(&self, id: FlowID) -> Option<&TSNFlow> {
        if self.check_exist(id) {
            self.get_inner_arena().get_tsn(id)
        } else {
            None
        }
    }
    fn is_same_flow_list<T: IFlowTable<INFO = Self::INFO>>(&self, other: &T) -> bool {
        let a = &**self.get_inner_arena() as *const FlowArena;
        let b = &**other.get_inner_arena() as *const FlowArena;
        a == b
    }
    fn foreach_avb(&self, callback: impl FnMut(&AVBFlow, &Self::INFO));
    fn foreach_tsn(&self, callback: impl FnMut(&TSNFlow, &Self::INFO));
    /// __慎用！__ 實現了內部可變性
    fn foreach_tsn_mut(&self, mut callback: impl FnMut(&TSNFlow, &mut Self::INFO)) {
        self.foreach_tsn(|flow, t| unsafe {
            callback(flow, &mut *(t as *const Self::INFO as *mut Self::INFO));
        });
    }
    /// __慎用！__ 實現了內部可變性
    fn foreach_avb_mut(&self, mut callback: impl FnMut(&AVBFlow, &mut Self::INFO)) {
        self.foreach_avb(|flow, t| unsafe {
            callback(flow, &mut *(t as *const Self::INFO as *mut Self::INFO));
        });
    }
    /// 建立一個新的資料流表。邏輯上，這個新資料流表為空，但可以執行 update_info。
    /// 遍歷新產生的表時，會自動跳過沒有修改過的資料流，且效能較高。
    /// # 範例
    /// ```
    /// let mut table = FlowTable::<usize>::new();
    /// table.insert(vec![flow0, flow1], 0);
    /// // table 中有兩個資料流，隨附資訊皆為0
    /// let mut changed_table = table.clone_as_diff();
    /// // changed_table 中有零個資料流
    /// changed_table.update(1, 99);
    /// // changed_table 中有一個 id=1 的資料流，且隨附資訊為99
    /// changed_table.insert(vec![flow2], 0);
    /// // will panic!
    /// ```
    fn clone_as_diff(&self) -> DiffFlowTable<Self::INFO>;
    fn get_avb_cnt(&self) -> usize;
    fn get_tsn_cnt(&self) -> usize;
    fn get_flow_cnt(&self) -> usize {
        self.get_tsn_cnt() + self.get_avb_cnt()
    }
    fn get_max_id(&self) -> FlowID {
        self.get_inner_arena().max_id
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
    avb_cnt: usize,
    tsn_cnt: usize,
}
impl<T: Clone> FlowTable<T> {
    pub fn new() -> Self {
        FlowTable {
            changed: None,
            infos: vec![],
            arena: Rc::new(FlowArena::new()),
            avb_cnt: 0,
            tsn_cnt: 0,
        }
    }
    pub fn apply_diff(&mut self, is_avb: bool, other: &DiffFlowTable<T>) {
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
    pub fn insert<'a>(
        &'a mut self,
        tsns: Vec<TSNFlow>,
        avbs: Vec<AVBFlow>,
        default_info: T,
    ) -> Vec<FlowID> {
        if let Some(arena) = Rc::get_mut(&mut self.arena) {
            let mut id_list = vec![];
            for flow in tsns.into_iter() {
                let id = arena.insert_tsn(flow);
                self.infos.push(Some(default_info.clone()));
                id_list.push(id);
                self.tsn_cnt += 1;
            }
            for flow in avbs.into_iter() {
                let id = arena.insert_avb(flow);
                self.infos.push(Some(default_info.clone()));
                id_list.push(id);
                self.avb_cnt += 1;
            }
            id_list
        } else {
            panic!("插入資料流時發生數據爭用");
        }
    }
}
impl<T: Clone> IFlowTable for FlowTable<T> {
    type INFO = T;
    fn get_avb_cnt(&self) -> usize {
        self.avb_cnt
    }
    fn get_tsn_cnt(&self) -> usize {
        self.tsn_cnt
    }
    fn get_inner_arena(&self) -> &Rc<FlowArena> {
        &self.arena
    }
    fn get_info(&self, id: FlowID) -> Option<&T> {
        if id.0 < self.infos.len() {
            self.infos[id.0].as_ref()
        } else {
            None
        }
    }
    fn update_info(&mut self, id: FlowID, info: T) {
        if id.0 < self.infos.len() {
            self.infos[id.0] = Some(info);
            if let Some(changed) = &mut self.changed {
                changed.push(id);
            }
        } else {
            panic!("更新資訊時發生越界");
        }
    }

    fn foreach_avb(&self, mut callback: impl FnMut(&AVBFlow, &T)) {
        if let Some(changed) = &self.changed {
            for &i in changed.iter() {
                apply_callback(self, self.get_avb(i), |flow, t| callback(flow, t));
            }
        } else {
            for i in 0..self.arena.avbs.len() {
                apply_callback(self, self.arena.avbs[i].as_ref(), |flow, t| {
                    callback(flow, t)
                });
            }
        }
    }

    fn foreach_tsn(&self, mut callback: impl FnMut(&TSNFlow, &T)) {
        if let Some(changed) = &self.changed {
            for &i in changed.iter() {
                apply_callback(self, self.get_tsn(i), |flow, t| callback(flow, t));
            }
        } else {
            for i in 0..self.arena.tsns.len() {
                apply_callback(self, self.arena.tsns[i].as_ref(), |flow, t| {
                    callback(flow, t)
                });
            }
        }
    }
    fn clone_as_diff(&self) -> DiffFlowTable<T> {
        DiffFlowTable::new(self)
    }
}

#[derive(Clone)]
pub struct DiffFlowTable<T: Clone> {
    avb_diff: Vec<FlowID>,
    tsn_diff: Vec<FlowID>,
    table: FlowTable<T>,
}
impl<T: Clone> DiffFlowTable<T> {
    pub fn new(og_table: &FlowTable<T>) -> Self {
        DiffFlowTable {
            avb_diff: vec![],
            tsn_diff: vec![],
            table: FlowTable {
                arena: og_table.arena.clone(),
                changed: None,
                infos: vec![None; og_table.infos.len()],
                avb_cnt: 0,
                tsn_cnt: 0,
            },
        }
    }
}
impl<T: Clone> IFlowTable for DiffFlowTable<T> {
    type INFO = T;
    fn get_avb_cnt(&self) -> usize {
        self.avb_diff.len()
    }
    fn get_tsn_cnt(&self) -> usize {
        self.tsn_diff.len()
    }
    fn get_inner_arena(&self) -> &Rc<FlowArena> {
        &self.table.get_inner_arena()
    }
    fn get_info(&self, id: FlowID) -> Option<&Self::INFO> {
        self.table.get_info(id)
    }
    fn update_info(&mut self, id: FlowID, info: Self::INFO) {
        // NOTE: 若 check_exist 有東西，就不記錄到 diff 裡（因為記錄過了）
        if !self.check_exist(id) {
            match self.get_inner_arena().get_flow_type(id) {
                FlowType::TSN => self.tsn_diff.push(id),
                FlowType::AVB => self.avb_diff.push(id),
            }
        }
        self.table.update_info(id, info);
    }
    fn foreach_avb(&self, mut callback: impl FnMut(&AVBFlow, &T)) {
        for &id in self.avb_diff.iter() {
            apply_callback(self, self.get_avb(id), |flow, t| callback(flow, t));
        }
    }
    fn foreach_tsn(&self, mut callback: impl FnMut(&TSNFlow, &T)) {
        for &id in self.tsn_diff.iter() {
            apply_callback(self, self.get_tsn(id), |flow, t| callback(flow, t));
        }
    }
    fn clone_as_diff(&self) -> DiffFlowTable<T> {
        self.clone()
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
    fn test_diff_flow_table() {
        let mut table = FlowTable::<usize>::new();
        let (tsns, avbs) = read_flows_from_file("test_flow.json", 1);
        assert_eq!(1, tsns.len());
        assert_eq!(5, avbs.len());
        assert_eq!(FlowID(0), table.get_max_id());
        table.insert(tsns, avbs, 0);
        assert_eq!(FlowID(5), table.get_max_id());
        assert_eq!(count_flows_iterative(&table), 6);
        assert_eq!(table.get_flow_cnt(), 6);

        assert_eq!(1, table.get_tsn_cnt());
        assert_eq!(5, table.get_avb_cnt());

        let mut changed = table.clone_as_diff();
        assert_eq!(changed.get_flow_cnt(), 0);
        assert_eq!(count_flows_iterative(&changed), 0);

        changed.update_info(2.into(), 99);
        assert_eq!(changed.get_flow_cnt(), 1);
        assert_eq!(count_flows_iterative(&changed), 1);

        changed.update_info(4.into(), 77);
        assert_eq!(changed.get_flow_cnt(), 2);
        assert_eq!(count_flows_iterative(&changed), 2);

        assert_eq!(changed.get_info(0.into()), None);
        assert_eq!(changed.get_info(2.into()), Some(&99));
        assert_eq!(changed.get_info(4.into()), Some(&77));
        assert_eq!(table.get_info(0.into()), Some(&0));
        assert_eq!(table.get_info(2.into()), Some(&0));
        assert_eq!(table.get_info(4.into()), Some(&0));

        table.apply_diff(true, &changed);
        assert_eq!(table.get_info(0.into()), Some(&0));
        assert_eq!(table.get_info(2.into()), Some(&99));
        assert_eq!(table.get_info(4.into()), Some(&77));
        assert_eq!(table.get_flow_cnt(), 6);
        assert_eq!(count_flows_iterative(&table), 6);
    }
    #[test]
    fn test_insert_return_id() {
        let mut table = FlowTable::<usize>::new();
        let (tsns, avbs) = read_flows_from_file("test_flow.json", 1);
        let new_ids = table.insert(tsns, avbs, 0);
        assert_eq!(6, new_ids.len());
        assert_eq!(FlowID(0), new_ids[0]);
        assert_eq!(FlowID(1), new_ids[1]);
        assert_eq!(FlowID(2), new_ids[2]);
        assert_eq!(FlowID(3), new_ids[3]);
        assert_eq!(FlowID(5), new_ids[5]);
    }
    #[test]
    #[should_panic]
    fn apply_diff_different_flows_should_panic() {
        let mut table = FlowTable::<usize>::new();
        let (tsns, avbs) = read_flows_from_file("test_flow.json", 1);
        table.insert(tsns.clone(), avbs.clone(), 0);
        let table2 = FlowTable::<usize>::new().clone_as_diff();
        table.insert(tsns, avbs, 0);
        table.apply_diff(true, &table2);
    }
    fn count_flows_iterative<FT: IFlowTable<INFO = usize>>(table: &FT) -> usize {
        let mut cnt = 0;
        table.foreach_avb(|_, _| {
            cnt += 1;
        });
        table.foreach_tsn(|_, _| {
            cnt += 1;
        });
        cnt
    }
}
