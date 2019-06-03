use std::marker::PhantomData;

use super::FlowTable;

type FT<T> = FlowTable<T>;

pub struct CostCalculator<T: Clone, U: Clone> {
    func: Box<dyn Fn(&mut Vec<U>, &Vec<f64>, &mut Vec<(usize, f64)>, &FT<T>)>,
    // general_cost: f64,
    prev_cost: Vec<f64>,
    prev_infos: Vec<U>,
    data: PhantomData<T>,
    prev_total_cost: f64,
    updated_costs: Vec<(usize, f64)>
}
impl <T, U> CostCalculator<T, U> where T: Clone, U: Clone {
    pub fn new<F>(func: F, len: usize,
        init_cost: f64, init_info: U)
    -> Self where F: Fn(&mut Vec<U>, &Vec<f64>, &mut Vec<(usize, f64)>, &FT<T>) + 'static {
        CostCalculator {
            func: Box::new(func),
            // general_cost: init_value,
            prev_cost: vec![init_cost; len],
            prev_infos: vec![init_info; len],
            prev_total_cost: init_cost * len as f64,
            updated_costs: vec![],
            data: PhantomData
        }
    }
    pub fn compute_cost(&mut self, changed: &FT<T>) -> f64 {
        self.updated_costs.clear();
        (self.func)(&mut self.prev_infos, &self.prev_cost, &mut self.updated_costs, changed);
        for(id, new_cost) in self.updated_costs.iter() {
            self.prev_total_cost += new_cost - self.prev_cost[*id];
            self.prev_cost[*id] = *new_cost;
        }
        self.prev_total_cost
    }
    pub fn get_prev_cost(&self) -> &Vec<f64> {
        &self.prev_cost
    }
    pub fn get_prev_info(&self) -> &Vec<U> {
        &self.prev_infos
    }
}

#[cfg(test)]
mod test {
    use super::CostCalculator;
    use super::super::FlowTable;
    use crate::read_flows_from_file;
    type FT<T> = FlowTable<T>;
    #[test]
    fn test_incremental_calculate() {
        let mut calculator = CostCalculator::new(|infos, _, updated, ft: &FT<usize>| {
            ft.foreach(true, |flow, &k| {
                let id = *flow.id();
                if k > 0 {
                    let new_cost = {
                        if infos[id] {
                            2.0 * k as f64
                        } else {
                            k as f64
                        }
                    };
                    infos[id] = !infos[id];
                    updated.push((id, new_cost));
                }
            });
        }, 5, 0.0, false);
        let flows = read_flows_from_file(0, "flows.json");
        let mut table = FlowTable::<usize>::new();
        table.insert(flows, 0);

        let c = calculator.compute_cost(&table);
        assert_eq!(c, 0.0);
        assert_eq!(&vec![0.0; 5], calculator.get_prev_cost());

        table.update_info(1, 3);
        table.update_info(3, 1);
        let c = calculator.compute_cost(&table);
        assert_eq!(c, 4.0);
        assert_eq!(&vec![0.0, 3.0, 0.0, 1.0, 0.0], calculator.get_prev_cost());
        assert_eq!(&vec![false, true, false, true, false], calculator.get_prev_info());

        table.update_info(4, 9);
        let c = calculator.compute_cost(&table);
        assert_eq!(c, 17.0);
        assert_eq!(&vec![0.0, 6.0, 0.0, 2.0, 9.0], calculator.get_prev_cost());
        assert_eq!(&vec![false, false, false, false, true], calculator.get_prev_info());

        let c = calculator.compute_cost(&table);
        assert_eq!(c, 22.0);
        assert_eq!(&vec![0.0, 3.0, 0.0, 1.0, 18.0], calculator.get_prev_cost());
        assert_eq!(&vec![false, true, false, true, false], calculator.get_prev_info());
    }
}