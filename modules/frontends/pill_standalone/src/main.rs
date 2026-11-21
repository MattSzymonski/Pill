//! Standalone binary: the engine's desktop frontend, headless or windowed.
//!
//! # Responsibilities
//!
//! - Install the engine report handler and the shared telemetry stack.
//! - Select what to run - the development host (`development`) or the linked project
//!   (`shipping`) - and run it through the windowed or headless loop, whichever
//!   this build compiled.
//! - Convert the run's failure into one styled miette report at the single
//!   reporting boundary.
//!
//! # Design
//!
//! This binary owns its event loop and, in a windowed build, its window. What
//! runs inside the loop is either `pill_development_host`'s development host
//! or `pill_player` running the generated shipping bundle; both are driven
//! through `pill_player::FrameDriver`. A shipping build does not depend on
//! `pill_development_host` at all, so development code cannot reach a shipped
//! binary. There is no GPU code here: the runtime attaches the renderer, and
//! the renderer lives in its own crate.

// Standard library
use std::path::PathBuf;

// External crates
use pill_core::error::{engine_report, install_engine_report_handler};
use pill_core::telemetry::telemetry_target;

// Current crate
use crate::error::RunError;

// =============================================================================
// Modules
// =============================================================================

/// The standalone frontend's errors.
mod error;
/// The headless run loop.
#[cfg(not(feature = "rendering"))]
mod headless_standalone;
/// What this binary runs: the development host, or a shipped project.
mod posture;
/// The windowed run loop and its `winit` application.
#[cfg(feature = "rendering")]
mod windowed_standalone;

// =============================================================================
// Posture
// =============================================================================

// Exactly one posture: without one there is nothing to run, and with both a
// shipping binary would also link the development host, which it never runs.
// A workspace build selects every member's default features, so a `shipping`
// feature requested on the same command line adds the second posture to the
// default one. The pair is refused at compile time, before such a binary can
// be shipped.
#[cfg(not(any(feature = "development", feature = "shipping")))]
compile_error!(
    "pill_standalone needs a posture: `development` (the default) or `shipping` - build a shipped project as `--no-default-features --features shipping`."
);
#[cfg(all(feature = "development", feature = "shipping"))]
compile_error!(
    "pill_standalone cannot be both `development` and `shipping`: build a shipped project as `--no-default-features --features shipping`."
);

// =============================================================================
// Telemetry
// =============================================================================

/// Installs the shared telemetry stack that the rest of the run logs through.
///
/// Terminal logging is always on. When `ECS_LOG_DIR` is set, the log is also
/// written to a rolling file in that directory. With the `profiling` feature,
/// `profile::*` spans go to Tracy through their own filter; `profiling-fine`
/// also sends the fine-grained zones.
///
/// Setup is best-effort: a failure only costs telemetry. It is printed to
/// stderr and does not stop the run.
fn init_telemetry() {
    // Step 1: read the optional log directory from the environment.
    let file_log_directory = std::env::var_os("ECS_LOG_DIR").map(PathBuf::from);
    // Step 2: install the stack, printing a setup failure to stderr.
    if let Err(error) = pill_player::init_telemetry(file_log_directory) {
        // This message cannot go through the logger: it reports that the
        // logger itself failed to install.
        eprintln!("[standalone] telemetry setup failed: {error}");
    }
}

// =============================================================================
// Device Runtime Setup
// =============================================================================

/// Builds the Rayon global pool the device runs on.
///
/// Rayon's default pool takes its worker count from a CPU count the device
/// reports as zero and its stack size from a desktop default. One worker with
/// an explicit stack is what fits the internal DRAM, and building it here
/// keeps anything else from spawning the default pool first.
#[cfg(target_os = "espidf")]
fn configure_device_thread_pool() {
    let _ = pill_core::rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .stack_size(64 * 1024)
        .build_global();
}

// =============================================================================
// Reporting Boundary
// =============================================================================

/// Installs the report handler, runs the project, and turns a failure into a
/// report.
///
/// The telemetry stack is installed before the run loop starts, so a failure is
/// logged to the terminal and, when one is configured, to the log file.
///
/// # Errors
///
/// Returns the configuration failure as a report when the project's settings
/// are missing or invalid. Otherwise, returns the styled [`engine_report`] of
/// the failure that ended the run, after also logging that failure to the
/// tracing output, so it can be matched with the active spans and log files.
fn run_standalone() -> miette::Result<()> {
    // Step 1: install the miette report handler before anything can fail.
    install_engine_report_handler();
    // Step 2: install the telemetry stack (best effort).
    init_telemetry();
    // Step 3: the device's worker pool, before anything can touch Rayon.
    #[cfg(target_os = "espidf")]
    configure_device_thread_pool();
    // Step 3: log the declarations the image carries. `inventory`'s
    // constructors run before `main` on a static build, so they are already
    // here; a device image with none means they were never emitted into
    // `.init_array`, and the project then fails at its first system spawn.
    #[cfg(feature = "shipping")]
    {
        let declarations = pill_engine::declarations::DeclarationBaseline::current();
        pill_core::info!(
            target: telemetry_target::ENGINE,
            components = declarations.components,
            systems = declarations.systems,
            resources = declarations.resources,
            assets = declarations.assets,
            scripts = declarations.scripts,
            value_types = declarations.value_types,
            "declarations registered before the project starts"
        );
        #[cfg(target_os = "espidf")]
        if declarations.components == 0 || declarations.systems == 0 {
            return Err(miette::miette!(
                "the device image registered no components or systems: \
                 inventory's .init_array constructors did not run \
                 (see modules/vendor/inventory/README.pill.md)"
            ));
        }
    }
    // Step 4: get the project this build runs. A development build reads
    // `PROJECT_PATH` and the project's `project_settings.yaml`. A shipping
    // build uses the generated bundle, which
    // `devops/tools/generate_shipping_bundle.py` writes from the same settings
    // file before the build, so this binary never names a project or module
    // itself.
    #[cfg(feature = "development")]
    let project = pill_development_host::HostConfig::from_environment()?;
    #[cfg(feature = "shipping")]
    let project = pill_shipping_bundle::static_project();
    // Step 5: run the project and turn a failure into one report.
    #[cfg(not(feature = "rendering"))]
    let project_run: Result<(), RunError> = headless_standalone::run(project);
    #[cfg(feature = "rendering")]
    let project_run: Result<(), RunError> = windowed_standalone::run(project);

    project_run.map_err(|error| {
        // Also send the fatal failure to the tracing output, so it appears in
        // any active spans and in the log files.
        pill_core::error!(
            target: telemetry_target::ENGINE,
            error = %error,
            "host terminated with an error"
        );
        engine_report(error)
    })
}

// =============================================================================
// Entry Points
// =============================================================================

/// Runs the standalone binary on a desktop.
///
/// The report's `Err` becomes the process exit code through `Termination`.
#[cfg(not(target_os = "espidf"))]
fn main() -> miette::Result<()> {
    run_standalone()
}

/// The device entry point, called by `esp-idf-sys`'s `app_main`.
///
/// `link_patches` is what binds the crate's `main` to the C `app_main` the
/// ESP-IDF runtime calls. Returning from here hands control back to ESP-IDF
/// with nothing left to run, so a failure parks instead: the report goes to
/// the UART and the board does not reboot-loop.
#[cfg(target_os = "espidf")]
fn main() {
    esp_idf_sys::link_patches();
    if let Err(report) = run_standalone() {
        println!("pill_standalone stopped: {report}");
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}
