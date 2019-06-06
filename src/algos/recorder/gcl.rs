use std::collections::HashMap;

use crate::MAX_QUEUE;

pub struct GCL {
    hyper_p: u32,
    // TODO 這個資料結構有優化的空間
    gate_evt: Vec<Vec<(u32, u32, u8)>>,
    queue_occupy_evt: Vec<[Vec<(u32, u32)>; MAX_QUEUE as usize]>,
    queue_map: HashMap<(usize, usize), u8>,
}
impl GCL {
    pub fn new(hyper_p: u32, edge_count: usize) -> Self {
        GCL {
            gate_evt: vec![vec![]; edge_count],
            queue_occupy_evt: vec![Default::default(); edge_count],
            queue_map: HashMap::new(),
            hyper_p
        }
    }
    pub fn clear(&mut self) {
        self.queue_map = HashMap::new();
        self.gate_evt.clear();
    }
    pub fn get_hyper_p(&self) -> u32 {
        self.hyper_p
    }
    /// 回傳 `link_id` 上所有閘門關閉事件。
    /// * `回傳值` - 一個陣列，其內容為 (事件開始時間, 事件持續時間);
    pub fn get_close_event(&self, link_id: usize) -> &Vec<(u32, u32, u8)> {
        assert!(self.gate_evt.len() > link_id, "GCL: 指定了超出範圍的邊");
        return &self.gate_evt[link_id];
    }
    pub fn insert_gate_evt(&mut self, link_id: usize,
        queue_id: u8, start_time: u32, duration: u32
    ) {
        // FIXME: 應該做個二元搜索再插入
        return self.gate_evt[link_id].push((start_time, duration, queue_id));
    }
    pub fn insert_queue_evt(&mut self, link_id: usize,
        queue_id: u8, start_time: u32, duration: u32
    ) {
        let vec = &mut self.queue_occupy_evt[link_id][queue_id as usize];
        // FIXME: 應該做個二元搜索再插入
        vec.push((start_time, duration));
    }
    /// 會先確認 start~(start+duration) 這段時間中有沒有與其它事件重疊
    /// 
    /// 若否，則回傳 None，應可直接塞進去。若有重疊，則會告知下一個空的時間（但不一定塞得進去）
    pub fn get_next_empty_time(&self, link_id: usize,
        start: u32, duration: u32
    ) -> Option<u32> {
        let s1 = self.get_next_spot(link_id, start);
        let s2 = self.get_next_spot(link_id, start+duration);
        if s1.0 != s2.0 {
            Some(s2.0)
        } else if s1.1 { // 是同一個閘門事件的開始
            None
        } else { // 是同一個閘門事件的結束，代表 start~duration 這段時間正處於該事件之中，重疊了!
            Some(s2.0)
        }
    }
    /// 計算最近的下一個「時間點」，此處的時間點有可能是閘門事件的開啟或結束。
    /// 
    /// 回傳一組資料(usize, bool)，前者代表時間，後者代表該時間是閘門事件的開始還是結束（真代表開始）
    fn get_next_spot(&self, link_id: usize, time: u32) -> (u32, bool) {
        // TODO 應該用二元搜索來優化?
        unimplemented!();
    }
    pub fn get_queueid(&self, link_id: usize, flow_id: usize) -> u8 {
        *self.queue_map.get(&(link_id, flow_id)).unwrap()
    }
    pub fn set_queueid(&mut self, queueid: u8, link_id: usize, flow_id: usize) {
        self.queue_map.insert((link_id, flow_id), queueid);
    }
    pub fn get_next_queue_empty_time(&self, link_id: usize,
        queue_id: u8, time: u32,
    ) -> Option<u32> {
        unimplemented!();
    }
}
