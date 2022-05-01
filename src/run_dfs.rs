use std::borrow::Cow;
use std::collections::VecDeque;
use std::fmt::{self, Debug, Formatter};
use std::ops::Deref;

use log::*;

use crate::*;

// SearchMatches class without "lifetime" mess.
struct DFSSearchMatches<L: Language> {
    /// The eclass id that these matches were found in.
    pub eclass: Id,
    /// The substitutions for each match.
    pub substs: Vec<Subst>,
    /// Optionally, an ast for the matches used in proof production.
    pub ast: Option<PatternAst<L>>,
}

impl<L: Language> PartialEq<Self> for DFSSearchMatches<L> {
    fn eq(&self, other: &Self) -> bool {
        other.eclass == self.eclass
    }
}

impl<L: Language> Eq for DFSSearchMatches<L> {

}

pub struct DFSScheduler<L: Language> {
    max_depth: usize,
    dfs_stack: Vec::<DFSSearchMatches<L>>,
    visited: Vec<DFSSearchMatches<L>>,
    curr_depth: usize,
    matches: Vec::<DFSSearchMatches<L>>,
    has_been_initialized: bool
}

impl<L: Language> DFSScheduler<L>
{
    /// Set the default maximum DFS depth limit after which DFS will stop.
    /// Default: 1,000
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }


    fn get_dfssearchmatches(&mut self, sm: Vec<SearchMatches<L>>) -> Vec<DFSSearchMatches<L>> {
        let mut dfs_sm = vec![];
        for m in sm {
            let eclass = m.eclass;
            let substs = m.substs;
            let mut ast;
            if m.ast.is_some() {
                ast = Some(m.ast.unwrap().into_owned().clone());
            }  else {
                ast = None;
            }
            dfs_sm.push(DFSSearchMatches {
                eclass,
                substs,
                ast,
            })
        }
        dfs_sm
    }

    fn dfssearchmatch_to_searchmatch<'a>(dfs_m: &DFSSearchMatches<L>) -> SearchMatches<'a, L> {
        let eclass = dfs_m.eclass;
        let substs = dfs_m.substs.clone();

        // if the AST is present, get the value.
        let mut ast;
        if dfs_m.ast.is_some() {
            let ast_clone = dfs_m.ast.clone();
            ast = Some(Cow::Owned(ast_clone.unwrap()));
        } else {
            ast = None;
        }

        SearchMatches {
            eclass,
            substs,
            ast
        }
    }
}

// Default constructor.
impl<L: Language> Default for DFSScheduler<L> {
    fn default() -> Self {
        Self {
            max_depth: 1_000,
            dfs_stack: vec![],
            visited: vec![],
            curr_depth: 0,
            matches: vec![],
            has_been_initialized: false
        }
    }
}

// The *secret sauce*:  add DFS here!
impl<L: Language, N: Analysis<L>> RewriteScheduler<L, N> for DFSScheduler<L>
    where
        L: Language,
        N: Analysis<L>,
{
    // TODO MAKE DFS
    fn can_stop(&mut self, iteration: usize) -> bool {
        self.has_been_initialized && self.dfs_stack.len() == 0
    }

    // after each call, the match is applied.
    // we can take advantage of this to do DFS.  Just return in a DFS pattern, store changes as EGraph is computed.
    fn search_rewrite<'a>(
        &mut self,
        iteration: usize,
        egraph: &EGraph<L, N>,
        rewrite: &'a Rewrite<L, N>,
    ) -> Vec<SearchMatches<'a, L>> {
        // if we have not been initialized, initialize
        if !self.has_been_initialized {
            self.has_been_initialized = true;
        }
        // if we're not at the max_depth, search the egraph + push results to stack
        if self.curr_depth != self.max_depth {
            let mut matches = rewrite.search(egraph);

            let mut dfs_matches = self.get_dfssearchmatches(matches);
            // add the matches to the front of the stack
            dfs_matches.append(&mut self.dfs_stack);
            self.dfs_stack = dfs_matches;
            self.curr_depth += 1;
        } else {
            // while the top of the stack was not in visited, pop it
            while !self.visited.contains(self.dfs_stack.get(0).unwrap()) {
                self.dfs_stack.remove(0);
                self.curr_depth -= 1;
            }
        }

        // pop and return the 1 match from the top of the stack.
        let mut top_of_stack = self.dfs_stack.remove(0);
        let mut top_of_stack_sm = DFSScheduler::<L>::dfssearchmatch_to_searchmatch(&top_of_stack);
        self.visited.push(top_of_stack);
        vec![top_of_stack_sm]
    }

    // WE CAN ABUSE THE CURRENT RUNNER!
    //  store private Stack (Vec) of matches, visited, curr_depth, max_depth, has_been_initialized
    //  each itr:
    //      if max_depth, pop + increment max depth.
    //      compute matches, add them to stack.
    //      increment max depth.
    //      add top of stack to visited.
    //      pop & return top of stack.
    //
    // make can_stop == "is there anything in the Vec of iGraph yet-to-be-explored" && has_been_initialized


    // // TODO MAKE DFS
    // fn search_rewrite<'a>(
    //     &mut self,
    //     iteration: usize,
    //     mut egraph: EGraph<L, N>,
    //     rewrite: &'a Rewrite<L, N>,
    // ) -> Vec<SearchMatches<'a, L>> {
    //     // if we've reached the max_depth, return an empty vector
    //     if iteration == self.max_depth {
    //         vec![]
    //     } else {
    //         let mut matches: Vec::<SearchMatches<'a, L>> = vec![];
    //
    //         // do DFS over the EClasses
    //         egraph.classes_mut().for_each(|class| {
    //             // explore child
    //             let wrapped_child_matches: Option<SearchMatches<L>> = rewrite.searcher.search_eclass(&egraph, class.id);
    //             // if we found any child matches, add to all matches and do do DFS
    //             if wrapped_child_matches.is_some() {
    //                 // unwrap from Option, make into mutable Vec.
    //                 let mut child_matches = vec![wrapped_child_matches.unwrap()];
    //
    //                 // store the matches
    //                 matches.append(&mut child_matches);
    //
    //                 // apply the matches
    //                 rewrite.applier.apply_matches(&mut egraph, &*child_matches, rewrite.name);
    //
    //                 // further do DFS, get more matches
    //                 matches.append(&mut self.search_rewrite(iteration + 1, egraph, rewrite));
    //             }
    //         });
    //         // return all matches
    //         matches
    //     }
    // }

    // for class
    //     get matches
    //     get more matches (does this require apply?)
    //     go deeper
    //
    // THIS CODE IS PROBABLY ALL GARBAGE
}