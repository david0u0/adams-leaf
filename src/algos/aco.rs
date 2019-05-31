extern crate rand;
use std::collections::BinaryHeap;
use rand::Rng;

const MAX_K: usize = 10;

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
    state: State,
    k: usize,
    tao0: f64,
    r: usize,
    l: usize,
    rho: f64,
    q0: f64,
    max_ph: f64,
    min_ph: f64
}

impl ACO {
    pub fn new(state: Vec<usize>, k: usize, tao0: Option<f64>) -> Self {
        let tao0 = {
            if let Some(t) = tao0 {
                t
            } else {
                TAO0
            }
        };
        ACO {
            pheromone: state.iter().map(|_| [tao0; MAX_K]).collect(),
            state, tao0, k,
            r: R,
            l: L,
            rho: RHO,
            q0: Q0,
            max_ph: MAX_PH,
            min_ph: MIN_PH
        }
    }
    pub fn get_state(&self) -> &State {
        &self.state
    }
    pub fn get_pharamon(&self) -> &Vec<[f64; MAX_K]> {
        return &self.pheromone;
    }
    pub fn routine_aco<F>(&mut self, epoch: usize,
        visibility: &Vec<[f64; MAX_K]>, mut cost_estimate: F
    ) where F: FnMut(&State) -> f64 {
        let mut min_cost = std::f64::MAX;
        let mut best_state: State = vec![];
        for _ in 0..epoch {
            let (local_best_state, local_min_cost)
                = self.do_single_colony(&visibility, &mut cost_estimate);
            if local_min_cost < min_cost {
                min_cost = local_min_cost;
                best_state = local_best_state;
            }
        }
        self.state = best_state;
    }
    fn do_single_colony<F>(&mut self, visibility: &Vec<[f64; MAX_K]>,
        cost_estimate: &mut F) -> (State, f64)
    where F: FnMut(&State) -> f64 {
        let mut max_heap: BinaryHeap<WeightedState> = BinaryHeap::new();
        for _ in 0..self.r {
            let mut cur_state = self.state.clone();
            for i in 0..self.state.len() {
                let next = select_cluster(&visibility[i], &self.pheromone[i], self.k, self.q0);
                cur_state[i] = next;
                // online pharamon update
            }
            let cost = (cost_estimate)(&cur_state);
            max_heap.push(WeightedState(-cost, cur_state));
        }
        // offline update
        let best_state = max_heap.pop().unwrap();
        self.evaporate();
        self.update_pheromon(&best_state);
        for _ in 0..self.l-1 {
            self.update_pheromon(&max_heap.pop().unwrap());
        }
        (best_state.1, -best_state.0)
    }
    fn evaporate(&mut self) {
        for i in 0..self.state.len() {
            for j in 0..self.k {
                let ph = (1.0 - self.rho) * self.pheromone[i][j];
                self.pheromone[i][j] = ph;
            }
        }
    }
    fn update_pheromon(&mut self, w_state: &WeightedState) {
        for i in 0..w_state.1.len() {
            for j in 0..self.k {
                let mut ph = self.pheromone[i][j];
                if w_state.1[i] == j {
                    ph += (1.0 / (-w_state.0));
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
    /// 根據一組鍵值重新排列所有狀態，鍵值亦會被重新排列
    /// * `state_key` - 所有狀態將會依照此鍵值表重新排列
    /// * `cmp` - 一個函式。若 cmp(a, b) = true，則 a 會排在 b 前面。
    pub fn reorder<T: Clone, F: Fn(&T, &T) -> bool>(&mut self, state_key: &mut Vec<T>, cmp: F) {
        assert_eq!(state_key.len(), self.state.len());
        let mut tmp_vec = state_key.into_iter().enumerate().map(|(i, sv)| {
            (i, sv)
        }).collect::<Vec<_>>();
        tmp_vec.sort_by(|a, b| {
            if cmp(&a.1, &b.1) {
                return std::cmp::Ordering::Less;
            } else {
                return std::cmp::Ordering::Greater;
            }
        });
        let mut tmp_state = Vec::<usize>::with_capacity(tmp_vec.len());
        let mut tmp_ph = Vec::<[f64; MAX_K]>::with_capacity(tmp_vec.len());
        for i in 0..tmp_vec.len() {
            tmp_state.push(self.state[tmp_vec[i].0]);
            tmp_ph.push(self.pheromone[tmp_vec[i].0]);
        }
        self.state = tmp_state;
        self.pheromone = tmp_ph;
        *state_key = tmp_vec.into_iter().map(|(_, s)| s.clone()).collect();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_ant_aco1() {
        let mut aco = ACO::new(vec![0; 10], 2, None);
        aco.routine_aco(20, &vec![[1.0; 10]; 10], |state| {
            let mut cost = 6.0;
            for (i, &s) in state.iter().enumerate() {
                if i % 2 == 0 {
                    cost += s as f64;
                } else {
                    cost -= s as f64;
                }
            }
            cost / 6.0
        });
        assert_eq!(vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1], *aco.get_state());
    }
    #[test]
    fn test_ant_reorder() {
        let mut aco = ACO::new(vec![5, 6, 7, 8, 9], 10, None);
        let mut state_key = vec![0, 1, 2, 4, 3];
        aco.reorder(&mut state_key, |a, b| a > b);
        assert_eq!(vec![4, 3, 2, 1, 0], state_key);
        assert_eq!(vec![8, 9, 7, 6, 5], *aco.get_state());
    }
}