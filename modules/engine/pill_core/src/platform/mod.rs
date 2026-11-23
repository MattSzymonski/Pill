//! The platform layer: the only code in this crate that differs between
//! native, the web and the embedded target.
//!
//! # Responsibilities
//!
//! - Provide the monotonic clock ([`Instant`]) and the wall-clock time.
//! - Provide the log writers: terminal and file on native, the browser
//!   console on the web, the UART alone on the embedded target.
//! - Provide the calls that run a future: block on native, spawn on the web.
//! - Provide the terminal styling implementation for `&str` - colored on
//!   native, plain on the web - and the ANSI switch the log layer reads.
//!
//! # Design
//!
//! This layer uses plain functions and types, not trait objects. The target is
//! fixed when the code is compiled, so there is nothing to pick at run time:
//! the compiler chooses the platform-specific parts.
//!
//! All `cfg(target_arch = "wasm32")` and `cfg(target_os = "espidf")` checks
//! are inside these submodules. Code above this layer therefore reads the
//! same on every target.
//!
//! The workspace `clippy.toml` bans `std::time::Instant` (`disallowed-types`)
//! outside [`clock`]. `Instant::now()` panics on `wasm32-unknown-unknown`, so
//! code that compiles on native would fail only once the game runs in a
//! browser.

/// Monotonic time and wall-clock time.
pub mod clock;
/// Running futures: block on native, spawn on the web.
pub mod futures;
/// Where log output goes: terminal and file on native, the console on the web.
pub mod log_output;
/// Terminal styling: colored and bold on native, plain on the web.
pub mod styling;

/// The monotonic clock, re-exported so callers write `platform::Instant`.
pub use clock::Instant;
