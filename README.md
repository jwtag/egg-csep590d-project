# <img src="doc/egg.svg" alt="egg logo" height="40" align="left"> CSEP590D Project:  EGG with different search algorithms.

[![Crates.io](https://img.shields.io/crates/v/egg.svg)](https://crates.io/crates/egg)
[![Released Docs.rs](https://img.shields.io/crates/v/egg?color=blue&label=docs)](https://docs.rs/egg/)
[![Main branch docs](https://img.shields.io/badge/docs-main-blue)](https://egraphs-good.github.io/egg/egg/)

This repo contains the code for my CSEP590D project which adjusts EGG to work with different algorithms for exploring the search space.

These algorithms are:
- BFS 
- Optimized BFS through BackoffScheduler - default in parent EGG repo, see this paper for in-depth details:  https://arxiv.org/pdf/2111.13040.pdf
- Beam Search
- DFS

###Running different algorithms.

####Running DFS.
DFS is implemented via replacing one of the methods in the Runner class.

To run DFS, do the following in run.rs:
- In the "run()" method, set the loop to call `self.run_one_dfs()` instead of `self.run_one()`.
- In the "new()" method, make the Runner object be constructed with a `BackoffScheduler` in the `scheduler` field.

####Running all non-DFS algorithms.
All other algorithms in this repo are based upon some form of BFS.  They're implemented via different RewriteScheduler objects.

To run any non-DFS algorithm, do the following in run.rs:
- In the "run()" method, set the loop to call `self.run_one()` instead of `self.run_one_dfs()`.
- In the "new()" method, make the Runner object be constructed with the correct scheduler type in the `scheduler` field.  You should choose based upon the following:
  - BFS = BFSScheduler
  - BackoffScheduler
  - Beam Search = BeamScheduler

NOTE:  If you use BeamScheduler, you can adjust the beam width by going into `run_beam.rs` and modifying the `beam_width` variable in the `default()` method.

###Benchmarking with unit tests.

Unit tests are provided in the `tests` directory to test out EGG.
-	transitive- tests based around evaluating transitive boolean operations.
-	lambda - tests based around lambda calculus..
-	math – tests using a language of various mathematical operations.
-	prop – tests using some propositional language.
-	simple – tests using a language with commutative addition and multiplication.
-	udp – tests which evaluate UDP query equivalence using EGG.  Adapted from https://github.com/remysucre/udp/tree/main/src


To run a specific test suite, run:
```
cargo test <suitename>_ --no-fail-fast
```

For example, to run the transitive tests, you'd run:
```
cargo test transitive_ --no-fail-fast
```

The tools used for benchmarking the algorithms can interfere with each other, so you'll want to obtain each metric via its own independent run.

NOTE:  Some of these tests will fail for Beam Search at certain beam_widths.  This is expected since Beam Search may not find the optimal algorithm.  This is because Beam Search has lower resource usage at the expense of thoroughness.  The --no-fail-fast flag allows the test to complete execution instead of failing as soon as one runs.

NOTE2:  To get test-specific (not suite-level) metrics for any of the below benchmarks, substitute the test name for the `<suitename>_` text in any of the below commands.

####Getting the runtimes of the algorithms in ms.

Rust's built-in testing lib does not provide ms-level metrics for test runtimes.  This means that the execution times of some of these tests are reported as 0.0s since some of the tests are so fast.

To complete ms-level measurements, we can modify the built-in Unix `time` command to report ms-level metrics and use that instead.  To do this, first set the following variable in your enviroment:
```
TIMEFMT=$'\n================\nCPU\t%P\nuser\t%mU\nsystem\t%mS\ntotal\t%mE'
```

Then, run the following command:
```
time cargo test --no-fail-fast <suitename>_
```

The results will contain the runtime in ms.

NOTE:  The `time` command will pull-in the time spent for Rust to setup and run the test suite.  This is consistent across all algorithms though, so it does not impact our ability to use these metrics for runtime comparisons.

####Getting the peak memory usage of the algorithms in ms.

Included in this repo is a script called `memusg.sh`.  This script can be used to get the peak memory usage of EGG during the test execution.

To run the script, execute the following command:
```
./memusg.sh cargo test --no-fail-fast <suitename>_
```

The results will contain the peak memory usage in KB.

NOTE:  This script was adapted from https://gist.github.com/netj/526585

####Getting the number of search algorithm executions requried to achieve equality saturation.

To get the number of search algorithm executions required to achieve equality saturation, run the following command:
```
cargo test --no-fail-fast <suitename>_ -- --nocapture | grep "REBUILD COUNT" | grep -Eo '[0-9]' | awk '{ sum += $1; } END { print sum; }' "$@"
```

The number output at the end of the command is the aggregate number of search algorithm executions required to achieve equality saturation.