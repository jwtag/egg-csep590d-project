#![allow(dead_code)]
#![allow(unused_variables)]
use std::cmp::min;

use crate::*;

pub struct BeamScheduler {
    beam_width: usize
}

// Default constructor.
impl Default for BeamScheduler {
    fn default() -> Self {
        Self {
            beam_width: 100
        }
    }
}

// The *secret sauce*:  add DFS here!
impl<L: Language, N: Analysis<L>> RewriteScheduler<L, N> for BeamScheduler
    where
        L: Language,
        N: Analysis<L>,
{
    // always can stop.
    fn can_stop(&mut self, iteration: usize) -> bool {
        true
    }

    // after each call, the match is applied.
    fn search_rewrite<'a>(
        &mut self,
        iteration: usize,
        egraph: &EGraph<L, N>,
        rewrite: &'a Rewrite<L, N>,
    ) -> Vec<SearchMatches<'a, L>> {
        let mut matches: Vec<SearchMatches<'a, L>> = rewrite.search(egraph);

        // scoot the matches with the fewest substitutes to the front
        matches.sort_by(|a, b| a.substs.len().cmp(&b.substs.len()));

        // get the new size of the array (based upon beam size)
        let vec_len = min(self.beam_width, matches.len());
        unsafe { matches.set_len(vec_len); }
        matches
    }
}