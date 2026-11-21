//! Where log output goes on every target.
//!
//! # Responsibilities
//!
//! - Provide the terminal writer: stdout on native, the browser console on the
//!   web.
//! - Start the file writer: on a desktop, a log that goes to a new file each
//!   day, written by a background thread. The web and the device have no log
//!   file, so they report an error.
//!
//! # Design
//!
//! Only this module chooses the writer for each target.
//! [`TelemetryBuilder`](crate::telemetry::TelemetryBuilder) builds the
//! subscriber from these items, so its own code is the same everywhere.

// Standard library
use std::path::Path;

// External crates
#[cfg(target_arch = "wasm32")]
use tracing::{Level, Metadata};
#[cfg(target_arch = "wasm32")]
use tracing_subscriber::fmt::MakeWriter;

// =============================================================================
// Types
// =============================================================================

/// The terminal writer on native: stdout.
#[cfg(not(target_arch = "wasm32"))]
pub type TerminalWriter = fn() -> std::io::Stdout;

/// The terminal writer on the web: the browser console.
#[cfg(target_arch = "wasm32")]
pub type TerminalWriter = ConsoleWriter;

/// The file writer on a target with a writable filesystem: a non-blocking
/// writer over a log that is split into one file per day.
#[cfg(not(any(target_arch = "wasm32", target_os = "espidf")))]
pub type FileWriter = tracing_appender::non_blocking::NonBlocking;

/// The file writer where there is no log file to write: the web and the
/// device both discard every line.
#[cfg(any(target_arch = "wasm32", target_os = "espidf"))]
pub type FileWriter = fn() -> std::io::Sink;

/// Keeps the file writer's background thread alive. Dropping it writes out the
/// lines still waiting in the buffer.
#[cfg(not(any(target_arch = "wasm32", target_os = "espidf")))]
pub type FileGuard = tracing_appender::non_blocking::WorkerGuard;

/// Without a file writer there is nothing to keep alive.
#[cfg(any(target_arch = "wasm32", target_os = "espidf"))]
pub type FileGuard = ();

/// The browser console, used as the terminal layer's `tracing` writer.
///
/// Every event becomes one console message, at the console level that matches
/// the event's severity.
#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, Default)]
pub struct ConsoleWriter;

#[cfg(target_arch = "wasm32")]
impl<'writer> MakeWriter<'writer> for ConsoleWriter {
    type Writer = ConsoleLine;

    // The writer used when an event arrives without metadata. Such lines are
    // logged at INFO.
    fn make_writer(&'writer self) -> Self::Writer {
        ConsoleLine::new(Level::INFO)
    }

    fn make_writer_for(&'writer self, metadata: &Metadata<'_>) -> Self::Writer {
        ConsoleLine::new(*metadata.level())
    }
}

/// One formatted event on its way to the browser console.
///
/// The formatter writes an event in pieces. The pieces are collected here and
/// sent as one console message when the value is dropped.
#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub struct ConsoleLine {
    level: Level,
    buffer: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
impl ConsoleLine {
    fn new(level: Level) -> Self {
        Self {
            level,
            buffer: Vec::new(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl std::io::Write for ConsoleLine {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl Drop for ConsoleLine {
    // Send the whole event, without the formatter's trailing newline, through
    // the console method of its severity so the browser can filter by level.
    fn drop(&mut self) {
        let text = String::from_utf8_lossy(&self.buffer);
        let message = wasm_bindgen::JsValue::from_str(text.trim_end());
        match self.level {
            Level::ERROR => web_sys::console::error_1(&message),
            Level::WARN => web_sys::console::warn_1(&message),
            Level::INFO => web_sys::console::info_1(&message),
            Level::DEBUG | Level::TRACE => web_sys::console::debug_1(&message),
        }
    }
}

// =============================================================================
// Free Functions
// =============================================================================

/// Returns stdout as the terminal writer.
#[cfg(not(target_arch = "wasm32"))]
pub fn terminal_writer() -> TerminalWriter {
    std::io::stdout
}

/// Returns the browser console as the terminal writer.
#[cfg(target_arch = "wasm32")]
pub fn terminal_writer() -> TerminalWriter {
    ConsoleWriter
}

/// Opens the log file in `directory` and returns its writer and guard. Keep
/// the guard alive for as long as the writer is used.
///
/// Each day's file is named `<file_name>.<YYYY-MM-DD>`, so `file_name` is the
/// file name prefix, not the whole name.
///
/// # Errors
///
/// Returns the failure message when the directory or the file cannot be
/// created.
#[cfg(not(any(target_arch = "wasm32", target_os = "espidf")))]
pub fn open_file_output(
    directory: &Path,
    file_name: &str,
) -> Result<(FileWriter, FileGuard), String> {
    let appender = tracing_appender::rolling::RollingFileAppender::builder()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix(file_name)
        .build(directory)
        .map_err(|error| {
            format!(
                "cannot open log file `{file_name}` in `{}`: {error}",
                directory.display()
            )
        })?;
    Ok(tracing_appender::non_blocking(appender))
}

/// Opens the file writer, which targets without a writable filesystem lack.
///
/// # Errors
///
/// Always returns an error: the web and the device have no log file to write.
#[cfg(any(target_arch = "wasm32", target_os = "espidf"))]
pub fn open_file_output(
    directory: &Path,
    file_name: &str,
) -> Result<(FileWriter, FileGuard), String> {
    Err(format!(
        "cannot write log file `{file_name}` to `{}`: this target has no writable filesystem",
        directory.display()
    ))
}
