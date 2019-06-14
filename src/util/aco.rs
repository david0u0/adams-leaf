extern crate rand;
use std::collections::BinaryHeap;
use rand::Rng;
use crate::MAX_K;

const R: usize = 50;
const L: usize = 7;
const TAO0: f64 = 10.0;
const RHO: f64 = 0.65;
const Q0: f64 = 0.3;
const MAX_PH: f64 = 50.0;
const MIN_PH: f64 = 0.5;

pub enum ACOArgsF64 {
    Tao0, Rho, Q0, MaxPh, MinPh
}
pub enum ACOArgsUSize {
    R, L
}

type State = Vec<usize>;

#[derive(PartialOrd)]
struct WeightedState {
    neg_dist: f64, state: Option<State>
}
impl WeightedState {
    fn new(dist: f64, state: Option<State>) -> Self {
        WeightedState { neg_dist: -dist, state }
    }
    fn get_dist(&self) -> f64 {
        -self.neg_dist
    }
}
impl PartialEq for WeightedState {
    fn eq(&self, other: &Self) -> bool {
        return self.neg_dist == other.neg_dist;
    }
}
impl Eq for WeightedState { }
impl Ord for WeightedState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.neg_dist > other.neg_dist {
            std::cmp::Ordering::Greater
        } else if self.neg_dist < other.neg_dist {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

fn select_cluster(visibility: &[f64; MAX_K], pheromone: &[f64; MAX_K], k: usize, q0: f64) -> usize {
    if rand::thread_rng().gen_range(0.0, 1.0) < q0 {
        let (mut min_i, mut min) = (0, std::f64::MAX);
        for i in 0..k {
            if min > pheromone[i] * visibility[i] {
                min = pheromone[i] * visibility[i];
                min_i = i;
            }
        }
        min_i
    } else {
        let mut sum = 0.0;
        for i in 0..k {
            sum += pheromone[i] * visibility[i];
        }
        let rand_f = rand::thread_rng().gen_range(0.0, 1.0);
        let mut accumulation = 0.0;
        for i in 0..k {
            accumulation += (pheromone[i] * visibility[i]) / sum;
            if accumulation >= rand_f {
                return i;
            }
        }
        k-1
    }
}

pub struct ACO {
    pheromone: Vec<[f64; MAX_K]>,
    k: usize,
    r: usize,
    l: usize,
    rho: f64,
    tao0: f64,
    q0: f64,
    max_ph: f64,
    min_ph: f64
}

impl ACO {
    pub fn new(state_len: usize, k: usize, tao0: Option<f64>) -> Self {
        assert!(k <= MAX_K, "K值必需在{}以下", MAX_K);
        let tao0 = {
            if let Some(t) = tao0 {
                t
            } else {
                TAO0
            }
        };
        ACO {
            pheromone: (0..state_len).map(|_| [tao0; MAX_K]).collect(),
            tao0, k,
            r: R,
            l: L,
            rho: RHO,
            q0: Q0,
            max_ph: MAX_PH,
            min_ph: MIN_PH
        }
    }
    pub fn extend_state_len(&mut self, new_len: usize) {
        if new_len > self.pheromone.len() {
            let diff_len = new_len - self.pheromone.len();
            let tao0 = self.tao0;
            self.pheromone.extend((0..diff_len).map(|_| [tao0; MAX_K]));
        }
    }
    pub fn get_pharamon(&self) -> &Vec<[f64; MAX_K]> {
        return &self.pheromone;
    }
    pub fn do_aco<F>(&mut self, time_limit: u128,
        visibility: &Vec<[f64; MAX_K]>,
        mut calculate_dist: F, cur_dist: f64
    ) -> Option<State> where F: FnMut(&State) -> f64 {
        let time = std::time::Instant::now();
        let mut best_state = WeightedState::new(cur_dist, None);
        while time.elapsed().as_micros() < time_limit {
            let local_best_state = self.do_single_epoch(&visibility, &mut calculate_dist);
            if local_best_state.get_dist() < best_state.get_dist() {
                best_state = local_best_state;
            }
        }
        best_state.state
    }
    fn do_single_epoch<F>(&mut self, visibility: &Vec<[f64; MAX_K]>,
        calculate_dist: &mut F) -> WeightedState
    where F: FnMut(&State) -> f64 {
        let mut max_heap: BinaryHeap<WeightedState> = BinaryHeap::new();
        let state_len = self.pheromone.len();
        for _ in 0..self.r {
            let mut cur_state = Vec::<usize>::with_capacity(state_len);
            for i in 0..state_len {
                let next = select_cluster(&visibility[i], &self.pheromone[i], self.k, self.q0);
                cur_state.push(next);
                // TODO online pharamon update
            }
            let dist = calculate_dist(&cur_state);
            max_heap.push(WeightedState::new(dist, Some(cur_state)));
        }
        // offline update
        self.evaporate();
        let best_state = max_heap.pop().unwrap();
        self.update_pheromon(&best_state);
        for _ in 0..self.l-1 {
            self.update_pheromon(&max_heap.pop().unwrap());
        }
        best_state
    }
    fn evaporate(&mut self) {
        let state_len = self.pheromone.len();
        for i in 0..state_len {
            for j in 0..self.k {
                let ph = (1.0 - self.rho) * self.pheromone[i][j];
                self.pheromone[i][j] = ph;
            }
        }
    }
    fn update_pheromon(&mut self, w_state: &WeightedState) {
        let dist = w_state.get_dist();
        let state_len = self.pheromone.len();
        for i in 0..state_len {
            for j in 0..self.k {
                let mut ph = self.pheromone[i][j];
                if w_state.state.as_ref().unwrap()[i] == j {
                    ph += 1.0 / dist;
                }
                if ph > self.max_ph {
                    ph = self.max_ph;
                } else if ph < self.min_ph {
                    ph = self.min_ph;
                }
                self.pheromone[i][j] = ph;
            }
        }
    }
    pub fn set_args_f64(&mut self, arg_type: ACOArgsF64, arg: f64) {
        match arg_type {
            ACOArgsF64::Tao0 => self.tao0 = arg,
            ACOArgsF64::Rho => self.rho = arg,
            ACOArgsF64::Q0 => self.q0 = arg,
            ACOArgsF64::MaxPh => self.max_ph = arg,
            ACOArgsF64::MinPh => self.min_ph = arg
        }
    }
    pub fn set_args_usize(&mut self, arg_type: ACOArgsUSize, arg: usize) {
        match arg_type {
            ACOArgsUSize::L => self.l = arg,
            ACOArgsUSize::R => self.r = arg,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_ant_aco() {
        let mut aco = ACO::new(0, 2, None);
        aco.extend_state_len(10);
        let new_state = aco.do_aco(50000, &vec![[1.0; MAX_K]; 10], |state| {
            let mut cost = 6.0;
            for (i, &s) in state.iter().enumerate() {
                if i % 2 == 0 {
                    cost += s as f64;
                } else {
                    cost -= s as f64;
                }
            }
            cost / 6.0
        }, std::f64::MAX).unwrap();
        assert_eq!(vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1], new_state);
    }
}