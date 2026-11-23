//! Stage the toolchain's standard-library dylib next to `pill_standalone.exe`.
//!
//! # Responsibilities
//!
//! - Stage the std dylib through the shared `frontends/build_support` staging
//!   code.
//! - Refuse a non-debug build that enables `development`.
//! - Refuse an `embedded` feature that disagrees with the build target, and
//!   publish the ESP-IDF link arguments when the two agree.
//!
//! # Design
//!
//! A development build links with `-C prefer-dynamic`, so this executable
//! imports `std-<hash>.dll` when the process starts. That dylib sits in the
//! toolchain's `lib/rustlib/<host>/lib` directory, which Windows does not
//! search. The copy staged beside the executable is what lets the binary
//! start without the toolchain directory on `PATH`; without it, the process
//! fails with `STATUS_DLL_NOT_FOUND`. The staging does nothing off Windows.
//!
//! A release-lineage profile links everything into one binary and is always
//! the shipping mode, so this script refuses the `development` feature there.
//! The binary itself refuses a build with neither mode or with both.

// The shared staging code every executable's build script includes.
#[path = "../build_support/stage_std_dylib.rs"]
mod stage_std_dylib;

fn main() {
    // The embedded posture is a *program*, not a target: it selects the ESP-IDF
    // entry point, pool and loop. A build script runs on the host, so both
    // sides come from cargo's environment rather than `cfg!`, and a mismatch
    // is refused here, where the message can name both of them.
    let target = std::env::var("TARGET").unwrap_or_default();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let embedded = std::env::var_os("CARGO_FEATURE_EMBEDDED").is_some();
    if embedded && target_os != "espidf" {
        panic!(
            "pill_standalone was built for `{target}` with the `embedded` feature: an embedded \
             image needs an `*-esp-espidf` target. Build it with \
             `--target riscv32imac-esp-espidf`."
        );
    }
    if !embedded && target_os == "espidf" {
        panic!(
            "pill_standalone was built for `{target}` without the `embedded` feature: on an \
             `*-esp-espidf` target the ESP-IDF runtime supplies the entry point. Build it with \
             `--no-default-features --features shipping,embedded`."
        );
    }
    if embedded {
        // `esp-idf-sys` publishes the link arguments the ESP-IDF runtime needs.
        embuild::espidf::sysenv::output();
    }
    // Refused here rather than in the binary, which cannot see the profile. A
    // release-lineage build is always the shipping mode, so the `development`
    // posture is refused there. Cargo passes the profile as `PROFILE` and each
    // enabled feature as `CARGO_FEATURE_*`.
    let profile = std::env::var("PROFILE").unwrap_or_default();
    let development_enabled = std::env::var_os("CARGO_FEATURE_DEVELOPMENT").is_some();
    if profile != "debug" && development_enabled {
        panic!(
            "pill_standalone cannot be built with `development` in the `{profile}` profile: \
             a release-lineage build is always the shipping mode. Build it as \
             `--no-default-features --features shipping`."
        );
    }

    // Copy `std-*.dll` next to the built executable (no-op off Windows).
    stage_std_dylib::stage_std_dylib();
}
