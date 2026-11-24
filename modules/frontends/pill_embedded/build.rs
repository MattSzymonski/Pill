//! Publish the ESP-IDF link arguments, and refuse a feature/target mismatch.
//!
//! # Responsibilities
//!
//! - Refuse the `espidf` feature on a target that is not an ESP-IDF target, and
//!   an ESP-IDF target without the feature, each with a message naming both.
//! - Forward the link arguments `esp-idf-sys` publishes when the two agree.
//!
//! # Design
//!
//! A build script runs on the host, so both sides come from cargo's environment
//! rather than from `cfg!`, and a mismatch is refused here, where the message can
//! name the target and the feature. The crate has one binary per SDK, so the
//! guard is what keeps `--features espidf` off a build that was not aimed at an
//! ESP-IDF target.

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let espidf = std::env::var_os("CARGO_FEATURE_ESPIDF").is_some();
    if espidf && target_os != "espidf" {
        panic!(
            "pill_embedded was built for `{target}` with the `espidf` feature: this SDK \
             needs an `*-esp-espidf` target. Build it with `--target riscv32imac-esp-espidf`."
        );
    }
    if !espidf && target_os == "espidf" {
        panic!(
            "pill_embedded was built for `{target}` without an SDK feature: on an ESP-IDF \
             target the runtime supplies the entry point. Build it with \
             `--no-default-features --features espidf`."
        );
    }
    // The optional build-dependency is only compiled in with its feature, so the
    // call is gated rather than the branch.
    #[cfg(feature = "espidf")]
    if espidf {
        // `esp-idf-sys` publishes the link arguments the ESP-IDF runtime needs.
        embuild::espidf::sysenv::output();
    }
}
