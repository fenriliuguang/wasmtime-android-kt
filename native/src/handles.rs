//! Opaque `jlong` handles for Wasmtime objects (M1).

use jni::sys::jlong;

pub fn to_handle<T>(val: T) -> jlong {
    Box::into_raw(Box::new(val)) as jlong
}

/// # Safety
/// `h` must be a handle from [`to_handle`] for `T`, and still live.
pub unsafe fn from_handle<'a, T>(h: jlong) -> &'a mut T {
    &mut *(h as *mut T)
}

/// # Safety
/// `h` must be a handle from [`to_handle`] for `T`, and not used afterward.
pub unsafe fn drop_handle<T>(h: jlong) {
    if h != 0 {
        drop(Box::from_raw(h as *mut T));
    }
}
