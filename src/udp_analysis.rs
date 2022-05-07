use std::collections::HashSet;

use crate::udp_lang::*;
use crate::{Analysis, DidMerge, EGraph, Id, Language};

#[derive(Default, Clone)]
pub struct UAnalysis;

// Metadata for each class
#[derive(Debug, PartialEq, Eq)]
pub struct Data {
    // Set of free variables by their class ID
    pub free: HashSet<Id>,
}

impl Analysis<USr> for UAnalysis {
    type Data = Data;

    fn merge(&mut self, to: &mut Data, from: Data) -> DidMerge {
        let before_len = to.free.len();
        // to.free.extend(from.free);
        to.free.retain(|i| from.free.contains(i));
        DidMerge(before_len != to.free.len(), true)
    }

    fn make(egraph: &EGraph<USr, UAnalysis>, enode: &USr) -> Data {
        let fvs = |i: &Id| egraph[*i].data.free.iter().copied();
        let mut free = HashSet::default();
        match enode {
            USr::Var(v) => {
                free.insert(*v);
            }
            USr::Let([v, a, b]) => {
                free.extend(fvs(b));
                // NOTE only do this if v free in b?
                free.remove(v);
                free.extend(fvs(a));
            }
            USr::Sig([v, a]) => {
                free.extend(fvs(a));
                free.remove(v);
            }
            USr::Other(_, xs) => {
                for x in xs {
                    free.extend(fvs(x));
                }
            }
            _ => enode.for_each(|c| free.extend(&egraph[c].data.free)),
        }

        Data { free }
    }
}