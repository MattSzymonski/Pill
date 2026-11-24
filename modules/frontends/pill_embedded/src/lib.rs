//! The embedded frontend: a shipped project running headless on a bare board.
//!
//! # Responsibilities
//!
//! - Run a statically linked project headless on an embedded target: the shared
//!   start-up, the declaration check, the project setup and the frame loop.
//! - Define [`Sdk`], the pieces each SDK's binary supplies: the worker pool, the
//!   frame pacing and the heap probe. The entry point stays in the binary,
//!   because each SDK's runtime binds to it.
//! - Report the free heap's low-water mark every few seconds.
//!
//! # Design
//!
//! One binary per SDK - `src/bin/espidf_target.rs` for the ESP-IDF runtime - each gated
//! on its SDK feature with `required-features`, so the SDK a build runs is a
//! property of the binary rather than a feature on a desktop frontend.
//!
//! The crate is its own workspace, excluded from the host workspace, because its
//! only code needs an embedded SDK's crate and a host build cannot resolve one.
//! It builds no development posture at all: a board has no hot reload, so
//! "shipping only" is the dependency graph rather than a check.
//!
//! Everything shared with the desktop frontend comes from `pill_player`: the
//! report handler, the telemetry stack, the declaration baseline and the one
//! report a failure becomes. What differs per SDK is the four pieces of
//! [`Sdk`] and the entry point next to it.

// External crates
use pill_core::telemetry::telemetry_target;
use pill_player::FrameDriver;

// =============================================================================
// Constants
// =============================================================================

/// Frames between heap high-water reports: about five seconds at the 60-frame
/// budget the `espidf` binary paces to.
const HEAP_REPORT_FRAMES: u32 = 300;

// =============================================================================
// The SDK contract
// =============================================================================

/// The pieces of one embedded SDK that the shared run needs.
///
/// An implementation is a unit type beside the SDK's `main` - the entry point
/// itself cannot live here, because the SDK's runtime binds to it.
pub trait Sdk {
    /// Prepares the runtime before anything touches it.
    ///
    /// Rayon's default pool takes its worker count from a CPU count the target
    /// reports as zero and its stack size from a desktop default, so an
    /// embedded frontend builds its own pool first.
    fn prepare_runtime();

    /// Waits out the rest of this frame's budget.
    ///
    /// A board has no desktop event loop to pace it: without a sleep the main
    /// task never yields, the idle task never runs, and the task watchdog
    /// resets the board after a few seconds.
    fn pace_frame();

    /// The free heap's low-water mark in bytes, or `None` when the SDK cannot
    /// measure it.
    fn minimum_free_heap_bytes() -> Option<usize>;
}

// =============================================================================
// The shared run
// =============================================================================

/// Runs the linked project headless until the process ends.
///
/// The start-up is the one every game frontend runs, from `pill_player`; what
/// this frontend adds is the SDK's worker pool, the declaration check that only
/// a shipped image needs, and the frame loop with the heap report.
///
/// # Errors
///
/// Returns one report when the image registered no components or systems, or
/// when the project cannot be set up. A headless frame cannot fail, so once the
/// driver exists the loop runs until the process is stopped.
pub fn run<S: Sdk>() -> miette::Result<()> {
    // Step 1: install the report handler before anything can fail.
    pill_player::install_report_handler();
    // Step 2: install the telemetry stack (best effort).
    if let Err(error) = pill_player::init_frontend_telemetry() {
        // This message cannot go through the logger: it reports that the logger
        // itself failed to install.
        eprintln!("[embedded] telemetry setup failed: {error}");
    }
    // Step 3: the SDK's worker pool, before anything can touch Rayon.
    S::prepare_runtime();
    // Step 4: refuse an image that registered nothing. `inventory`'s
    // constructors run before `main`, so a shipped image carries them here; an
    // image with none never ran them, and its project would fail at the first
    // system spawn instead.
    pill_player::require_declarations()?;
    // Step 5: set the project up. The bundle knows the project and its
    // extensions, so this binary names neither.
    let mut driver = pill_player::setup(pill_shipping_bundle::static_project())
        .map_err(pill_player::run_failure_report)?;
    // Step 6: run a frame at a time. Only setup can fail, so the loop never
    // ends on its own.
    let mut frames_since_heap_report: u32 = 0;
    loop {
        match driver.run_frame() {
            Ok(()) => {}
            Err(never) => match never {},
        }
        S::pace_frame();
        frames_since_heap_report += 1;
        if frames_since_heap_report >= HEAP_REPORT_FRAMES {
            frames_since_heap_report = 0;
            if let Some(free_bytes) = S::minimum_free_heap_bytes() {
                pill_core::info!(
                    target: telemetry_target::SYSTEM,
                    free_bytes = free_bytes,
                    "embedded heap high-water"
                );
            }
        }
    }
}
