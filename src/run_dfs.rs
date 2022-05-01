#![allow(dead_code)]
#![allow(unused_variables)]
use std::borrow::Cow;

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
        other.eclass == self.eclass && other.substs == self.substs && other.ast == self.ast
    }
}

impl<L: Language> Eq for DFSSearchMatches<L> {

}

pub struct DFSScheduler<L: Language> {
    dfs_stack: Vec<DFSSearchMatches<L>>,
    visited: Vec<DFSSearchMatches<L>>,
    matches: Vec<DFSSearchMatches<L>>,
    has_been_initialized: bool
}

impl<L: Language> DFSScheduler<L>
{
    fn get_dfssearchmatches(&mut self, sm: Vec<SearchMatches<L>>) -> Vec<DFSSearchMatches<L>> {
        let mut dfs_sm = vec![];
        for m in sm {
            let eclass = m.eclass;
            let substs = m.substs;
            let ast;
            if m.ast.is_some() {
                ast = Some(m.ast.unwrap().into_owned().clone());
            }  else {  // handle none case for option
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
        let ast;
        if dfs_m.ast.is_some() {
            let ast_clone = dfs_m.ast.clone();
            ast = Some(Cow::Owned(ast_clone.unwrap()));  // we're extracting from option + wrapping in cow + rewrap in option
        } else {  // handle none case for option
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
            dfs_stack: vec![],
            visited: vec![],
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

        // TODO Adjust so that we're returning on the way down instead of using max_depth.
        //  Add the elements to visited as you go down.
        let mut matches = rewrite.search(egraph);

        if matches.len() > 0 {
            let mut dfs_matches = self.get_dfssearchmatches(matches);
            // add the matches to the front of the stack
            dfs_matches.append(&mut self.dfs_stack);
            self.dfs_stack = dfs_matches;
        } else {
            // while the top of the stack was not in visited, pop it
            while self.dfs_stack.len() > 0 &&
                (self.dfs_stack.get(0).is_none() || !self.visited.contains(self.dfs_stack.get(0).unwrap())) {
                self.dfs_stack.remove(0);
            }
        }

        // if dfs_stack is empty, return empty vector.
        if self.dfs_stack.len() == 0 {
            return vec![];
        }

        // pop and return the 1 match from the top of the stack.
        let top_of_stack = self.dfs_stack.remove(0);
        let top_of_stack_sm = DFSScheduler::<L>::dfssearchmatch_to_searchmatch(&top_of_stack);
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
}