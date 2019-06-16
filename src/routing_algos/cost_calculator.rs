pub struct CostCalculator {
    // general_cost: f64,
    prev_cost: Vec<f64>,
    total_cost: f64,
}
impl  CostCalculator {
    pub fn new(len: usize, init_cost: f64) -> Self {
        CostCalculator {
            // general_cost: init_value,
            prev_cost: vec![init_cost; len],
            total_cost: init_cost * len as f64,
        }
    }
    pub fn get_cost(&self) -> &Vec<f64> {
        &self.prev_cost
    }
    pub fn set_cost(&mut self, index: usize, cost: f64) {
        self.total_cost += cost - self.prev_cost[index];
        self.prev_cost[index] = cost;
    }
    pub fn get_total_cost(&self) -> f64 {
        self.total_cost
    }
}