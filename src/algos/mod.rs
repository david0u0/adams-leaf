macro_rules! build_shared_enum {
    (@dup $enum_name: ident, $( $name: ident {
        $( $field_name: ident: $field_type: ty ),*
    } {
        $( $shared_field_name: ident: $shared_field_type: ty ),*
    } ),*) => {
        #[derive(Clone)]
        pub enum $enum_name {
            $(
                $name {
                    $( $field_name: $field_type, )*
                    $( $shared_field_name: $shared_field_type, )*
                }
            ),*
        }
    };
    (@build_impl $enum_name: ident {
        $( $field_name: ident: $field_type: ty ),*
    }, $tail: tt) => {
        impl $enum_name {
            $(
                build_shared_enum!(@build_fn $enum_name,
                    $field_name, $field_type, $tail);
            )*
        }
    };
    (@build_fn $enum_name: ident, $field_name: ident, $field_type: ty,
        [ $($name: ident),* ]
    ) => {
        pub fn $field_name(&self) -> &$field_type {
            match self {
                $($enum_name::$name { $field_name, .. } => $field_name ),*
            }
        }
    };
    ($enum_name: ident $shared: tt, $( $name: ident $special: tt),* ) => {
        build_shared_enum!(@dup $enum_name, $( $name $special $shared ),*);
        build_shared_enum!(@build_impl $enum_name $shared, [ $( $name ),* ]);
    };
}

#[derive(Clone, Copy)]
pub struct AVBType(bool);
impl AVBType {
    pub fn new_type_a() -> Self {
        return AVBType(true);
    }
    pub fn new_type_b() -> Self {
        return AVBType(false);
    }
    pub fn is_type_a(&self) -> bool {
        return self.0;
    }
    pub fn is_type_b(&self) -> bool {
        return !self.0;
    }
}

build_shared_enum! { 
    Flow {
        id: usize,
        size: u32,
        src: usize,
        dst: usize,
        period: u32,
        max_delay: f64
    },
    AVB {
        avb_type: AVBType
    },
    TT {
        offset: u32
    }
}

pub trait RoutingAlgo {
    fn compute_routes(&mut self, flows: Vec<Flow>);
    fn get_retouted_flows(&self) -> &Vec<usize>;
    fn get_route(&self, id: usize) -> &Vec<usize>;
}

mod helper_struct {
    use super::{Flow};
    pub const MAX_FLOW_ID: usize = 9999;
    #[derive(Clone)]
    pub struct FlowTable<T: Clone> {
        vec: Vec<Option<(Flow, f64, T)>>
    }
    impl <T: Clone> FlowTable<T> {
        pub fn new() -> Self {
            return FlowTable { vec: vec![None; MAX_FLOW_ID] };
        }
        pub fn get_flow(&self, id: usize) -> &Flow {
            if let Some(t) = &self.vec[id] {
                return &t.0;
            }
            panic!("該資料流不存在");
        }
        pub fn get_cost(&self, id: usize) -> f64 {
            if let Some(t) = &self.vec[id] {
                return t.1;
            }
            panic!("該資料流不存在");
        }
        pub fn get_info(&self, id: usize) -> &T {
            if let Some(t) = &self.vec[id] {
                return &t.2;
            }
            panic!("該資料流不存在");
        }
        pub fn delete_flow(&mut self, id: usize) {
            if let Some(_) = &self.vec[id] {
                self.vec[id] = None;
            }
            panic!("該資料流不存在");
        }
        pub fn insert(&mut self, flow: Flow, info: T) {
            let id = *flow.id();
            if let Some(_) = self.vec[id] {
                panic!("插入資料流時發現該資料流已存在");
            }
            self.vec[id] = Some((flow, 0.0, info));
        }
        pub fn update_cost(&mut self, id: usize, cost: f64) {
            if let Some(entry) = &mut self.vec[id] {
                entry.1 = cost;
            } else {
                panic!("更新成本時發現資料流不存在");
            }
        }
        pub fn update_info(&mut self, id: usize, cost: f64, info: T) {
            if let Some(entry) = &mut self.vec[id] {
                entry.1 = cost;
                entry.2 = info;
            } else {
                panic!("更新路徑時發現資料流不存在");
            }
        }
        pub fn foreach_flowtuple(&self, is_avb: bool,
            mut callback: impl FnMut(&mut (Flow, f64, T))
        ) {
            for maybe_flow in self.vec.iter() {
                if let Some(tuple) = maybe_flow {
                    let _tuple = tuple as *const (Flow, f64, T) as *mut (Flow, f64, T);
                    if let Flow::AVB { .. } = tuple.0 {
                        if is_avb {
                            unsafe {
                                callback(&mut *_tuple);
                            }
                        }
                    } else if !is_avb {
                        unsafe {
                            callback(&mut *_tuple);
                        }
                    }
                } else {
                    break;
                }
            }
        }
    }
    
    pub struct GCL {
        hyper_p: usize,
        // TODO 這個資料結構有優化的空間
        vec: Vec<Vec<(usize, usize)>>,
    }
    impl GCL {
        pub fn new(hyper_p: usize, edge_count: usize) -> Self {
            return GCL { vec: vec![vec![]; edge_count], hyper_p };
        }
        /// 回傳 `link_id` 上所有閘門關閉事件。
        /// * `回傳值` - 一個陣列，其內容為 (事件開始時間, 事件持續時間);
        pub fn get_close_event(&self, link_id: usize) -> &Vec<(usize, usize)> {
            assert!(self.vec.len() > link_id, "GCL: 指定了超出範圍的邊");
            return &self.vec[link_id];
        }
        pub fn insert_close_event(&mut self, link_id: usize, start_time: usize, duration: usize) {
            // FIXME: 應該做個二元搜索再插入
            return self.vec[link_id].push((start_time, duration));
        }
    }
}

pub use helper_struct::{GCL, FlowTable};

mod stream_aware_graph;
pub use stream_aware_graph::StreamAwareGraph;

mod shortest_path;
pub use shortest_path::SPF;

mod routing_optimism;
pub use routing_optimism::RO;

mod aco;
pub use aco::ACO;

pub mod time_and_tide;

#[cfg(test)]
mod test {
    build_shared_enum! { 
        TestEnum { a: i32, b: String },
        Test1 { c1: String },
        Test2 { c2: usize, d2: Box<TestEnum> },
        Test3 { }
    }
    #[test]
    fn test_share_enum_macro() {
        let t1 = TestEnum::Test1 {
            a: 1, b: String::from("a"), c1: String::from("B")
        };
        assert_eq!(1, *t1.a());
        assert_eq!(String::from("a"), *t1.b());
        let t2 = TestEnum::Test2 {
            a: 2, b: String::from("gg"), c2: 9, d2: Box::new(t1)
        };
        if let TestEnum::Test2 { d2, .. } = t2 {
            assert_eq!(String::from("a"), *d2.b());
        } else {
            panic!();
        }
        let t3 = TestEnum::Test3 { a: 3, b: String::from("kk") };
        assert_eq!(3, *t3.a());
    }
}