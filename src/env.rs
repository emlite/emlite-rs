use crate::common::{Handle, emlite_target};

use alloc::alloc::dealloc;
use core::alloc::Layout;
use core::ffi::{c_char, c_double, c_int, c_longlong, c_uint, c_ulonglong, c_void};

unsafe extern "C" {
    pub fn emlite_init_handle_table();
    pub fn emlite_val_new_array() -> Handle;
    pub fn emlite_val_new_object() -> Handle;

    pub fn emlite_val_typeof(val: Handle) -> *mut c_char;

    pub fn emlite_val_construct_new(ctor: Handle, argv: Handle) -> Handle;
    pub fn emlite_val_func_call(func: Handle, argv: Handle) -> Handle;

    pub fn emlite_val_push(arr: Handle, v: Handle);

    pub fn emlite_val_make_bool(t: bool) -> Handle;
    pub fn emlite_val_make_int(t: c_int) -> Handle;
    pub fn emlite_val_make_uint(t: c_uint) -> Handle;
    pub fn emlite_val_make_bigint(t: c_longlong) -> Handle;
    pub fn emlite_val_make_biguint(t: c_ulonglong) -> Handle;
    pub fn emlite_val_make_double(t: c_double) -> Handle;
    pub fn emlite_val_make_str(s: *const c_char, len: usize) -> Handle;
    pub fn emlite_val_make_str_utf16(s: *const u16, len: usize) -> Handle;

    pub fn emlite_val_get_value_bool(val: Handle) -> bool;
    pub fn emlite_val_get_value_int(val: Handle) -> c_int;
    pub fn emlite_val_get_value_uint(val: Handle) -> c_int;
    pub fn emlite_val_get_value_bigint(val: Handle) -> c_longlong;
    pub fn emlite_val_get_value_biguint(val: Handle) -> c_ulonglong;
    pub fn emlite_val_get_value_double(val: Handle) -> c_double;
    pub fn emlite_val_get_value_string(val: Handle) -> *mut c_char;
    pub fn emlite_val_get_value_string_utf16(val: Handle) -> *mut u16;

    pub fn emlite_val_get(val: Handle, idx: Handle) -> Handle;
    pub fn emlite_val_set(val: Handle, idx: Handle, val: Handle);
    pub fn emlite_val_has(val: Handle, idx: Handle) -> bool;

    pub fn emlite_val_is_string(val: Handle) -> bool;
    pub fn emlite_val_is_number(val: Handle) -> bool;
    pub fn emlite_val_is_bool(val: Handle) -> bool;
    pub fn emlite_val_not(val: Handle) -> bool;
    pub fn emlite_val_gt(arg1: Handle, arg2: Handle) -> bool;
    pub fn emlite_val_gte(arg1: Handle, arg2: Handle) -> bool;
    pub fn emlite_val_lt(arg1: Handle, arg2: Handle) -> bool;
    pub fn emlite_val_lte(arg1: Handle, arg2: Handle) -> bool;
    pub fn emlite_val_equals(arg1: Handle, arg2: Handle) -> bool;
    pub fn emlite_val_strictly_equals(arg1: Handle, arg2: Handle) -> bool;
    pub fn emlite_val_instanceof(arg1: Handle, arg2: Handle) -> bool;
    pub fn emlite_val_inc_ref(val: Handle);
    pub fn emlite_val_dec_ref(val: Handle);
    pub fn emlite_val_throw(val: Handle) -> !;

    pub fn emlite_val_obj_call(
        obj: Handle,
        name: *const c_char,
        len: usize,
        argv: Handle,
    ) -> Handle;

    pub fn emlite_val_obj_has_own_prop(obj: Handle, prop: *const c_char, len: usize) -> bool;

    pub fn emlite_val_make_callback(id: Handle, data: Handle) -> Handle;

    pub fn emlite_print_object_map();

    pub fn emlite_reset_object_map();
}

// Unified interface functions to abstract away wasip2 vs other target differences
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ffi::CStr;

pub unsafe fn emlite_val_make_str_unified(s: &str) -> Handle {
    unsafe { emlite_val_make_str(s.as_ptr() as _, s.len()) }
}

pub unsafe fn emlite_val_make_str_utf16_unified(s: &[u16]) -> Handle {
    unsafe { emlite_val_make_str_utf16(s.as_ptr(), s.len()) }
}

pub unsafe fn emlite_val_obj_call_unified(obj: Handle, method: &str, argv: Handle) -> Handle {
    unsafe { emlite_val_obj_call(obj, method.as_ptr() as _, method.len(), argv) }
}

pub unsafe fn emlite_val_obj_has_own_prop_unified(obj: Handle, prop: &str) -> bool {
    unsafe { emlite_val_obj_has_own_prop(obj, prop.as_ptr() as _, prop.len()) }
}

pub unsafe fn emlite_val_typeof_unified(h: Handle) -> String {
    unsafe {
        let ptr = emlite_val_typeof(h);
        if ptr.is_null() {
            String::from("undefined")
        } else {
            String::from_utf8_lossy(CStr::from_ptr(ptr).to_bytes()).to_string()
        }
    }
}

pub unsafe fn emlite_val_get_value_string_unified(h: Handle) -> Option<String> {
    unsafe {
        let ptr = emlite_val_get_value_string(h);
        if ptr.is_null() {
            None
        } else {
            Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }
}

pub unsafe fn emlite_val_get_value_string_utf16_unified(h: Handle) -> Option<Vec<u16>> {
    unsafe {
        let ptr = emlite_val_get_value_string_utf16(h);
        if ptr.is_null() {
            None
        } else {
            // Find length by searching for null terminator
            let mut len = 0;
            let mut current = ptr;
            while *current != 0 {
                len += 1;
                current = current.add(1);
            }
            // Convert to Vec<u16>
            let slice = core::slice::from_raw_parts(ptr, len);
            Some(slice.to_vec())
        }
    }
}

pub unsafe fn emlite_val_not_unified(h: Handle) -> bool {
    unsafe { emlite_val_not(h) }
}

// Function pointer type for callbacks (match C ABI for indirect calls)
type CallbackFn = extern "C" fn(Handle, Handle) -> Handle;

pub unsafe fn emlite_register_callback_unified(f: CallbackFn) -> Handle {
    f as usize as Handle
}
