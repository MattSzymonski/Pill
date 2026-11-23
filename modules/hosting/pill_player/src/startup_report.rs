//! What a host reports and warms before the first frame.
//!
//! # Responsibilities
//!
//! - Report the machine the game runs on.
//! - Warm the Rayon pool, so the first parallel batch does not pay the cost of
//!   spawning its threads.
//! - Print the parallel-iteration options in use.
//!
//! # Design
//!
//! This is the only module in this crate that differs per target. A desktop
//! has a machine to report, a pool to warm and a terminal to print to; the
//! embedded target builds its own single-worker pool before the engine exists
//! and has none of the rest, so [`report_host_startup`] does nothing there.
//!
//! The order inside the desktop body matters: printing the parallel options
//! reads Rayon's thread count, and reading it builds the default pool when
//! none exists. The pool is therefore warmed first, so the count that gets
//! printed is the one the engine actually runs on.

/// Reports the machine, warms the Rayon pool and prints the parallel options.
///
/// Called once, by [`crate::Runtime::new`], before the engine is built.
#[cfg(not(target_os = "espidf"))]
pub fn report_host_startup() {
    machine_specs::report_machine_specs();
    // Empty work, run once per worker: this is the cheapest call that starts
    // every thread of the pool.
    pill_core::rayon::broadcast(|_worker| {});
    pill_engine::config::print_parallel_config();
}

/// Reports nothing and warms nothing.
///
/// The embedded target builds its own one-worker pool in its frontend, before
/// the engine exists, so a warm-up here would only start the default pool
/// first; and it has no desktop machine to read and no terminal to print the
/// parallel options to.
#[cfg(target_os = "espidf")]
pub fn report_host_startup() {}

/// The hardware report: CPU, memory, storage and OS.
#[cfg(not(target_os = "espidf"))]
mod machine_specs;
