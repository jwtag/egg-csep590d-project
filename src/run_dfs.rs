use std::fmt::{self, Debug, Formatter};

use log::*;

use crate::*;

pub struct DFSBackoffScheduler {
    original_backoff_scheduler: BackoffScheduler
}

// Just call the normal BackoffScheduler methods.
impl DFSBackoffScheduler {
    /// Set the initial match limit after which a rule will be banned.
    /// Default: 1,000
    pub fn with_initial_match_limit(mut self, limit: usize) -> Self {
        self.original_backoff_scheduler = self.original_backoff_scheduler.with_initial_match_limit(limit);
        self
    }

    /// Set the initial ban length.
    /// Default: 5 iterations
    pub fn with_ban_length(mut self, ban_length: usize) -> Self {
        self.original_backoff_scheduler = self.original_backoff_scheduler.with_ban_length(ban_length);
        self
    }

    pub fn rule_stats(&mut self, name: Symbol) -> &mut RuleStats {
        self.original_backoff_scheduler.rule_stats(name)
    }

    /// Never ban a particular rule.
    pub fn do_not_ban(mut self, name: impl Into<Symbol>) -> Self {
        self.original_backoff_scheduler = self.original_backoff_scheduler.do_not_ban(name);
        self
    }

    /// Set the initial match limit for a rule.
    pub fn rule_match_limit(mut self, name: impl Into<Symbol>, limit: usize) -> Self {
        self.original_backoff_scheduler = self.original_backoff_scheduler.rule_match_limit(name, limit);
        self
    }

    /// Set the initial ban length for a rule.
    pub fn rule_ban_length(mut self, name: impl Into<Symbol>, length: usize) -> Self {
        self.original_backoff_scheduler = self.original_backoff_scheduler.rule_ban_length(name, length);
        self
    }
}

// Default constructor.
impl Default for DFSBackoffScheduler {
    fn default() -> Self {
        Self {
            original_backoff_scheduler: BackoffScheduler::default()
        }
    }
}

// The *secret sauce*:  add DFS here!
impl<L, N> RewriteScheduler<L, N> for DFSBackoffScheduler
    where
        L: Language,
        N: Analysis<L>,
{
    // TODO MAKE DFS
    fn can_stop(&mut self, iteration: usize) -> bool {
        let n_stats = self.original_backoff_scheduler.stats.len();

        let mut banned: Vec<_> = self.original_backoff_scheduler
            .stats
            .iter_mut()
            .filter(|(_, s)| s.banned_until > iteration)
            .collect();

        if banned.is_empty() {
            true
        } else {
            let min_ban = banned
                .iter()
                .map(|(_, s)| s.banned_until)
                .min()
                .expect("banned cannot be empty here");

            assert!(min_ban >= iteration);
            let delta = min_ban - iteration;

            let mut unbanned = vec![];
            for (name, s) in &mut banned {
                s.banned_until -= delta;
                if s.banned_until == iteration {
                    unbanned.push(name.as_str());
                }
            }

            assert!(!unbanned.is_empty());
            info!(
                "Banned {}/{}, fast-forwarded by {} to unban {}",
                banned.len(),
                n_stats,
                delta,
                unbanned.join(", "),
            );

            false
        }
    }

    // TODO MAKE DFS
    fn search_rewrite<'a>(
        &mut self,
        iteration: usize,
        egraph: &EGraph<L, N>,
        rewrite: &'a Rewrite<L, N>,
    ) -> Vec<SearchMatches<'a, L>> {
        let stats = self.original_backoff_scheduler.rule_stats(rewrite.name);

        if iteration < stats.banned_until {
            debug!(
                "Skipping {} ({}-{}), banned until {}...",
                rewrite.name, stats.times_applied, stats.times_banned, stats.banned_until,
            );
            return vec![];
        }

        let matches = rewrite.search(egraph);
        let total_len: usize = matches.iter().map(|m| m.substs.len()).sum();
        let threshold = stats.match_limit << stats.times_banned;
        if total_len > threshold {
            let ban_length = stats.ban_length << stats.times_banned;
            stats.times_banned += 1;
            stats.banned_until = iteration + ban_length;
            info!(
                "Banning {} ({}-{}) for {} iters: {} < {}",
                rewrite.name,
                stats.times_applied,
                stats.times_banned,
                ban_length,
                threshold,
                total_len,
            );
            vec![]
        } else {
            stats.times_applied += 1;
            matches
        }
    }
}