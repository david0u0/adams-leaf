use super::{compute_avb_latency, NetworkWrapper, OldNew, OldNewTable};
use crate::config::Config;
use crate::flow::{AVBFlow, FlowEnum};
use crate::recorder::flow_table::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct RoutingCost {
    pub tsn_schedule_fail: bool,
    pub avb_fail_cnt: u32,
    pub avb_wcd: f64,
    pub reroute_overhead: u32,
    pub avb_cnt: usize,
    pub tsn_cnt: usize,
}

impl RoutingCost {
    pub fn compute(&self) -> f64 {
        let config = Config::get();
        let cost = self.compute_without_reroute_cost();
        cost + config.w3 * self.reroute_overhead as f64 / (self.avb_cnt + self.tsn_cnt) as f64
    }
    pub fn compute_without_reroute_cost(&self) -> f64 {
        let config = Config::get();
        let mut cost = 0.0;
        if self.tsn_schedule_fail {
            cost += config.w0;
        }
        cost += config.w1 * self.avb_fail_cnt as f64 / self.avb_cnt as f64;
        cost += config.w2 * self.avb_wcd / self.avb_cnt as f64;
        cost
    }
}
pub trait Calculator<T: Clone + Eq> {
    fn _compute_avb_wcd(&self, flow: &AVBFlow, route: Option<&T>) -> u32;
    fn _compute_single_avb_cost(&self, flow: &AVBFlow) -> RoutingCost;
    fn _compute_all_cost(&self) -> RoutingCost;
}

impl<T: Clone + Eq> Calculator<T> for NetworkWrapper<T> {
    fn _compute_avb_wcd(&self, flow: &AVBFlow, route: Option<&T>) -> u32 {
        let route_t = route.unwrap_or(self.flow_table.get_info(flow.id).unwrap());
        let route = unsafe {
            let r = (self.get_route_func)(self.flow_table.get(flow.id).unwrap(), route_t);
            &*r
        };
        compute_avb_latency(&self.graph, flow, route, &self.flow_table, &self.gcl)
    }
    fn _compute_single_avb_cost(&self, flow: &AVBFlow) -> RoutingCost {
        let avb_wcd = self._compute_avb_wcd(flow, None) as f64 / flow.max_delay as f64;
        let mut avb_fail_cnt = 0;
        let mut reroute_cnt = 0;
        if avb_wcd >= 1.0 {
            // 逾時了！
            avb_fail_cnt += 1;
        }
        if is_rerouted(
            self.flow_table.get(flow.id).unwrap(),
            self.flow_table.get_info(flow.id).unwrap(),
            self.old_new_table.as_ref().unwrap(),
        ) {
            reroute_cnt += 1;
        }
        RoutingCost {
            tsn_schedule_fail: self.tsn_fail,
            avb_cnt: 1,
            tsn_cnt: 0,
            avb_fail_cnt,
            avb_wcd,
            reroute_overhead: reroute_cnt,
        }
    }
    fn _compute_all_cost(&self) -> RoutingCost {
        let mut all_avb_fail_cnt = 0;
        let mut all_avb_wcd = 0.0;
        let mut all_reroute_cnt = 0;
        for (flow, t) in self.flow_table.iter() {
            if let FlowEnum::AVB(flow) = flow {
                let wcd = self._compute_avb_wcd(flow, None);
                all_avb_wcd += wcd as f64 / flow.max_delay as f64;
                if wcd > flow.max_delay {
                    // 逾時了！
                    all_avb_fail_cnt += 1;
                }
            }
            if is_rerouted(flow, t, self.old_new_table.as_ref().unwrap()) {
                all_reroute_cnt += 1;
            }
        }
        RoutingCost {
            tsn_schedule_fail: self.tsn_fail,
            avb_cnt: self.flow_table.get_avb_cnt(),
            tsn_cnt: self.flow_table.get_tsn_cnt(),
            avb_fail_cnt: all_avb_fail_cnt,
            avb_wcd: all_avb_wcd,
            reroute_overhead: all_reroute_cnt,
        }
    }
}

fn is_rerouted<T: Clone + Eq>(flow: &FlowEnum, route: &T, old_new_table: &OldNewTable<T>) -> bool {
    let id = match flow {
        FlowEnum::AVB(flow) => flow.id,
        FlowEnum::TSN(flow) => flow.id,
    };
    if let OldNew::Old(old_route) = old_new_table.get_info(id).unwrap() {
        route != old_route
    } else {
        false
    }
}
