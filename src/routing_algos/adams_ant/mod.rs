use std::time::Instant;
use crate::util::YensAlgo;
use crate::MAX_K;
use crate::network_struct::Graph;
use crate::util::aco::ACO;
use super::{StreamAwareGraph, RoutingAlgo, Flow, FlowTable, GCL};
use super::time_and_tide::{schedule_online, schedule_fixed_og};

mod cost_calculate;
use cost_calculate::{compute_avb_cost, compute_all_avb_cost};

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
    pub fn new(g: &'a StreamAwareGraph, flow_table: Option<FT>, gcl: Option<GCL>) -> Self {
        let flow_table = flow_table.unwrap_or(FlowTable::new());
        let gcl = gcl.unwrap_or(GCL::new(1, g.get_edge_cnt()));
        AdamsAnt {
            gcl, flow_table,
            aco: ACO::new(0, K, None),
            g: g.clone(),
            yens_algo: YensAlgo::new(g, K),
            avb_count: 0,
            tt_count: 0,
        }
    }
    fn get_kth_route(&self, flow: &Flow, k: usize) -> &Vec<usize> {
        self.yens_algo.get_kth_route(*flow.src(), *flow.dst(), k)
    }
    fn schedule_online(&mut self, changed_table: &FT) -> Result<(), ()> {
        let _self = self as *const Self;
        unsafe {
            schedule_online(&mut self.flow_table, changed_table, &mut self.gcl,
                |flow, &k| {
                    let r = (*_self).get_kth_route(flow, k);
                    (*_self).g.get_links_id_bandwidth(r)
                }
            )
        }
    }
    unsafe fn save_flowid_on_edge(&self, remember: bool, flow: &Flow, k: usize) {
        let _g = &self.g as *const StreamAwareGraph as *mut StreamAwareGraph;
        let route = self.get_kth_route(flow, k);
        (*_g).save_flowid_on_edge(remember, *flow.id(), route);
    }
    fn do_aco(&self, time: Instant) {
        let cur_cost = compute_all_avb_cost(self);
        // self.aco
    }
}

impl <'a> RoutingAlgo for AdamsAnt<'a> {
    fn add_flows(&mut self, flows: Vec<Flow>) {
        let mut max_id = 0;
        for flow in flows.iter() {
            max_id = std::cmp::max(max_id, *flow.id());
            self.yens_algo.compute_routes(*flow.src(), *flow.dst());
            if flow.is_avb() {
                self.avb_count += 1;
            } else {
                self.tt_count += 1;
            }
        }
        self.aco.extend_state_len(max_id);

        // TT 排程
        let time = Instant::now();
        let mut new_table = FlowTable::new();
        new_table.insert(flows, 0);
        self.schedule_online(&new_table).unwrap();
        self.flow_table.union(true, &new_table);

        self.do_aco(time);
        let g = &mut self.g as *mut StreamAwareGraph;
        self.g.forget_all_flows();
        self.flow_table.foreach(true, |flow, r| {
            unsafe { self.save_flowid_on_edge(true, flow, *r) }
        });
    }
    fn del_flows(&mut self, flows: Vec<Flow>) {
        unimplemented!();
    }
    fn get_retouted_flows(&self) -> &Vec<usize> {
        unimplemented!();
    }
    fn get_route(&self, id: usize) -> &Vec<usize> {
        unimplemented!();
    }
}