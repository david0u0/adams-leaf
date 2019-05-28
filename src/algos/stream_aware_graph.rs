use std::collections::HashMap;

use crate::network_struct::{Graph, OnOffGraph};

struct Node {
    is_switch: bool,
    edges: HashMap<usize, (usize, f64, bool)>,
    exist: bool,
    active: bool,
}
impl Clone for Node {
    fn clone(&self) -> Self {
        let mut edges: HashMap<usize, (usize, f64, bool)> = HashMap::new();
        for (&id, &edge) in self.edges.iter() {
            edges.insert(id, edge);
        }
        return Node {
            is_switch: self.is_switch,
            exist: self.exist,
            active: self.active,
            edges
        }
    }
}
#[derive(Clone)]
pub struct StreamAwareGraph {
    nodes: Vec<Node>,
    node_cnt: usize,
    edge_cnt: usize,
    cur_edge_id: usize,
    inactive_edges: Vec<(usize, usize)>,
    inactive_nodes: Vec<usize>
}
impl StreamAwareGraph {
    fn _add_node(&mut self, cnt: Option<usize>, is_switch: bool) -> Vec<usize> {
        let cnt = {
            if let Some(_cnt) = cnt {
                _cnt
            } else {
                1
            }
        };
        let mut v: Vec<usize> = vec![];
        for _ in 0..cnt {
            let id = self.nodes.len();
            self.node_cnt += 1;
            let node = Node {
                is_switch,
                exist: true,
                active: true,
                edges: HashMap::new(),
            };
            self.nodes.push(node);
            v.push(id);
        }
        return v;
    }
    fn _check_exist(&self, id: usize) -> bool {
        return id < self.nodes.len() && self.nodes[id].exist;
    }
    fn _add_single_edge(&mut self, edge_id: usize,
        id_pair: (usize, usize), bandwidth: f64
    ) {
        self.nodes[id_pair.0].edges.insert(id_pair.1, (edge_id, bandwidth, true));
    }
    fn _del_single_edge(&mut self, id_pair: (usize, usize)) -> Result<f64, String> {
        if let Some(e) = self.nodes[id_pair.0].edges.remove(&id_pair.1) {
            return Ok(e.1);
        } else {
            return Err("刪除邊時發現邊不存在".to_owned());
        }
    }

