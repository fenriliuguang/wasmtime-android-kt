//! Store host state: Kotlin callbacks + u32-rep widget resources.

use jni::objects::GlobalRef;
use wasmtime::component::ResourceTable;

#[derive(Debug)]
pub struct Widget {
    pub rep: u32,
}

pub struct HostState {
    pub table: ResourceTable,
    pub add_cb: Option<GlobalRef>,
    /// Kotlin `HostU32Supplier` for experimental `request-adapter`.
    pub request_adapter_cb: Option<GlobalRef>,
}

impl Default for HostState {
    fn default() -> Self {
        Self {
            table: ResourceTable::new(),
            add_cb: None,
            request_adapter_cb: None,
        }
    }
}
