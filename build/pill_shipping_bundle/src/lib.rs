//! Generated shipping bundle - do not edit. Regenerated from
//! the project's `project_settings.yaml` by
//! `devops/tools/generate_shipping_bundle.py`.
//!
//! # Responsibilities
//!
//! - Link the project, its modules and its renderer into one binary, and
//!   describe them as the `StaticProject` the shipping frontends run.

use pill_player::{
    StaticLogging, StaticModule, StaticProject, StaticProjectBackend,
    StaticRenderer,
};

/// Every selected extension: the renderer's data crate first, then
/// `project_settings.yaml` order.
#[rustfmt::skip]
pub const STATIC_MODULES: &[StaticModule] = &[
];

/// The project backend for this shipping project.
#[rustfmt::skip]
pub fn project_backend() -> StaticProjectBackend {
    StaticProjectBackend::Native {
        register: pill_player::no_project_registration,
        init: headless_rust::init,
    }
}

/// The renderer this binary links, or `None` for a headless build.
pub fn static_renderer() -> Option<StaticRenderer> {
    None
}

/// The `logging:` section of `project_settings.yaml`.
#[rustfmt::skip]
const LOGGING: StaticLogging = StaticLogging::NONE;


/// The complete shipping project: modules first, then the project.
pub fn static_project() -> StaticProject {
    StaticProject {
        name: "Headless",
        backend: project_backend(),
        modules: STATIC_MODULES,
        renderer: static_renderer(),
        asset_pack: None,
        asset_pack_url: None,
        logging: LOGGING,
        debug_draw: false,
        startup_scene: None,
    }
}
