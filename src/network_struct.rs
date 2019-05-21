use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

enum NodeType {
    Host,
    Switch,
}
struct Node {
    node_type: NodeType,
    id: i32,
    edges: HashMap<i32, (u32, Rc<RefCell<Node>>)>
}

pub struct Graph {
    map: HashMap<i32, Rc<RefCell<Node>>>,
    node_cnt: usize,
    edge_cnt: usize,
    next_node_id: i32,
}

impl Graph {
    pub fn new() -> Self {
        return Graph {
            map: HashMap::new(),
            node_cnt: 0,
            edge_cnt: 0,
            next_node_id: 0,
        };
    }
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
            id,
            edges: HashMap::new(),
        };
        self.map.insert(id, Rc::new(RefCell::new(node)));
        return id;
    }
    pub fn add_host(&mut self) -> i32 {
        return self._add_node(false);
    }
    pub fn add_switch(&mut self) -> i32 {
        return self._add_node(true);
    }
    fn _add_single_edge(&mut self, id_pair: (i32, i32), bandwidth: u32) -> bool {
        assert_ne!(id_pair.0, id_pair.1);
        if let Some(rc_node) = self.map.get(&id_pair.0) {
            let mut node = rc_node.borrow_mut();
            if let Some(rc_next) = self.map.get(&id_pair.1) {
                let edge = (bandwidth, rc_next.clone());
                node.edges.insert(id_pair.1, edge);
                return true;
            }
        }
        return false;
    }
    pub fn add_edge(&mut self, id_pair: (i32, i32), bandwidth: u32) -> bool {
        let mut t = self._add_single_edge(id_pair, bandwidth);
        t = t && self._add_single_edge((id_pair.1, id_pair.0), bandwidth);
        if t {
            self.edge_cnt += 1;
        }
        return t;
    }

    fn _del_single_edge(&mut self, id_pair: (i32, i32)) -> u32 {
        if let Some(rc_node) = self.map.get(&id_pair.0) {
            let mut node = rc_node.borrow_mut();
            if let Some(e) = node.edges.remove(&id_pair.1) {
                return e.0;
            }
        }
        panic!();
    }
    pub fn del_edge(&mut self, id_pair: (i32, i32)) -> u32 {
        let t = self._del_single_edge(id_pair);
        self.edge_cnt -= 1;
        return t;
    }
    pub fn del_node(&mut self, id: i32) -> bool {
        if let Some(rc_node) = self.map.remove(&id) {
            let node = rc_node.borrow();
            for (&next_id, _edge) in node.edges.iter() {
                self.del_edge((next_id, id));
            }
            self.node_cnt -= 1;
            return true;
        }
        return false;
    }
    pub fn foreach_edge(&self, id: i32, mut callback: impl FnMut(i32, u32) -> ()) {
        let node = self.map.get(&id).unwrap().borrow();
        for (id, (bandwidth, _)) in node.edges.iter() {
            callback(*id, *bandwidth);
        }
    }
    pub fn foreach_node(&self, mut callback: impl FnMut(i32, bool) -> ()) {
        for (&id, rc_node) in self.map.iter() {
            match rc_node.borrow().node_type {
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