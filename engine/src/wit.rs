use crate::wit::engine::Engine;

wit_bindgen::generate!("engine");

struct Exports;

impl Engine for Exports {
    fn version() -> String {
        crate::version().to_string()
    }
}

export_engine!(Exports);
