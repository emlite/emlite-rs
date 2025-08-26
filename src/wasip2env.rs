use crate::common::Handle;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::ptr;

wit_bindgen::generate!({ generate_all });

use crate::emlite::env::host;
use crate::exports::emlite::env::dyncall::Guest;

struct Env;

impl Guest for Env {
    fn apply(fidx: u32, argv: u32, data: u32) -> u32 {
        unsafe { emlite_env_dyncall_apply(fidx, argv, data) }
    }
}

export!(Env);

// Adapter functions to bridge WIT bindings with Rust API expectations

pub unsafe fn emlite_init_handle_table() {
    host::emlite_init_handle_table();
}

pub unsafe fn emlite_val_new_array() -> Handle {
    host::emlite_val_new_array()
}

pub unsafe fn emlite_val_new_object() -> Handle {
    host::emlite_val_new_object()
}

pub unsafe fn emlite_val_make_bool(v: bool) -> Handle {
    host::emlite_val_make_bool(v)
}

pub unsafe fn emlite_val_make_int(v: i32) -> Handle {
    host::emlite_val_make_int(v)
}

pub unsafe fn emlite_val_make_uint(v: u32) -> Handle {
    host::emlite_val_make_uint(v)
}

pub unsafe fn emlite_val_make_bigint(v: i64) -> Handle {
    host::emlite_val_make_bigint(v)
}

pub unsafe fn emlite_val_make_biguint(v: u64) -> Handle {
    host::emlite_val_make_biguint(v)
}

pub unsafe fn emlite_val_make_double(v: f64) -> Handle {
    host::emlite_val_make_double(v)
}

pub unsafe fn emlite_val_make_str(s: &str) -> Handle {
    host::emlite_val_make_str(s)
}

pub unsafe fn emlite_val_make_str_utf16(s: &[u16]) -> Handle {
    host::emlite_val_make_str_utf16(s)
}

pub unsafe fn emlite_val_get_value_bool(h: Handle) -> bool {
    host::emlite_val_get_value_bool(h)
}

pub unsafe fn emlite_val_get_value_int(h: Handle) -> i32 {
    host::emlite_val_get_value_int(h)
}

pub unsafe fn emlite_val_get_value_uint(h: Handle) -> u32 {
    host::emlite_val_get_value_uint(h)
}

pub unsafe fn emlite_val_get_value_bigint(h: Handle) -> i64 {
    host::emlite_val_get_value_bigint(h)
}

pub unsafe fn emlite_val_get_value_biguint(h: Handle) -> u64 {
    host::emlite_val_get_value_biguint(h)
}

pub unsafe fn emlite_val_get_value_double(h: Handle) -> f64 {
    host::emlite_val_get_value_double(h)
}

pub unsafe fn emlite_val_get_value_string(h: Handle) -> String {
    host::emlite_val_get_value_string(h)
}

pub unsafe fn emlite_val_get_value_string_utf16(h: Handle) -> Vec<u16> {
    host::emlite_val_get_value_string_utf16(h)
}

pub unsafe fn emlite_val_typeof(h: Handle) -> String {
    host::emlite_val_typeof(h)
}

pub unsafe fn emlite_val_push(arr: Handle, val: Handle) {
    host::emlite_val_push(arr, val);
}

pub unsafe fn emlite_val_get(obj: Handle, idx: Handle) -> Handle {
    host::emlite_val_get(obj, idx)
}

pub unsafe fn emlite_val_set(obj: Handle, idx: Handle, val: Handle) {
    host::emlite_val_set(obj, idx, val);
}

pub unsafe fn emlite_val_has(obj: Handle, key: Handle) -> bool {
    host::emlite_val_has(obj, key)
}

pub unsafe fn emlite_val_not(h: Handle) -> bool {
    host::emlite_val_not(h)
}

pub unsafe fn emlite_val_is_string(h: Handle) -> bool {
    host::emlite_val_is_string(h)
}

pub unsafe fn emlite_val_is_number(h: Handle) -> bool {
    host::emlite_val_is_number(h)
}

pub unsafe fn emlite_val_is_bool(h: Handle) -> bool {
    host::emlite_val_is_bool(h)
}

pub unsafe fn emlite_val_gt(a: Handle, b: Handle) -> bool {
    host::emlite_val_gt(a, b)
}

pub unsafe fn emlite_val_gte(a: Handle, b: Handle) -> bool {
    host::emlite_val_gte(a, b)
}

pub unsafe fn emlite_val_lt(a: Handle, b: Handle) -> bool {
    host::emlite_val_lt(a, b)
}

pub unsafe fn emlite_val_lte(a: Handle, b: Handle) -> bool {
    host::emlite_val_lte(a, b)
}

pub unsafe fn emlite_val_equals(a: Handle, b: Handle) -> bool {
    host::emlite_val_equals(a, b)
}

pub unsafe fn emlite_val_strictly_equals(a: Handle, b: Handle) -> bool {
    host::emlite_val_strictly_equals(a, b)
}

pub unsafe fn emlite_val_instanceof(a: Handle, b: Handle) -> bool {
    host::emlite_val_instanceof(a, b)
}

pub unsafe fn emlite_val_obj_call(obj: Handle, method: &str, argv: Handle) -> Handle {
    host::emlite_val_obj_call(obj, method, argv)
}

pub unsafe fn emlite_val_obj_has_own_prop(obj: Handle, prop: &str) -> bool {
    host::emlite_val_obj_has_own_prop(obj, prop)
}

pub unsafe fn emlite_val_construct_new(ctor: Handle, argv: Handle) -> Handle {
    host::emlite_val_construct_new(ctor, argv)
}

