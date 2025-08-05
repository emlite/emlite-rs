#![allow(unused)]

#[unsafe(no_mangle)]
#[unsafe(export_name = "emlite_target")]
pub extern "C" fn emlite_target() -> i32 {
    1031
}

#[unsafe(no_mangle)]
#[unsafe(export_name = "emlite_malloc")]
pub extern "C" fn emlite_malloc(sz: usize) -> *mut core::ffi::c_void {
    use core::alloc::{Layout};
    let size = core::cmp::max(sz, 1);
    let layout = Layout::from_size_align(size, 1).unwrap();
    unsafe { alloc::alloc::alloc(layout) as _ }
}

use core::ffi::{c_char, c_double, c_int, c_uint, c_void, c_longlong, c_ulonglong};
pub type Handle = u32;

unsafe extern "C" {
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
