//! The headless run loop: no window and no renderer.
//!
//! # Responsibilities
//!
//! - Run the project one frame after another until the process exits.
//! - Report setup failures, the only failure a headless run can have.
//!
//! # Design
//!
//! The loop is written against [`FrameDriver`], so it runs whichever driver
//! [`crate::posture`] compiled. A headless frame cannot fail, so [`RunError`]
//! converts the driver's `Infallible` error, and the loop never returns.
//!
//! This file is compiled only in a headless build: `main.rs` declares it
//! instead of `windowed_standalone` while the `rendering` feature is off.

// External crates
#[cfg(feature = "embedded")]
use pill_core::telemetry::telemetry_target;
use pill_player::FrameDriver;

// Current crate
use crate::error::RunError;
use crate::posture;

// =============================================================================
// Constants
// =============================================================================

/// The embedded frame budget in microseconds: 60 frames a second. Sleeping
/// between frames is also what lets the idle task run, and the idle task is
/// what feeds the task watchdog.
#[cfg(feature = "embedded")]
const EMBEDDED_FRAME_MICROS: u64 = 16_667;

/// Frames between heap high-water reports: about five seconds at the embedded
/// target's frame rate.
#[cfg(feature = "embedded")]
const EMBEDDED_HEAP_REPORT_FRAMES: u32 = 300;

// =============================================================================
// Free Functions
// =============================================================================

/// Runs the project continuously without creating a native window.
///
/// Setup is the only step that can fail: a headless driver's frame cannot
/// fail, so once the driver exists the loop runs until the process exits.
///
/// # Errors
///
/// Returns [`RunError`] if setup fails, such as when the project cannot be
/// built, loaded or initialized.
pub fn run(project: posture::ProjectSource) -> Result<(), RunError> {
    // Step 1: set up the driver this build's posture compiled.
    let mut driver = posture::setup_driver(project)?;
    // Step 2: run a frame at a time. Only setup can fail, so the loop never
    // ends on its own.
    #[cfg(feature = "embedded")]
    let mut frames_since_heap_report: u32 = 0;
    loop {
        driver.run_frame()?;
        // The embedded target has no desktop event loop to pace it: without
        // this the main task never yields, the idle task never runs, and the
        // task watchdog resets the board after a few seconds.
        #[cfg(feature = "embedded")]
        {
            std::thread::sleep(std::time::Duration::from_micros(EMBEDDED_FRAME_MICROS));
            frames_since_heap_report += 1;
            if frames_since_heap_report >= EMBEDDED_HEAP_REPORT_FRAMES {
                frames_since_heap_report = 0;
                // SAFETY: the call only reads the allocator's own low-water
                // statistic. It takes no arguments, cannot fail and touches
                // no memory this code owns.
                let free_bytes = unsafe { esp_idf_sys::esp_get_minimum_free_heap_size() };
                pill_core::info!(
                    target: telemetry_target::SYSTEM,
                    free_bytes = free_bytes,
                    "embedded heap high-water"
                );
            }
        }
    }
}
