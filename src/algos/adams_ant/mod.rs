use crate::util::YensAlgo;
use crate::MAX_K;
use crate::network_struct::Graph;
use super::{StreamAwareGraph, RoutingAlgo, Flow, FlowTable, GCL};

mod aco;
use aco::{ACO, ACOArgsF64, ACOArgsUSize};

type FT = FlowTable<usize>;
const K: usize = 20;

pub struct AdamsAnt<'a> {
    aco: ACO,
    g: StreamAwareGraph,
    flow_table: FT,
    yens_algo: YensAlgo<'a, usize, StreamAwareGraph>,
    gcl: GCL,
    avb_count: usize,
    tt_count: usize,
}
impl <'a> AdamsAnt<'a> {
    pub fn new(g: &'a StreamAwareGraph) -> Self {
        AdamsAnt {
            aco: ACO::new(vec![], K, None),
            gcl: GCL::new(1, g.get_edge_cnt()),
            g: g.clone(),
            flow_table: FlowTable::new(),
            yens_algo: YensAlgo::new(g, K),
            avb_count: 0,
            tt_count: 0,
        }
    }
}

impl <'a> RoutingAlgo for AdamsAnt<'a> {
    fn compute_routes(&mut self, flows: Vec<Flow>) {
        unimplemented!();
    }
    fn get_retouted_flows(&self) -> &Vec<usize> {
        unimplemented!();
    }
    fn get_route(&self, id: usize) -> &Vec<usize> {
        unimplemented!();
    }
}