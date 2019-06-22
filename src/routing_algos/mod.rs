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
        #[inline(always)]
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
        size: usize,
        src: usize,
        dst: usize,
        period: u32,
        max_delay: u32
    },
    AVB {
        avb_type: AVBType
    },
    TT {
        offset: u32
    }
}

impl Flow {
    pub fn offset(&self) -> u32 {
        if let &Flow::TT { offset, .. } = self {
            offset
        } else {
            panic!("並非TT資料流卻想取 offset");
        }
    }
    pub fn is_avb(&self) -> bool {
        if let Flow::AVB { .. } = self {
            true
        } else {
            false
        }
    }
    pub fn is_tt(&self) -> bool {
        !self.is_avb()
    }
}

pub trait RoutingAlgo {
    fn add_flows(&mut self, flows: Vec<Flow>);
    fn del_flows(&mut self, flows: Vec<Flow>);
    fn get_retouted_flows(&self) -> &Vec<usize>;
    fn get_route(&self, id: usize) -> &Vec<usize>;
}

mod stream_aware_graph;
pub use stream_aware_graph::StreamAwareGraph;

mod shortest_path;
pub use shortest_path::SPF;

mod routing_optimism;
pub use routing_optimism::RO;

mod recorder;
pub use recorder::{FlowTable, GCL};

pub(self) mod time_and_tide;

mod adams_ant;
pub use adams_ant::AdamsAnt;

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