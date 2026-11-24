//! The ESP-IDF binary: the entry point, the worker pool, the frame pacing and
//! the heap probe.
//!
//! # Responsibilities
//!
//! - Own the process: `esp-idf-sys` binds this binary's `main` to the C
//!   `app_main` the ESP-IDF runtime calls.
//! - Supply the four pieces ESP-IDF needs: the entry point, the one-worker
//!   Rayon pool, the frame budget and the free-heap query.
//!
//! # Design
//!
//! `link_patches` is what makes the binding: it must run before anything else,
//! because it is the patch table the runtime calls into. Returning from `main`
//! hands control back to ESP-IDF with nothing left to run, so a failed run
//! parks: the report goes to the UART and the board does not reboot-loop.
//!
//! Everything else is a [`pill_embedded::Sdk`] implementation over the shared
//! run. A second SDK adds a sibling binary with its own feature; nothing here
//! changes.

// Standard library
use std::thread;
use std::time::Duration;

// External crates
use pill_embedded::Sdk;

// =============================================================================
// Constants
// =============================================================================

/// The frame budget in microseconds: 60 frames a second.
const FRAME_MICROS: u64 = 16_667;

/// The Rayon worker stack in bytes: what fits the module's internal DRAM.
const WORKER_STACK_BYTES: usize = 64 * 1024;

// =============================================================================
// The SDK
// =============================================================================

/// ESP-IDF: the runtime supplies `app_main`, and the allocator answers the heap
/// query.
struct EspIdf;

impl Sdk for EspIdf {
    fn prepare_runtime() {
        // One worker with an explicit stack: Rayon's default would take its
        // worker count from a CPU count the target reports as zero, and its
        // stack size from a desktop default. Building it here keeps anything
        // else from spawning the default pool first.
        let _ = pill_core::rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .stack_size(WORKER_STACK_BYTES)
            .build_global();
    }

    fn pace_frame() {
        thread::sleep(Duration::from_micros(FRAME_MICROS));
    }

    fn minimum_free_heap_bytes() -> Option<usize> {
        // SAFETY: the call only reads the allocator's own low-water statistic.
        // It takes no arguments, cannot fail and touches no memory this code
        // owns.
        let free_bytes = unsafe { esp_idf_sys::esp_get_minimum_free_heap_size() };
        Some(free_bytes as usize)
    }
}

// =============================================================================
// Entry Point
// =============================================================================

/// The ESP-IDF entry point, called by `esp-idf-sys`'s `app_main`.
fn main() {
    esp_idf_sys::link_patches();
    if let Err(report) = pill_embedded::run::<EspIdf>() {
        println!("pill_embedded stopped: {report}");
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    }
}
