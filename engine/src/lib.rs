#![doc = include_str!("../../README.md")]
//#![deny(warnings)]
//#![warn(missing_docs)]
wit_bindgen::generate!("component");

#[cfg(not(target_arch = "wasm32"))]
pub mod client;
mod core;
pub mod data;
pub mod lang;
mod package;
pub mod runtime;
pub mod value;

/// Common test functionality
#[doc(hidden)]
pub mod test;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const TAG: Option<&str> = option_env!("TAG");

/// Current version of Seedwing
pub const fn version() -> &'static str {
    if let Some(tag) = TAG {
        tag
    } else {
        VERSION
    }
}

struct Something;

impl Component for Something {
    fn something(s: String) -> String {
        format!("something was passed: {s}")
    }
}

export_component!(Something);