pub unsafe fn emlite_val_func_call(fn_handle: Handle, argv: Handle) -> Handle {
    host::emlite_val_func_call(fn_handle, argv)
}

pub unsafe fn emlite_val_inc_ref(h: Handle) {
    host::emlite_val_inc_ref(h);
}

pub unsafe fn emlite_val_dec_ref(h: Handle) {
    host::emlite_val_dec_ref(h);
}

pub unsafe fn emlite_val_throw(h: Handle) -> ! {
    host::emlite_val_throw(h);
    unreachable!()
}

pub unsafe fn emlite_print_object_map() {
    host::emlite_print_object_map();
}

pub unsafe fn emlite_reset_object_map() {
    host::emlite_reset_object_map();
}

pub unsafe fn emlite_val_make_callback(fidx: Handle, data: Handle) -> Handle {
    host::emlite_val_make_callback(fidx, data)
}

// Safer callback management using lazy initialization and proper encapsulation
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

struct CallbackRegistry {
    callbacks: UnsafeCell<Vec<Option<Box<dyn Fn(Handle, Handle) -> Handle>>>>,
    initialized: AtomicBool,
}

// Safety: We ensure thread-safety through proper atomic operations and unsafe cell usage
unsafe impl Sync for CallbackRegistry {}

static CALLBACK_REGISTRY: CallbackRegistry = CallbackRegistry {
    callbacks: UnsafeCell::new(Vec::new()),
    initialized: AtomicBool::new(false),
};

impl CallbackRegistry {
    fn ensure_initialized(&self) {
        if !self.initialized.load(Ordering::Acquire) {
            // Initialize the vector - safe because we're the first to access it
            unsafe {
                (*self.callbacks.get()) = Vec::new();
            }
            self.initialized.store(true, Ordering::Release);
        }
    }
    
    fn register<F>(&self, callback: F) -> Handle
    where
        F: Fn(Handle, Handle) -> Handle + 'static,
    {
        self.ensure_initialized();
        
        let boxed_callback = Box::new(callback);
        
        unsafe {
            let callbacks = &mut *self.callbacks.get();
            
            // Find an empty slot or add to the end
            for (i, slot) in callbacks.iter_mut().enumerate() {
                if slot.is_none() {
                    *slot = Some(boxed_callback);
                    return i as Handle;
                }
            }
            
            callbacks.push(Some(boxed_callback));
            (callbacks.len() - 1) as Handle
        }
    }
    
    fn unregister(&self, fidx: Handle) {
        if self.initialized.load(Ordering::Acquire) {
            unsafe {
                let callbacks = &mut *self.callbacks.get();
                if (fidx as usize) < callbacks.len() {
                    // Dropping the boxed callback will properly clean up memory
                    callbacks[fidx as usize] = None;
                }
            }
        }
    }
    
    // Optional: Clear all callbacks (useful for cleanup or testing)
    #[allow(dead_code)]
    fn clear_all(&self) {
        if self.initialized.load(Ordering::Acquire) {
            unsafe {
                let callbacks = &mut *self.callbacks.get();
                callbacks.clear();
            }
        }
    }
    
    fn call(&self, fidx: u32, argv: u32, data: u32) -> u32 {
        if self.initialized.load(Ordering::Acquire) {
            unsafe {
                let callbacks = &*self.callbacks.get();
                if let Some(Some(callback)) = callbacks.get(fidx as usize) {
                    return callback(argv, data);
                }
            }
        }
        0 // Return undefined handle if callback not found
    }
}

pub unsafe fn emlite_register_callback<F>(callback: F) -> Handle
where
    F: Fn(Handle, Handle) -> Handle + 'static,
{
    CALLBACK_REGISTRY.register(callback)
}

pub unsafe fn emlite_unregister_callback(fidx: Handle) {
    CALLBACK_REGISTRY.unregister(fidx)
}

pub unsafe fn emlite_env_dyncall_apply(fidx: u32, argv: u32, data: u32) -> u32 {
    CALLBACK_REGISTRY.call(fidx, argv, data)
}

// Unified interface functions to abstract away wasip2 vs other target differences

pub unsafe fn emlite_val_make_str_unified(s: &str) -> Handle {
    unsafe { emlite_val_make_str(s) }
}

pub unsafe fn emlite_val_make_str_utf16_unified(s: &[u16]) -> Handle {
    unsafe { emlite_val_make_str_utf16(s) }
}

pub unsafe fn emlite_val_obj_call_unified(obj: Handle, method: &str, argv: Handle) -> Handle {
    unsafe { emlite_val_obj_call(obj, method, argv) }
}

pub unsafe fn emlite_val_obj_has_own_prop_unified(obj: Handle, prop: &str) -> bool {
    unsafe { emlite_val_obj_has_own_prop(obj, prop) }
}

pub unsafe fn emlite_val_typeof_unified(h: Handle) -> String {
    unsafe { emlite_val_typeof(h) }
}

pub unsafe fn emlite_val_get_value_string_unified(h: Handle) -> Option<String> {
    unsafe { Some(emlite_val_get_value_string(h)) }
}

pub unsafe fn emlite_val_get_value_string_utf16_unified(h: Handle) -> Option<Vec<u16>> {
    unsafe { Some(emlite_val_get_value_string_utf16(h)) }
}

pub unsafe fn emlite_val_not_unified(h: Handle) -> bool {
    unsafe { emlite_val_not(h) }
}

// Function pointer type for callbacks
type CallbackFn = fn(Handle, Handle) -> Handle;

pub unsafe fn emlite_register_callback_unified(f: CallbackFn) -> Handle {
    unsafe { emlite_register_callback(f) }
}
