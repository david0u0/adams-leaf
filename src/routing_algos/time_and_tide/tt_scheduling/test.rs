use super::super::super::{Flow, FlowTable, GCL};
use super::*;

type Info = Vec<(usize, f64)>;
type FT = FlowTable<Info>;

/**
 * 頻寬皆為100
 *  0--1
 *  |\ |\
 *  | \| \
 *  2--3  4
 *     | /
 *     |/
 *     5
 * 
 * flow0(0->4): route=[0, 1, 4], links = [0, 4]
 * flow1(0->5): route=[0, 3, 5], links = [2, 6]
 * flow2(0->4): route=[0, 3, 5, 4], links = [2, 6, 7]
 * flow3(0->4): route=[0, 2, 3, 5, 4], links = [1, 5, 6, 7]
 */

fn gen_links(ids: Vec<usize>) -> Info {
    ids.into_iter().map(|id| (id, MTU as f64)).collect()
}
fn gen_flow_table() -> FT {
    let mut ft = FlowTable::new();
    ft.insert(vec![
        Flow::TT {
            id: 0, src: 0, dst: 4, size: MTU,
            period: 100, max_delay: 100, offset: 0
        },
        Flow::TT {
            id: 1, src: 0, dst: 5, size: MTU*3,
            period: 150, max_delay: 150, offset: 0
        },
        Flow::TT {
            id: 2, src: 0, dst: 4, size: MTU*2,
            period: 200, max_delay: 200, offset: 0
        },
        Flow::TT {
            id: 3, src: 0, dst: 4, size: MTU*3,
            period: 300, max_delay: 300, offset: 0
        }
    ], vec![]);
    ft.update_info(0, gen_links(vec![0, 4]));
    ft.update_info(1, gen_links(vec![2, 6]));
    ft.update_info(2, gen_links(vec![2, 6, 7]));
    ft.update_info(3, gen_links(vec![1, 5, 6, 7]));
    ft
}

#[test]
fn simple_calculate_offset() {
    let gcl = GCL::new(60, 16);
    let ft = gen_flow_table();
    let flow = ft.get_flow(0);
    let links = ft.get_info(0);
    let a = calculate_offsets(&flow, &vec![], links, &vec![0; 2], &gcl);
    assert_eq!(vec![0.0, 1.0], a);

    let flow = ft.get_flow(2);
    let links = ft.get_info(2);
    let a = calculate_offsets(&flow, &vec![], links, &vec![0; 3], &gcl);
    assert_eq!(vec![0.0, 1.0, 2.0], a);
}
#[test]
fn test_online_schedule() {
    let mut gcl = GCL::new(600, 16);
    let ft = gen_flow_table();

    schedule_fixed_og(&ft, &mut gcl, |_, info| info.clone()).unwrap();
    //schedule_online(&ft, &ft, &mut gcl, |_, info| info);
    let ans: Vec<u32> = vec![0, 1, 2, 3, 4,
        150, 151, 152,
        203, 204,
        300, 301, 302,
        403, 404, 450,
        451, 452];
    let start_times: Vec<_> = gcl.get_gate_events(2).iter().map(|(t, ..)| *t).collect();
    assert_eq!(start_times, ans);
    //panic!("{:?}", gcl.get_gate_events(7));
    //panic!("{:?}", gcl.get_gate_events(6));
}