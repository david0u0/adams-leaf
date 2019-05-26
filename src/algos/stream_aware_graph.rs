use std::collections::HashMap;

use crate::network_struct::{Graph, OnOffGraph};

#[derive(Copy, Clone)]
enum NodeType {
    Host,
    Switch,
}
struct Node {
    node_type: NodeType,
    edges: HashMap<usize, (f64, bool)>,
    exist: bool
}
impl Clone for Node {
    fn clone(&self) -> Self {
        let mut edges: HashMap<usize, (f64, bool)> = HashMap::new();
        for (&id, &edge) in self.edges.iter() {
            edges.insert(id, edge);
        }
        return Node {
            node_type: self.node_type.clone(),
            exist: self.exist,
            edges
        }
    }
}
#[derive(Clone)]
pub struct StreamAwareGraph {
    nodes: Vec<Node>,
    node_cnt: usize,
    edge_cnt: usize,
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
            let node_type = {
                if is_switch {
                    NodeType::Switch
                } else {
                    NodeType::Host
                }
            };
            let id = self.nodes.len();
            self.node_cnt += 1;
            let node = Node {
                node_type,
                exist: true,
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
    fn _add_single_edge(&mut self,
        id_pair: (usize, usize), bandwidth: f64
    ) {
        self.nodes[id_pair.0].edges.insert(id_pair.1, (bandwidth, true));
    }
    fn _del_single_edge(&mut self, id_pair: (usize, usize)) -> Result<f64, String> {
        if let Some(e) = self.nodes[id_pair.0].edges.remove(&id_pair.1) {
            return Ok(e.0);
        } else {
            return Err("刪除邊時發現邊不存在".to_owned());
        }
    }

    fn _change_edge_active(&mut self,
        id_pair: (usize, usize), active: bool
    ) -> Result<(), String> {
        if let Some(e) = self.nodes[id_pair.0].edges.get_mut(&id_pair.1) {
            e.1 = active;
            return Ok(());
        } else {
            return Err("修改邊的活性時發現邊不存在".to_owned());
        }
    }
    pub fn new() -> Self {
        return StreamAwareGraph {
            nodes: vec![],
            node_cnt: 0,
            edge_cnt: 0,
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
    fn add_edge(&mut self, id_pair: (usize, usize), bandwidth: f64) -> Result<(), String> {
        if self._check_exist(id_pair.0) && self._check_exist(id_pair.1) {
            self._add_single_edge(id_pair, bandwidth);
            self._add_single_edge((id_pair.1, id_pair.0), bandwidth);
            self.edge_cnt += 1;
            return Ok(());
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
        for (id, (bandwidth, active)) in node.edges.iter() {
            if *active && self.nodes[*id].exist {
                callback(*id, *bandwidth);
            }
        }
    }
    fn foreach_node(&self, mut callback: impl FnMut(usize, bool) -> ()) {
        for (id, node) in self.nodes.iter().enumerate() {
            if node.exist {
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
}
impl OnOffGraph<usize> for StreamAwareGraph {
    #[allow(unused_must_use)]
    fn activate_edge(&mut self, id_pair: (usize, usize)) -> Result<(), String> {
        self._change_edge_active(id_pair, true)?;
        self._change_edge_active((id_pair.1, id_pair.0), true);
        return Ok(());
    }
    #[allow(unused_must_use)]
    fn inactivate_edge(&mut self, id_pair: (usize, usize)) -> Result<(), String> {
        self._change_edge_active(id_pair, false)?;
        self._change_edge_active((id_pair.1, id_pair.0), false);
        return Ok(());
    }
}
