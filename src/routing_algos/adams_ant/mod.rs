use std::time::Instant;

use super::time_and_tide::schedule_online;
use super::{
    flow::{Flow, FlowID},
    flow_table_prelude::*,
    AVBFlow, RoutingAlgo, TSNFlow, GCL,
};
use crate::graph_util::{Graph, StreamAwareGraph};
use crate::util::aco::ACO;
use crate::util::YensAlgo;
use crate::T_LIMIT;

mod cost_calculate;
pub(self) use cost_calculate::{compute_all_avb_cost, compute_avb_cost, AVBCostResult};

mod aco_routing;
use aco_routing::do_aco;

type FT = FlowTable<usize>;
type DT = DiffFlowTable<usize>;
const K: usize = 20;

pub struct AdamsAnt<'a> {
    aco: ACO,
    g: StreamAwareGraph,
    flow_table: FT,
    yens_algo: YensAlgo<'a, usize, StreamAwareGraph>,
    gcl: GCL,
    compute_time: u128,
}
impl<'a> AdamsAnt<'a> {
    pub fn new(g: &'a StreamAwareGraph, flow_table: Option<FT>, gcl: Option<GCL>) -> Self {
        let flow_table = flow_table.unwrap_or(FlowTable::new());
        let gcl = gcl.unwrap_or(GCL::new(1, g.get_edge_cnt()));
        AdamsAnt {
            gcl,
            flow_table,
            aco: ACO::new(0, K, None),
            g: g.clone(),
            yens_algo: YensAlgo::new(g, K),
            compute_time: 0,
        }
    }
    pub fn get_kth_route<T: Clone>(&self, flow: &Flow<T>, k: usize) -> &Vec<usize> {
        self.yens_algo.get_kth_route(flow.src, flow.dst, k)
    }
    fn get_candidate_count<T: Clone>(&self, flow: &Flow<T>) -> usize {
        self.yens_algo.get_route_count(flow.src, flow.dst)
    }
    fn schedule_online(
        &self,
        gcl: &mut GCL,
        og_table: &mut FT,
        changed_table: &DT,
    ) -> Result<bool, ()> {
        let _self = self as *const Self;
        // TODO: 為什麼要這個 unsafe ?
        unsafe {
            schedule_online(og_table, changed_table, gcl, |flow, &k| {
                let r = (*_self).get_kth_route(flow, k);
                (*_self).g.get_links_id_bandwidth(r)
            })
        }
    }
    unsafe fn update_flowid_on_route(&self, remember: bool, flow: &AVBFlow, k: usize) {
        let _g = &self.g as *const StreamAwareGraph as *mut StreamAwareGraph;
        let route = self.get_kth_route(flow, k);
        (*_g).update_flowid_on_route(remember, flow.id, route);
    }
}

impl<'a> RoutingAlgo for AdamsAnt<'a> {
    fn add_flows(&mut self, tsns: Vec<TSNFlow>, avbs: Vec<AVBFlow>) {
        let init_time = Instant::now();
        self.add_flows_in_time(tsns, avbs, T_LIMIT);
        self.compute_time = init_time.elapsed().as_micros();
    }
    fn del_flows(&mut self, tsns: Vec<TSNFlow>, avbs: Vec<AVBFlow>) {
        unimplemented!();
    }
    fn get_rerouted_flows(&self) -> &Vec<FlowID> {
        unimplemented!();
    }
    fn get_route(&self, id: FlowID) -> &Vec<usize> {
        let k = *self.flow_table.get_info(id).unwrap();
        if let Some(flow) = self.flow_table.get_avb(id) {
            self.get_kth_route(flow, k)
        } else if let Some(flow) = self.flow_table.get_tsn(id) {
            self.get_kth_route(flow, k)
        } else {
            panic!("啥都找不到！")
        }
    }
    fn show_results(&self) {
        println!("TT Flows:");
        self.flow_table.foreach_tsn(|flow, &route_k| {
            let route = self.get_kth_route(flow, route_k);
            println!("flow id = {:?}, route = {:?}", flow.id, route);
        });
        println!("AVB Flows:");
        self.flow_table.foreach_avb(|flow, &route_k| {
            let route = self.get_kth_route(flow, route_k);
            let cost = self.compute_avb_cost(flow, Some(route_k)).1;
            println!(
                "flow id = {:?}, route = {:?} cost = {}",
                flow.id, route, cost
            );
        });
        println!("total avb cost = {}", self.compute_all_avb_cost().1);
    }
    fn get_last_compute_time(&self) -> u128 {
        self.compute_time
    }
}

impl<'a> AdamsAnt<'a> {
    pub fn compute_avb_cost(&self, flow: &AVBFlow, k: Option<usize>) -> AVBCostResult {
        compute_avb_cost(self, flow, k, &self.flow_table, &self.gcl)
    }
    pub fn compute_all_avb_cost(&self) -> AVBCostResult {
        compute_all_avb_cost(self, &self.flow_table, &self.gcl)
    }
    unsafe fn do_yens_algo(&self, src: usize, dst: usize) {
        let _self = self as *const Self as *mut Self;
        (*_self).yens_algo.compute_routes(src, dst);
    }
    pub fn add_flows_in_time(&mut self, tsns: Vec<TSNFlow>, avbs: Vec<AVBFlow>, t_limit: u128) {
        let new_ids = self.flow_table.insert(tsns, avbs, 0);
        let mut reconf = self.flow_table.clone_as_diff();

        unsafe {
            for &id in new_ids.iter() {
                reconf.update_info(id, 0);
                if let Some(flow) = self.flow_table.get_avb(id) {
                    self.yens_algo.compute_routes(flow.src, flow.dst);
                } else if let Some(flow) = self.flow_table.get_tsn(id) {
                    self.yens_algo.compute_routes(flow.src, flow.dst);
                }
            }
        }
        /*unsafe {
            for flow in tsns.iter() {
                self.do_yens_algo(flow.src, flow.dst);
                reconf.update_info(flow.id, 0);
            }
            for flow in avbs.iter() {
                self.do_yens_algo(flow.src, flow.dst);
                reconf.update_info(flow.id, 0);
            }
        }*/

        self.aco
            .extend_state_len(self.flow_table.get_max_id().0 + 1);

        do_aco(self, t_limit, reconf);
        self.g.forget_all_flows();
        self.flow_table
            .foreach_avb(|flow, r| unsafe { self.update_flowid_on_route(true, flow, *r) });
    }
}
