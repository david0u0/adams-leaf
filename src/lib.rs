pub mod network_struct;
pub mod algos;
pub mod util;

#[cfg(none)]
mod test {
    use crate::network_struct::Graph;
    use crate::algos::{RO, Dijkstra, RoutingAlgo, Flow, FlowStruct, StreamAwareGraph};
    fn assert_noneorder_vec(mut v1: Vec<Vec<i32>>, mut v2: Vec<Vec<i32>>) {
        v1.sort();
        v2.sort();
        assert_eq!(v1, v2);
    }
    #[test]
    fn test_dijkstra_multi_path1() {
        let mut g = StreamAwareGraph::new();
        g.add_host();
        g.add_host();
        g.add_host();
        g.add_edge((1, 0), 20.0);
        g.add_edge((1, 2), 20.0);
        g.add_edge((0, 2), 10.0);
    
        let flow = Flow::AVB(FlowStruct {
            id: 0, src: 0, dst: 2, size: 100, period: 10, max_delay: 10
        });
        let mut algo = Dijkstra::new(g);
        algo.compute_routes(vec![flow]);
        let v = algo.get_multi_routes(0, 2);
        assert_noneorder_vec(vec![vec![0, 1, 2], vec![0 ,2]], v);
    }
    #[test]
    fn test_dijkstra_multi_path2() {
        let mut g = StreamAwareGraph::new();
        g.add_host();
        g.add_host();
        g.add_host();
        g.add_host();
        g.add_host();
        g.add_host();
        g.add_edge((0, 1), 100.0);
        g.add_edge((0, 2), 20.0/3.0);
        g.add_edge((0, 5), 20.0);
        g.add_edge((4, 5), 20.0);
        g.add_edge((0, 4), 10.0);
        g.add_edge((2, 4), 20.0);
        g.add_edge((2, 3), 20.0);
        g.add_edge((4, 3), 10.0);
        g.add_edge((1, 3), 100.0);
    
        g.inactivate_edge((0, 1));
        
        let flow = Flow::AVB(FlowStruct {
            id: 0, src: 0, dst: 1, size: 100, period: 10, max_delay: 10
        });
        let mut algo = RO::new(g);
        algo.compute_routes(vec![flow]);
        let v = algo.get_multi_routes(0, 1);
        assert_noneorder_vec(v, vec![
            vec![0, 5, 4, 2, 3, 1],
            vec![0, 2, 3, 1],
            vec![0 ,4, 2, 3, 1],
            vec![0 ,4, 3, 1],
            vec![0, 5, 4, 3, 1],
        ]);

    }
}