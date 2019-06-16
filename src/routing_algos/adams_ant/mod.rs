use std::time::Instant;

use crate::util::YensAlgo;
use crate::network_struct::Graph;
use crate::util::aco::ACO;
use super::{StreamAwareGraph, RoutingAlgo, Flow, FlowTable, GCL};
use super::time_and_tide::schedule_online;

mod cost_calculate;
pub(self) use cost_calculate::{compute_avb_cost, compute_all_avb_cost};

mod aco_routing;
use aco_routing::do_aco;

type FT = FlowTable<usize>;
const K: usize = 20;
const T_LIMIT: u128 = 100 * 1000;

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
    pub fn get_kth_route(&self, flow_id: usize, k: usize) -> &Vec<usize> {
        let flow = self.flow_table.get_flow(flow_id);
        self.yens_algo.get_kth_route(*flow.src(), *flow.dst(), k)
    }
    fn get_candidate_count(&self, flow: &Flow) -> usize {
        self.yens_algo.get_route_count(*flow.src(), *flow.dst())
    }
    fn schedule_online(&mut self, changed_table: &FT) -> Result<(), ()> {
        let _self = self as *const Self;
        unsafe {
            schedule_online(&mut self.flow_table, changed_table, &mut self.gcl,
                |flow, &k| {
                    let r = (*_self).get_kth_route(*flow.id(), k);
                    (*_self).g.get_links_id_bandwidth(r)
                }
            )
        }
    }
    unsafe fn save_flowid_on_edge(&self, remember: bool, flow_id: usize, k: usize) {
        let _g = &self.g as *const StreamAwareGraph as *mut StreamAwareGraph;
        let route = self.get_kth_route(flow_id, k);
        (*_g).save_flowid_on_edge(remember, flow_id, route);
    }
}

impl <'a> RoutingAlgo for AdamsAnt<'a> {
    fn add_flows(&mut self, flows: Vec<Flow>) {
        let time = Instant::now();
        let mut max_id = 0;
        self.flow_table.insert(flows.clone(), 0);
        let mut table_changed = self.flow_table.clone_into_changed_table();
        for flow in flows.iter() {
            max_id = std::cmp::max(max_id, *flow.id());
            self.yens_algo.compute_routes(*flow.src(), *flow.dst());
            table_changed.update_info(*flow.id(), 0);
            if flow.is_avb() {
                self.avb_count += 1;
            } else {
                self.tt_count += 1;
            }
        }
        self.aco.extend_state_len(max_id + 1);

        // TT 排程
        self.schedule_online(&table_changed).unwrap();

        do_aco(self, T_LIMIT - time.elapsed().as_micros(), table_changed);
        self.g.forget_all_flows();
        self.flow_table.foreach(true, |flow, r| {
            unsafe { self.save_flowid_on_edge(true, *flow.id(), *r) }
        });
    }
    fn del_flows(&mut self, flows: Vec<Flow>) {
        unimplemented!();
    }
    fn get_retouted_flows(&self) -> &Vec<usize> {
        unimplemented!();
    }
    fn get_route(&self, id: usize) -> &Vec<usize> {
        let k = *self.flow_table.get_info(id);
        self.get_kth_route(id, k)
    }
}

impl <'a> AdamsAnt<'a> {
    pub fn compute_avb_cost(&self, flow: &Flow, k: Option<usize>) -> f64 {
        compute_avb_cost(self, flow, k, &self.flow_table, &self.gcl)
    }
    pub fn compute_all_avb_cost(&self) -> f64 {
        compute_all_avb_cost(self, &self.flow_table, &self.gcl)
    }
}