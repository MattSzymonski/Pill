//! Runs a game: the engine, its frames, and the statically linked project.
//!
//! # Responsibilities
//!
//! - Owns the [`pill_engine::Engine`] and runs frames ([`Runtime`]).
//! - Defines the static-link contract ([`StaticProject`], [`StaticModule`],
//!   [`StaticRenderer`], [`StaticProjectBackend`], [`ProjectBackend`]) and
//!   registers a linked project in the same order, and under the same owners,
//!   as the hot-reloading host.
//! - Gives every registering part (a module, the renderer or the project) the
//!   owner its ECS systems are recorded under, and runs an entry point with
//!   that owner in effect ([`registration`]). The development host uses the
//!   same owners.
//! - With `rendering`: attaches a renderer to a window passed as raw handles
//!   ([`RendererWindow`], [`attach_renderer`]) and draws frames
//!   ([`AttachedRenderer`], [`RenderingRuntime`]).
//! - Installs the telemetry stack ([`init_telemetry`]), runs the start-up a
//!   game frontend shares ([`install_report_handler`],
//!   [`init_frontend_telemetry`], [`run_failure_report`]) and defines
//!   [`FrameDriver`], the trait a frontend's loop drives.
//!
//! # Design
//!
//! Linked into every executable: the development host
//! (`pill_development_host`) builds on it, and a shipping build and a web
//! build use it without the host. It therefore holds only what a shipped game
//! needs: no DLL loading, file watching, build tools, .NET hosting, winit or
//! YAML. A C# project reaches it as an external [`ProjectBackend`] that
//! `pill_csharp_bridge` implements.

// =============================================================================
// Module Declarations
// =============================================================================

/// Errors of a windowed runtime.
#[cfg(feature = "rendering")]
mod error;
/// Which owner a module, the renderer or the project registers its ECS
/// systems under.
pub mod registration;
/// Attaching a renderer to a frontend's window, through its handles as data.
#[cfg(feature = "rendering")]
mod render_window;
/// A renderer attached to a window, and the runtime that draws a statically
/// linked project on it.
#[cfg(feature = "rendering")]
mod rendering;
/// Engine ownership and the frame every build runs.
mod runtime;
/// What a host reports and warms before the first frame: the machine report,
/// the pool warm-up and the parallel options. The only module in this crate
/// that differs per target.
mod startup_report;
/// Statically linked project and module registration.
mod static_project;
/// Application telemetry bootstrap for every frontend.
mod telemetry;

// =============================================================================
// Re-exports
// =============================================================================

/// The engine's transform component, re-exported so a frontend names this
/// crate alone.
pub use pill_engine::common_components::TransformComponent;

/// The declaration counts an image registers before its project starts, as
/// [`report_declarations`] returns them.
pub use pill_engine::declarations::DeclarationBaseline;

/// The renderer contract's types that appear in this crate's API: surfaces,
/// cameras, picking, previews and the scene grid.
pub use pill_renderer_api::{
    editor_view_surfaces, ensure_preview_resources, frame_grid, CameraBinding, CameraComponent,
    PickHit, PickRequests, PickResults, PresentPreference, PreviewAsset, PreviewImage,
    PreviewRequests, PreviewResults, RenderViewport, RendererError, SceneGrid, SelectionOutlines,
    SurfaceCameras, SurfaceId, SurfaceOptions,
};

/// Engine start-up, the asset pack a shipped build mounts, the resources and
/// camera bindings a frame reads, and the frame driver - [`FrameDriver`] and,
/// windowed, [`WindowedFrameDriver`].
pub use runtime::{
    insert_debug_resources, mount_asset_pack, set_surface_camera, setup, setup_with_asset_database,
    FrameDriver, PackSource, Runtime,
};

/// What a driver that draws through a renderer adds to [`FrameDriver`].
#[cfg(feature = "rendering")]
pub use runtime::WindowedFrameDriver;

/// The static-link contract a shipping bundle passes in.
pub use static_project::{
    no_project_registration, report_declarations, require_declarations, ProjectBackend,
    StaticLogging, StaticModule, StaticProject, StaticProjectBackend, StaticRenderer,
    StaticRendererAttachFn,
};

/// The telemetry stack, the logging settings a project applies to it, failure
/// reporting, and the start-up a game frontend shares.
pub use telemetry::{
    apply_logging_settings, init_frontend_telemetry, init_telemetry, init_telemetry_with_capture,
    install_report_handler, report_failure, run_failure_report, LoggingSettings,
};

/// The windowed runtime's error.
#[cfg(feature = "rendering")]
pub use error::RenderingError;

/// The bound a frontend's window type meets to take a renderer.
#[cfg(feature = "rendering")]
pub use render_window::RendererWindow;

/// Attaching a renderer to a window, and the runtime that draws a statically
/// linked project on it.
#[cfg(feature = "rendering")]
pub use rendering::{attach_renderer, setup_rendering, AttachedRenderer, RenderingRuntime};
