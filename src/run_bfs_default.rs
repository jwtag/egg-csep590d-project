#![allow(dead_code)]
#![allow(unused_variables)]
use std::cmp::min;

use crate::*;

// this file creates a class which solely uses the default RewriteScheduler methods.

pub struct BFSScheduler {
}

// Default constructor.
impl Default for BFSScheduler {
    fn default() -> Self {
        Self {}
    }
}

impl<L: Language, N: Analysis<L>> RewriteScheduler<L, N> for BFSScheduler
    where
        L: Language,
        N: Analysis<L>,
{
}