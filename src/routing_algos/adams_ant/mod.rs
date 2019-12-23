use super::RoutingAlgo;
use crate::flow::{AVBFlow, Flow, FlowEnum, FlowID, TSNFlow};
use crate::graph_util::StreamAwareGraph;
use crate::network_wrapper::NetworkWrapper;
use crate::recorder::flow_table::prelude::*;
use crate::util::{aco::ACO, YensAlgo};
use crate::config::Config;
use crate::MAX_K;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

mod aco_routing;
use aco_routing::do_aco;

fn get_src_dst(flow: &FlowEnum) -> (usize, usize) {
    match flow {
        FlowEnum::AVB(flow) => (flow.src, flow.dst),
        FlowEnum::TSN(flow) => (flow.src, flow.dst),
    }
}

pub struct AdamsAnt {
    aco: ACO,
    yens_algo: Rc<RefCell<YensAlgo<usize, StreamAwareGraph>>>,
    wrapper: NetworkWrapper<usize>,
    compute_time: u128,
}
impl AdamsAnt {
    pub fn new(g: StreamAwareGraph) -> Self {
        let yens_algo = Rc::new(RefCell::new(YensAlgo::new(g.clone(), MAX_K)));
        let tmp_yens = yens_algo.clone();
        let wrapper = NetworkWrapper::new(g, move |flow_enum, &k| {
            let (src, dst) = get_src_dst(flow_enum);
            tmp_yens.borrow().get_kth_route(src, dst, k) as *const Vec<usize>
        });

        AdamsAnt {
            aco: ACO::new(0, MAX_K, None),
            yens_algo,
            compute_time: 0,
            wrapper,
        }
    }
    fn get_candidate_count<T: Clone>(&self, flow: &Flow<T>) -> usize {
        self.yens_algo.borrow().get_route_count(flow.src, flow.dst)
    }
}

impl RoutingAlgo for AdamsAnt {
    fn add_flows(&mut self, tsns: Vec<TSNFlow>, avbs: Vec<AVBFlow>) {
        let init_time = Instant::now();
        self.add_flows_in_time(tsns, avbs, Config::get().t_limit);
        self.compute_time = init_time.elapsed().as_micros();
    }
    fn del_flows(&mut self, tsns: Vec<TSNFlow>, avbs: Vec<AVBFlow>) {
        unimplemented!();
    }
    fn get_rerouted_flows(&self) -> &Vec<FlowID> {
        unimplemented!();
    }
    fn get_route(&self, id: FlowID) -> &Vec<usize> {
        self.wrapper.get_route(id)
    }
    fn show_results(&self) {
        println!("TT Flows:");
        for (flow, _) in self.wrapper.get_flow_table().iter_tsn() {
            let route = self.get_route(flow.id);
            println!("flow id = {:?}, route = {:?}", flow.id, route);
        }
        println!("AVB Flows:");
        for (flow, _) in self.wrapper.get_flow_table().iter_avb() {
            let route = self.get_route(flow.id);
            let cost = self.wrapper.compute_single_avb_cost(flow);
            println!(
                "flow id = {:?}, route = {:?} avb wcd / max latency = {:?}, reroute = {}",
                flow.id, route, cost.avb_wcd, cost.reroute_overhead
            );
        }
        let all_cost = self.wrapper.compute_all_cost();
        println!("the cost structure = {:?}", all_cost);
        println!("{}", all_cost.compute());
    }
    fn get_last_compute_time(&self) -> u128 {
        self.compute_time
    }
}

impl AdamsAnt {
    pub fn add_flows_in_time(&mut self, tsns: Vec<TSNFlow>, avbs: Vec<AVBFlow>, t_limit: u128) {
        for flow in tsns.iter() {
            self.yens_algo
                .borrow_mut()
                .compute_routes(flow.src, flow.dst);
        }
        for flow in avbs.iter() {
            self.yens_algo
                .borrow_mut()
                .compute_routes(flow.src, flow.dst);
        }
        self.wrapper.insert(tsns, avbs, 0);

        self.aco
            .extend_state_len(self.wrapper.get_flow_table().get_max_id().0 + 1);

        do_aco(self, t_limit);
    }
}
