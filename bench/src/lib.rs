//! # VTBench
//!
//! A catch-all harness for microbenchmarks that do not belong to any other library
//! in this repository. This is to enable testing the performance of VileTech-relevant
//! functions, patterns, and techniques not just in isolation but also over time,
//! between compiler versions/configurations, and across different platforms.
//!
//! These benchmarks naturally cannot be used to accurately gauge the performance
//! of an entire system from end to end but they are also the only reasonable starting point.

pub extern crate util;
