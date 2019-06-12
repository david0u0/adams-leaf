pub struct CostCalculator<U: Clone> {
    // general_cost: f64,
    prev_cost: Vec<f64>,
    prev_infos: Vec<U>,
    total_cost: f64,
}
impl <U: Clone> CostCalculator<U> {
    pub fn new(len: usize, init_cost: f64, init_info: U) -> Self {
        CostCalculator {
            // general_cost: init_value,
            prev_cost: vec![init_cost; len],
            prev_infos: vec![init_info; len],
            total_cost: init_cost * len as f64,
        }
    }
    pub fn get_cost(&self) -> &Vec<f64> {
        &self.prev_cost
    }
    pub fn get_info(&self) -> &Vec<U> {
        &self.prev_infos
    }
    pub fn set_cost(&mut self, index: usize, cost: f64) {
        self.total_cost += cost - self.prev_cost[index];
        self.prev_cost[index] = cost;
    }
    pub fn set_info(&mut self, index: usize, info: U) {
        self.prev_infos[index] = info;
    }
    pub fn get_total_cost(&self) -> f64 {
        self.total_cost
    }
}

#[cfg(test)]
mod test {
    use super::CostCalculator;
    use super::super::FlowTable;
    use crate::read_flows_from_file;
    type FT = FlowTable<usize>;

    fn compute_cost(calc: &mut CostCalculator<bool>, table: &FT) -> f64 {
        table.foreach(true, |flow, &n| {
            if n > 0 {
                let id = *flow.id();
                let info = calc.get_info()[id];
                if info {
                    calc.set_cost(id, n as f64 * 2.0);
                } else {
                    calc.set_cost(id, n as f64);
                }
                calc.set_info(id, !info);
            }
        });
        calc.get_total_cost()
    }

    #[test]
    fn test_incremental_calculate() {
        let mut calculator = CostCalculator::new(5, 0.0, false);
        let flows = read_flows_from_file(0, "flows.json");
        let mut table = FlowTable::<usize>::new();
        table.insert(flows, 0);

        let c = compute_cost(&mut calculator, &table);
        assert_eq!(c, 0.0);
        assert_eq!(&vec![0.0; 5], calculator.get_cost());

        table.update_info(1, 3);
        table.update_info(3, 1);
        let c = compute_cost(&mut calculator, &table);
        assert_eq!(c, 4.0);
        assert_eq!(&vec![0.0, 3.0, 0.0, 1.0, 0.0], calculator.get_cost());
        assert_eq!(&vec![false, true, false, true, false], calculator.get_info());

        table.update_info(4, 9);
        let c = compute_cost(&mut calculator, &table);
        assert_eq!(c, 17.0);
        assert_eq!(&vec![0.0, 6.0, 0.0, 2.0, 9.0], calculator.get_cost());
        assert_eq!(&vec![false, false, false, false, true], calculator.get_info());

        let c = compute_cost(&mut calculator, &table);
        assert_eq!(c, 22.0);
        assert_eq!(&vec![0.0, 3.0, 0.0, 1.0, 18.0], calculator.get_cost());
        assert_eq!(&vec![false, true, false, true, false], calculator.get_info());
    }
}