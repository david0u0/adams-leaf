extern crate rand;
use std::collections::BinaryHeap;
use rand::Rng;

const MAX_K: usize = 10;

const TAO0: f64 = 5.0;
const R: usize = 50;
const L: usize = 7;
const RHO: f64 = 0.65;
const Q0: f64 = 0.3;

const MAXPH: f64 = 50.0;
const MINPH: f64 = 2.0;

type State = Vec<usize>;

#[derive(PartialOrd)]
struct WeightedState(f64, State);
impl PartialEq for WeightedState {
    fn eq(&self, other: &Self) -> bool {
        return self.0 == other.0;
    }
}
impl Eq for WeightedState { }
impl Ord for WeightedState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.0 > other.0 {
            std::cmp::Ordering::Greater
        } else if self.0 < other.0 {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

fn select_cluster(visibility: &[f64; MAX_K], pharamon: &[f64; MAX_K], k: usize) -> usize {
    if rand::thread_rng().gen_range(0.0, 1.0) < Q0 {
        let (mut min_i, mut min) = (0, std::f64::MAX);
        for i in 0..k {
            if min > pharamon[i] * visibility[i] {
                min = pharamon[i] * visibility[i];
                min_i = i;
            }
        }
        min_i
    } else {
        let mut sum = 0.0;
        for i in 0..k {
            sum += pharamon[i] * visibility[i];
        }
        let rand_f = rand::thread_rng().gen_range(0.0, 1.0);
        let mut accumulation = 0.0;
        for i in 0..k {
            accumulation += (pharamon[i] * visibility[i]) / sum;
            if accumulation >= rand_f {
                return i;
            }
        }
        k-1
    }
}

pub struct ACO<F> where F: FnMut(&State) -> f64 {
    pharamon: Vec<[f64; MAX_K]>,
    cost_estimate: F,
    state: State,
    k: usize
}

impl <F> ACO<F> where F: FnMut(&State) -> f64 {
    pub fn new(k: usize, state: Vec<usize>, cost_estimate: F) -> Self {
        ACO {
            pharamon: state.iter().map(|_| [TAO0; MAX_K]).collect(),
            cost_estimate, state, k,
        }
    }
    pub fn get_state(&self) -> &State {
        &self.state
    }
    pub fn get_pharamon(&self) -> &Vec<[f64; MAX_K]> {
        return &self.pharamon;
    }
    pub fn routine_aco(&mut self, visibility: &Vec<[f64; MAX_K]>, epoch: usize) {
        let mut min_cost = std::f64::MAX;
        let mut best_state: State = vec![];
        for _ in 0..epoch {
            let (local_best_state, local_min_cost)
                = self.do_single_colony(&visibility);
            if local_min_cost < min_cost {
                min_cost = local_min_cost;
                best_state = local_best_state;
            }
        }
        self.state = best_state;
    }
    fn do_single_colony(&mut self, visibility: &Vec<[f64; MAX_K]>) -> (State, f64) {
        let mut max_heap: BinaryHeap<WeightedState> = BinaryHeap::new();
        for _ in 0..R {
            let mut cur_state = self.state.clone();
            for i in 0..self.state.len() {
                let next = select_cluster(&visibility[i], &self.pharamon[i], self.k);
                cur_state[i] = next;
                // online pharamon update
            }
            let cost = (self.cost_estimate)(&cur_state);
            max_heap.push(WeightedState(-cost, cur_state));
        }
        // offline update
        let best_state = max_heap.pop().unwrap();
        self.evaporate();
        self.update_pheromon(&best_state);
        for _ in 0..L-1 {
            self.update_pheromon(&max_heap.pop().unwrap());
        }
        (best_state.1, -best_state.0)
    }
    fn evaporate(&mut self) {
        for i in 0..self.state.len() {
            for j in 0..self.k {
                let ph = (1.0 - RHO) * self.pharamon[i][j];
                self.pharamon[i][j] = ph;
            }
        }
    }
    fn update_pheromon(&mut self, w_state: &WeightedState) {
        for i in 0..w_state.1.len() {
            for j in 0..self.k {
                let mut ph = self.pharamon[i][j];
                if w_state.1[i] == j {
                    ph += 30.0 / (-w_state.0);
                }
                if ph > MAXPH {
                    ph = MAXPH;
                } else if ph < MINPH {
                    ph = MINPH;
                }
                self.pharamon[i][j] = ph;
            }
        }
    }
}