//! Callback support infrastructure for C callbacks with Rust closures.
//!
//! This module provides safe abstractions for passing Rust closures to C functions
//! that expect callback function pointers with void* baton parameters.

use std::ffi::c_void;

/// Simple wrapper for passing Rust closures as C callbacks.
///
/// Boxes the closure and provides the pointer as a baton.
pub struct CallbackHandle<F> {
    boxed: Box<F>,
}

impl<F> CallbackHandle<F> {
    /// Create a new callback handle from a closure
    pub fn new(callback: F) -> Self {
        CallbackHandle {
            boxed: Box::new(callback),
        }
    }

    /// Get the baton pointer to pass to C functions
    pub fn baton(&self) -> *mut c_void {
        &*self.boxed as *const F as *mut c_void
    }
}

// Example of how to create extern "C" trampolines for callbacks:
//
// type CancelFn = dyn FnMut() -> bool;
//
// extern "C" fn cancel_trampoline(baton: *mut c_void) -> i32 {
//     if baton.is_null() {
//         return 0;
//     }
//     unsafe {
//         let cancel_fn = &mut *(baton as *mut Box<CancelFn>);
//         if cancel_fn() { 1 } else { 0 }
//     }
// }
//
// The CallbackHandle above can be used to manage the boxed closure lifetime.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_callback_handle() {
        let mut counter = 0;
        let callback = move || {
            counter += 1;
            counter
        };

        let handle = CallbackHandle::new(callback);
        assert!(!handle.baton().is_null());
    }
}
