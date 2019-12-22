use crate::recorder::flow_table::prelude::FlowTable;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OldNew<T: Clone + Eq> {
    Old(T),
    New,
}

pub type OldNewTable<T> = FlowTable<OldNew<T>>;