    fn _change_edge_active(&mut self,
        id_pair: (usize, usize), active: bool
    ) -> Result<(), String> {
        if let Some(e) = self.nodes[id_pair.0].edges.get_mut(&id_pair.1) {
            e.2 = active;
            return Ok(());
        } else {
            return Err("修改邊的活性時發現邊不存在".to_owned());
        }
    }
    fn _change_node_active(&mut self, id: usize, active: bool) -> Result<(), String> {
        if self._check_exist(id) {
            self.nodes[id].active = active;
            return Ok(());
        } else {
            return Err("修改節點的活性時發現節點不存在".to_owned());
        }
    }
    pub fn new() -> Self {
        return StreamAwareGraph {
            nodes: vec![],
            node_cnt: 0,
            edge_cnt: 0,
            cur_edge_id: 0,
            inactive_edges: vec![],
            inactive_nodes: vec![]
        };
    }
}
impl Graph<usize> for StreamAwareGraph {
    fn add_host(&mut self, cnt: Option<usize>) -> Vec<usize> {
        return self._add_node(cnt, false);
    }
    fn add_switch(&mut self, cnt: Option<usize>) -> Vec<usize> {
        return self._add_node(cnt, true);
    }
    fn get_edge_cnt(&self) -> usize {
        return self.edge_cnt;
    }
    fn get_node_cnt(&self) -> usize {
        return self.node_cnt;
    }
    fn add_edge(&mut self, id_pair: (usize, usize), bandwidth: f64) -> Result<usize, String> {
        if self._check_exist(id_pair.0) && self._check_exist(id_pair.1) {
            let edge_id = self.cur_edge_id;
            self._add_single_edge(edge_id, id_pair, bandwidth);
            self._add_single_edge(edge_id, (id_pair.1, id_pair.0), bandwidth);
            self.edge_cnt += 1;
            self.cur_edge_id += 1;
            return Ok(edge_id);
        } else {
            return Err("加入邊時發現節點不存在".to_owned());
        }
    }
    fn del_edge(&mut self, id_pair: (usize, usize)) -> Result<f64, String> {
        if self._check_exist(id_pair.0) && self._check_exist(id_pair.1) {
            self._del_single_edge(id_pair)?;
            self.edge_cnt -= 1;
            return self._del_single_edge((id_pair.1, id_pair.0));
        } else {
            return Err("刪除邊時發現節點不存在".to_owned());
        }
    }
    fn del_node(&mut self, id: usize) -> Result<(), String> {
        if self._check_exist(id) {
            let _self = self as *mut Self;
            let edges = &self.nodes[id].edges;
            for (&next_id, _edge) in edges.iter() {
                unsafe {
                    if let Err(msg) = (*_self).del_edge((next_id, id)) {
                        panic!(msg);
                    }
                }
            }
            self.nodes[id].exist = false;
            self.node_cnt -= 1;
            return Ok(());
        } else {
            return Err("找不到欲刪除的節點".to_owned());
        }
    }
    fn foreach_edge(&self, id: usize, mut callback: impl FnMut(usize, f64) -> ()) {
        let node = &self.nodes[id];
        for (&id, &(_, bandwidth, active)) in node.edges.iter() {
            let node = &self.nodes[id];
            if active && node.exist && node.active {
                callback(id, bandwidth);
            }
        }
    }
    fn foreach_node(&self, mut callback: impl FnMut(usize, bool) -> ()) {
        for (id, node) in self.nodes.iter().enumerate() {
            if node.exist && node.active {
                callback(id, node.is_switch);
            }
        }
    }
    fn get_dist(&self, path: &Vec<usize>) -> f64 {
        let mut dist = 0.0;
        for i in 0..path.len()-1 {
            let (cur, next) = (path[i], path[i+1]);
            if let Some((_, bandwidth, _)) = self.nodes[cur].edges.get(&next) {
                dist += 1.0 / bandwidth;
            } else {
                return std::f64::MAX;
            }
        }
        return dist;
    }
    fn get_edge_ids(&self, path: &Vec<usize>) -> Vec<usize> {
        let mut vec: Vec<usize> = vec![];
        for i in 0..path.len()-1 {
            let (cur, next) = (path[i], path[i+1]);
            if let Some((edge_id, _, _)) = self.nodes[cur].edges.get(&next) {
                vec.push(*edge_id);
            } else {
                panic!("get_link_ids: 不連通的路徑");
            }
        }
        return vec;
    }
}
impl OnOffGraph<usize> for StreamAwareGraph {
    #[allow(unused_must_use)]
    fn inactivate_edge(&mut self, id_pair: (usize, usize)) -> Result<(), String> {
        if self._check_exist(id_pair.0) && self._check_exist(id_pair.1) {
            self._change_edge_active(id_pair, false)?;
            self._change_edge_active((id_pair.1, id_pair.0), false);
            self.inactive_edges.push(id_pair);
            return Ok(());
        } else {
            return Err("修改邊的活性時發現節點不存在".to_owned());
        }
    }
    fn inactivate_node(&mut self, id: usize) -> Result<(), String> {
        self._change_node_active(id, false)?;
        self.inactive_nodes.push(id);
        return Ok(());
    }
    #[allow(unused_must_use)]
    fn reset(&mut self) {
        let _self = self as *mut Self;
        for pair in self.inactive_edges.iter() {
            unsafe {
                (*_self)._change_edge_active(*pair, true);
                (*_self)._change_edge_active((pair.1, pair.0), true);
            }
        }
        for id in self.inactive_nodes.iter() {
            unsafe {
                (*_self)._change_node_active(*id, true);
            }
        }
        self.inactive_edges.clear();
        self.inactive_nodes.clear();
    }
}
