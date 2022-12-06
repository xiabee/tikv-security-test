// Copyright 2019 TiKV Project Authors. Licensed under Apache-2.0.

//! Profile a part of the code using CPU Profiler from gperftools or Callgrind.
//! Supports Linux and macOS.
//!
//! ## Requirements
//!
//! 1. gperftools
//!
//!    Linux:
//!
//!      You can follow its [INSTALL manual](https://github.com/gperftools/gperftools/blob/master/INSTALL).
//!      Roughly the instructions are the following:
//!
//!      1. Download packages from [release](https://github.com/gperftools/gperftools/releases)
//!      2. Run `./configure`
//!      3. Run `make install`
//!
//!    macOS:
//!
//!      Simply `brew install gperftools`.
//!
//! ## Usage
//!
//! ```ignore
//! profiler::start("./app.profile");
//! some_complex_code();
//! profiler::stop();
//! ```
//!
//! Then, compile the code with `profiling` feature enabled.
//!
//! By default, a profile called `app.profile` will be generated by CPU
//! Profiler. You can then analyze the profile using [pprof](https://github.com/google/pprof).
//!
//! If the application is running in Callgrind, a Callgrind profile dump will be
//! generated instead. Notice that you should run Callgrind with command line
//! option `--instr-atstart=no`, e.g.:
//!
//! ```bash
//! valgrind --tool=callgrind --instr-atstart=no ./my_example
//! ```
//!
//! Also see `examples/prime.rs`.

#[allow(unused_extern_crates)]
extern crate tikv_alloc;

#[cfg(all(unix, feature = "profiling"))]
mod profiler_unix;

#[cfg(all(unix, feature = "profiling"))]
pub use profiler_unix::*;

#[cfg(not(all(unix, feature = "profiling")))]
mod profiler_dummy;

#[cfg(not(all(unix, feature = "profiling")))]
pub use profiler_dummy::*;
