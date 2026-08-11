wit_bindgen::generate!({
    world: "smoke",
    path: "wit",
});

struct Component;

export!(Component);

impl Guest for Component {
    fn run() -> u32 {
        wit_bindgen::rt::async_support::block_on(async {
            local::m2_async_smoke::host::get().await
        })
    }
}
