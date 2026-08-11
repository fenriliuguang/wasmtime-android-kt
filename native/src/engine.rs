//! Shared Engine construction (CM async enabled for M1+ / M2).

use wasmtime::{Config, Engine};

pub fn new_engine() -> Result<Engine, String> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    // concurrency_support defaults to true; required by FutureReader / run_concurrent.
    Engine::new(&config).map_err(|e| e.to_string())
}
