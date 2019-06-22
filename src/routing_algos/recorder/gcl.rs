use std::collections::HashMap;

use crate::MAX_QUEUE;

fn gcd(a: u32, b: u32) -> u32 {
    if a < b {
        gcd(b, a)
    } else if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}
fn lcm(a: u32, b: u32) -> u32 {
    let g = gcd(a, b);
    (a / g) * b
}

#[cfg(test)]
mod test {
    use super::lcm;
    #[test]
    fn test_lcm() {
        assert_eq!(36, lcm(4, 9));
        assert_eq!(81, lcm(27, 81));
        assert_eq!(84, lcm(12, 21));
    }
}

#[derive(Clone)]
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
    pub fn update_hyper_p(&mut self, new_p: u32) {
        self.hyper_p = lcm(self.hyper_p, new_p);
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
    pub fn get_gate_events(&self, link_id: usize) -> &Vec<(u32, u32, u8)> {
        assert!(self.gate_evt.len() > link_id, "GCL: 指定了超出範圍的邊");
        return &self.gate_evt[link_id];
    }
    pub fn insert_gate_evt(&mut self, link_id: usize,
        queue_id: u8, start_time: u32, duration: u32
    ) {
        let entry = (start_time, duration, queue_id);
        let evts = &mut self.gate_evt[link_id];
        match evts.binary_search(&entry) {
            Ok(_) => {
                // TODO 還有更多可能的錯誤
                println!("{} {:?}", link_id, self.get_gate_events(link_id));
                panic!("插入重複的閘門事件 {:?}", entry)
            },
            Err(pos) => evts.insert(pos, entry)
        }
    }
    pub fn insert_queue_evt(&mut self, link_id: usize,
        queue_id: u8, start_time: u32, duration: u32
    ) {
        let entry = (start_time, duration);
        let evts = &mut self.queue_occupy_evt[link_id][queue_id as usize];
        match evts.binary_search(&entry) {
            Ok(_) => panic!("插入重複的佇列事件"), // TODO 還有更多可能的錯誤
            Err(pos) => evts.insert(pos, entry)
        }
    }
    /// 會先確認 start~(start+duration) 這段時間中有沒有與其它事件重疊
    /// 
    /// 若否，則回傳 None，應可直接塞進去。若有重疊，則會告知下一個空的時間（但不一定塞得進去）
    pub fn get_next_empty_time(&self, link_id: usize,
        start: u32, duration: u32
    ) -> Option<u32> {
        let s1 = self.get_next_spot(link_id, start);
        let s2 = self.get_next_spot(link_id, start+duration);
        if s1.0 != s2.0 { // 是不同的閘門事
            Some(s1.0)
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
        for &(start, duration, _) in self.gate_evt[link_id].iter() {
            if start > time {
                return (start, true);
            } else if start + duration > time {
                return (start + duration, false);
            }
        }
        (self.hyper_p, true)
    }
    pub fn get_queueid(&self, link_id: usize, flow_id: usize) -> u8 {
        *self.queue_map.get(&(link_id, flow_id)).unwrap()
    }
    pub fn set_queueid(&mut self, queueid: u8, link_id: usize, flow_id: usize) {
        self.queue_map.insert((link_id, flow_id), queueid);
    }
    /// 回傳 None 者，代表當前即是空的
    pub fn get_next_queue_empty_time(&self,
        link_id: usize, queue_id: u8, time: u32,
    ) -> Option<u32> {
        let evts = &self.queue_occupy_evt[link_id][queue_id as usize];
        for &(start, duration) in evts.iter() {
            if start <= time {
                if start + duration > time {
                    return Some(start + duration);
                } else {
                    return None;
                }
            }
        }
        None
    }
}