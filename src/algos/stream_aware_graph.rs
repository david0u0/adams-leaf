use std::collections::HashMap;

use crate::network_struct::{Graph, OnOffGraph};

#[derive(Copy, Clone)]
enum NodeType {
    Host,
    Switch,
}
struct Node {
    node_type: NodeType,
    edges: HashMap<i32, (f64, bool)>
}
impl Clone for Node {
    fn clone(&self) -> Self {
        let mut edges: HashMap<i32, (f64, bool)> = HashMap::new();
        for (&id, &edge) in self.edges.iter() {
            edges.insert(id, edge);
        }
        return Node {
            node_type: self.node_type.clone(),
            edges
        }
    }
}

pub struct StreamAwareGraph {
    map: HashMap<i32, Node>,
    node_cnt: usize,
    edge_cnt: usize,
    next_node_id: i32,
}
impl StreamAwareGraph {
    fn _add_node(&mut self, is_switch: bool) -> i32 {
        let node_type = {
            if is_switch {
                NodeType::Switch
            } else {
                NodeType::Host
            }
        };
        let id = self.next_node_id;
        self.next_node_id += 1;
        self.node_cnt += 1;
        let node = Node {
            node_type,
            edges: HashMap::new(),
        };
        self.map.insert(id, node);
        return id;
    }
    fn _add_single_edge(&mut self, id_pair: (i32, i32), bandwidth: f64) {
        assert_ne!(id_pair.0, id_pair.1);
        if self.map.contains_key(&id_pair.1) {
            if let Some(node) = self.map.get_mut(&id_pair.0) {
                let edge = (bandwidth, true);
                node.edges.insert(id_pair.1, edge);
                return;
            }
        }
        panic!("加入邊的時候發現邊或節點不存在");
    }
    fn _del_single_edge(&mut self, id_pair: (i32, i32)) -> f64 {
        if let Some(node) = self.map.get_mut(&id_pair.0) {
            if let Some(e) = node.edges.remove(&id_pair.1) {
                return e.0;
            }
        }
        panic!("刪除邊的時候發現邊或節點不存在");
    }

    fn _change_edge_active(&mut self, id_pair: (i32, i32), active: bool) {
        if let Some(node) = self.map.get_mut(&id_pair.0) {
            if let Some(e) = node.edges.get_mut(&id_pair.1) {
                e.1 = active;
                return;
            }
        }
        panic!("修改邊的活性時發現邊或節點不存在");
    }
    pub fn new() -> Self {
        return StreamAwareGraph {
            map: HashMap::new(),
            node_cnt: 0,
            edge_cnt: 0,
            next_node_id: 0,
        };
    }
}
impl Clone for StreamAwareGraph {
    fn clone(&self) -> Self {
        let mut map: HashMap<i32, Node> = HashMap::new();
        for (&id, node) in self.map.iter() {
            map.insert(id, node.clone());
        }
        return StreamAwareGraph {
            map,
            node_cnt: self.node_cnt,
            edge_cnt: self.edge_cnt,
            next_node_id: self.next_node_id,
        }
    }
}
impl Graph for StreamAwareGraph {
    fn add_host(&mut self) -> i32 {
        return self._add_node(false);
    }
    fn add_switch(&mut self) -> i32 {
        return self._add_node(true);
    }
    fn add_edge(&mut self, id_pair: (i32, i32), bandwidth: f64) {
        self._add_single_edge(id_pair, bandwidth);
        self._add_single_edge((id_pair.1, id_pair.0), bandwidth);
        self.edge_cnt += 1;
    }
    fn del_edge(&mut self, id_pair: (i32, i32)) -> f64 {
        let t = self._del_single_edge(id_pair);
        self.edge_cnt -= 1;
        return t;
    }
    fn del_node(&mut self, id: i32) -> bool {
        if let Some(node) = self.map.remove(&id) {
            for (&next_id, _edge) in node.edges.iter() {
                self.del_edge((next_id, id));
            }
            self.node_cnt -= 1;
            return true;
        }
        return false;
    }
    fn foreach_edge(&self, id: i32, mut callback: impl FnMut(i32, f64) -> ()) {
        let node = self.map.get(&id).unwrap();
        for (id, (bandwidth, active)) in node.edges.iter() {
            if *active {
                callback(*id, *bandwidth);
            }
        }
    }
    fn foreach_node(&self, mut callback: impl FnMut(i32, bool) -> ()) {
        for (&id, node) in self.map.iter() {
            match node.node_type {
                NodeType::Host => {
                    callback(id, false);
                },
                NodeType::Switch => {
                    callback(id, true);
                }
            }
        }
    }
}
impl OnOffGraph for StreamAwareGraph {
    fn activate_edge(&mut self, id_pair: (i32, i32)) {
        self._change_edge_active(id_pair, true);
        self._change_edge_active((id_pair.1, id_pair.0), true);
    }
    fn inactivate_edge(&mut self, id_pair: (i32, i32)) {
        self._change_edge_active(id_pair, false);
        self._change_edge_active((id_pair.1, id_pair.0), false);
    }
}
