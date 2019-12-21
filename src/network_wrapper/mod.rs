use crate::flow::{AVBFlow, FlowEnum, FlowID, TSNFlow};
use crate::graph_util::{Graph, StreamAwareGraph};
use crate::recorder::{flow_table::prelude::*, GCL};

type Route = Vec<usize>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(self) enum OldNew<T: Clone + Eq> {
    Old(T),
    New,
}

pub struct NetworkWrapper<T: Clone + Eq, F: Fn(usize, usize, &T) -> *const Route> {
    flow_table: FlowTable<T>,
    old_new_table: Option<FlowTable<OldNew<T>>>,
    get_route_func: F,
    gcl: GCL,
    graph: StreamAwareGraph,
}

impl<T: Clone + Eq, F: Fn(usize, usize, &T) -> *const Route> NetworkWrapper<T, F> {
    pub fn new(graph: StreamAwareGraph, get_route_func: F) -> Self {
        NetworkWrapper {
            flow_table: FlowTable::new(),
            old_new_table: None,
            gcl: GCL::new(1, graph.get_edge_cnt()),
            graph,
            get_route_func,
        }
    }
    /// 插入新的資料流，同時會捨棄先前的新舊表，並
    pub fn insert(
        &mut self,
        tsns: Vec<TSNFlow>,
        avbs: Vec<AVBFlow>,
        default_info: T,
    ) -> DiffFlowTable<T> {
        // 釋放舊的表備份表
        self.old_new_table = None;
        // 插入
        let new_ids = self.flow_table.insert(tsns, avbs, default_info.clone());
        let mut reconf = self.flow_table.clone_as_diff();

        for &id in new_ids.iter() {
            reconf.update_info(id, default_info.clone());
        }

        let old_new_table = self.flow_table.clone_as_type(|id, t| {
            if reconf.check_exist(id) {
                OldNew::New
            } else {
                OldNew::Old(t.clone())
            }
        });
        self.old_new_table = Some(old_new_table);

        reconf
    }
    pub fn get_route(&self, flow_id: FlowID) -> &Route {
        let flow_enum = self.flow_table.get(flow_id).unwrap();
        let info = self.flow_table.get_info(flow_id).unwrap();
        let route = {
            match flow_enum {
                // TODO: rust 難道沒有更好的寫法嗎？
                FlowEnum::AVB(flow) => (self.get_route_func)(flow.src, flow.dst, info),
                FlowEnum::TSN(flow) => (self.get_route_func)(flow.src, flow.dst, info),
            }
        };
        unsafe { &*route }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::flow::{data::TSNData, TSNFlow};
    use crate::read_topo_from_file;
    use std::collections::HashMap;

    struct Env(HashMap<(usize, usize), Vec<Route>>);
    impl Env {
        pub fn new() -> Self {
            let mut map = HashMap::new();
            map.insert((0, 2), vec![vec![0, 2], vec![0, 1, 2]]);
            map.insert((1, 2), vec![vec![1, 2]]);
            Env(map)
        }
        pub fn get_route(&self, src: usize, dst: usize, i: usize) -> *const Route {
            &self.0.get(&(src, dst)).unwrap()[i]
        }
    }

    #[test]
    fn test_insert_get_route() {
        let graph = read_topo_from_file("test_graph.json");
        let env = Env::new();
        let mut wrapper = NetworkWrapper::new(graph, move |src: usize, dst: usize, k: &usize| {
            env.get_route(src, dst, *k)
        });
        let flows = vec![
            TSNFlow {
                id: 0.into(),
                src: 0,
                dst: 2,
                size: 100,
                period: 100,
                max_delay: 100,
                spec_data: TSNData { offset: 0 },
            },
            TSNFlow {
                id: 0.into(),
                src: 0,
                dst: 2,
                size: 100,
                period: 150,
                max_delay: 150,
                spec_data: TSNData { offset: 0 },
            },
            TSNFlow {
                id: 0.into(),
                src: 1,
                dst: 2,
                size: 100,
                period: 200,
                max_delay: 200,
                spec_data: TSNData { offset: 0 },
            },
        ];
        wrapper.insert(flows.clone(), vec![], 0);

        wrapper.flow_table.update_info(1.into(), 1);

        assert_eq!(&vec![0, 2], wrapper.get_route(0.into()));
        assert_eq!(&vec![0, 1, 2], wrapper.get_route(1.into()));
        assert_eq!(&vec![1, 2], wrapper.get_route(2.into()));
        let old_new = wrapper
            .old_new_table
            .as_ref()
            .unwrap()
            .get_info(1.into())
            .unwrap();
        assert_eq!(&OldNew::New, old_new);

        wrapper.insert(flows.clone(), vec![], 0);
        assert_eq!(&vec![0, 2], wrapper.get_route(3.into()));
        assert_eq!(&vec![0, 2], wrapper.get_route(4.into()));
        assert_eq!(&vec![1, 2], wrapper.get_route(5.into()));
        let old_new = wrapper
            .old_new_table
            .as_ref()
            .unwrap()
            .get_info(1.into())
            .unwrap();
        assert_eq!(&OldNew::Old(1), old_new);
        let old_new = wrapper
            .old_new_table
            .as_ref()
            .unwrap()
            .get_info(3.into())
            .unwrap();
        assert_eq!(&OldNew::New, old_new);

        // wrapper.flow_table.insert(flows, vec![], 0); // 反註解這行會導致執行期錯誤
    }
}
