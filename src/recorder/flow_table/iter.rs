use crate::flow::{Flow, FlowID};

pub enum Iter<'a, D: Clone, T: Clone> {
    FlowTable {
        ptr: usize,
        v: &'a Vec<Option<Flow<D>>>,
        infos: &'a Vec<Option<T>>,
    },
    DiffTable {
        ptr: usize,
        v: &'a Vec<Option<Flow<D>>>,
        diff: &'a Vec<FlowID>,
        pos_list: &'a Vec<Option<usize>>,
        infos: &'a Vec<Option<T>>,
    },
}

impl<'a, D: Clone, T: Clone> Iterator for Iter<'a, D, T> {
    type Item = (&'a Flow<D>, &'a T);
    fn next(&mut self) -> Option<(&'a Flow<D>, &'a T)> {
        match self {
            Iter::FlowTable {
                ref mut ptr,
                v,
                infos,
            } => {
                while *ptr < v.len() {
                    let flow_opt = v[*ptr].as_ref();
                    *ptr += 1;
                    if let Some(flow) = flow_opt {
                        let info_opt = infos[flow.id.0].as_ref();
                        return Some((flow, info_opt.unwrap()));
                    }
                }
            }
            Iter::DiffTable {
                ref mut ptr,
                v,
                diff,
                infos,
                pos_list,
            } => {
                if *ptr < diff.len() {
                    let id = diff[*ptr];
                    let pos = pos_list[id.0].as_ref().unwrap();
                    *ptr += 1;
                    return Some((v[*pos].as_ref().unwrap(), infos[id.0].as_ref().unwrap()));
                }
            }
        }
        None
    }
}

pub struct IterMut<'a, D: Clone, T: Clone> {
    pub(super) iter: Iter<'a, D, T>,
}

impl<'a, D: Clone, T: Clone> Iterator for IterMut<'a, D, T> {
    type Item = (&'a Flow<D>, &'a mut T);
    fn next(&mut self) -> Option<(&'a Flow<D>, &'a mut T)> {
        self.iter
            .next()
            .map(|(flow, t)| unsafe { (flow, &mut *(t as *const T as *mut T)) })
    }
}
