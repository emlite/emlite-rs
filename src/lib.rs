#![no_std]
#![allow(unused_unsafe)]
#![allow(unused_imports)]
extern crate alloc;

pub mod common;

#[cfg(not(all(target_os = "wasi", target_env = "p2")))]
pub mod env;
#[cfg(not(all(target_os = "wasi", target_env = "p2")))]
use crate::env::*;

#[cfg(all(target_os = "wasi", target_env = "p2"))]
pub mod wasip2env;
#[cfg(all(target_os = "wasi", target_env = "p2"))]
use crate::wasip2env::*;

use crate::common::Handle;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::ffi::CStr;

#[repr(u32)]
pub enum EmlitePredefHandles {
    Null = 0,
    Undefined,
    False,
    True,
    GlobalThis,
    Console,
    Reserved,
}

/// Runs JS eval
#[macro_export]
macro_rules! eval {
    ($src: literal) => {{
        $crate::Val::global("eval").invoke(&[$crate::Val::from($src)])
    }};
    ($src: literal $(, $arg:expr)* $(,)?) => {{
        $crate::Val::global("eval").invoke(
            &[$crate::Val::from(&format!($src, $( $arg ),*)) ]
        )
    }};
}

/// A helper macro which packs values into a slice of Val
#[macro_export]
macro_rules! argv {
    ($($rest:expr),*) => {{
        [$($crate::Val::from($rest)),*]
    }};
}

pub fn init() {
    unsafe {
        emlite_init_handle_table();
    }
}

/// A wrapper around a javascript handle
#[derive(Debug)]
pub struct Val {
    inner: Handle,
}

impl Val {
    /// Returns the globalThis object
    pub const fn global_this() -> Val {
        Val {
            inner: EmlitePredefHandles::GlobalThis as _,
        }
    }

    /// Gets the property `prop`
    pub fn get<T: Into<Val>>(&self, prop: T) -> Val {
        let h = unsafe { emlite_val_get(self.as_handle(), prop.into().as_handle()) };
        Val::take_ownership(h)
    }

    /// Gets a global object by `name`
    pub fn global(name: &str) -> Val {
        Val::global_this().get(name)
    }

    /// Gets a js null Val
    pub const fn null() -> Val {
        Val {
            inner: EmlitePredefHandles::Null as _,
        }
    }

    /// Gets a js undefined Val
    pub const fn undefined() -> Val {
        Val {
            inner: EmlitePredefHandles::Undefined as _,
        }
    }

    /// Gets a new js object
    pub fn object() -> Val {
        Val::take_ownership(unsafe { emlite_val_new_object() })
    }

    /// Gets a new js array
    pub fn array() -> Val {
        Val::take_ownership(unsafe { emlite_val_new_array() })
    }

    /// Creates a JavaScript Array from a Rust slice by pushing elements.
    /// Each element is converted via `Into<Val>` and appended using `Array.prototype.push`.
    /// This mirrors how argument arrays are built elsewhere and ensures JS values (not handles)
    /// are stored in the resulting array.
    pub fn from_slice<T>(slice: &[T]) -> Val
    where
        T: Clone + Into<Val>,
    {
        let arr = Val::array();
        for item in slice {
            // Use method call so JS receives actual values, avoiding handle-lifetime issues
            arr.call("push", &[item.clone().into()]);
        }
        arr
    }

    /// Set the underlying js object property `prop` to `val`
    pub fn set<K: Into<Val>, V: Into<Val>>(&self, prop: K, val: V) {
        unsafe {
            emlite_val_set(
                self.as_handle(),
                prop.into().as_handle(),
                val.into().as_handle(),
            )
        };
    }

    /// Checks whether a property `prop` exists
    pub fn has<T: Into<Val>>(&self, prop: T) -> bool {
        unsafe { emlite_val_has(self.as_handle(), prop.into().as_handle()) }
    }

    /// Checks whether a non-inherited property `prop` exists
    pub fn has_own_property(&self, prop: &str) -> bool {
        unsafe { emlite_val_obj_has_own_prop_unified(self.as_handle(), prop) }
    }

    /// Gets the typeof the underlying js object
    pub fn type_of(&self) -> String {
        unsafe { emlite_val_typeof_unified(self.as_handle()) }
    }

    /// Gets the element at index `idx`. Assumes the underlying js type is indexable
    pub fn at<T: Into<Val>>(&self, idx: T) -> Val {
        Val::take_ownership(unsafe { emlite_val_get(self.as_handle(), idx.into().as_handle()) })
    }

    /// Converts the underlying js array to a Vec of V
    pub fn to_vec<V: FromVal>(&self) -> Vec<V> {
        let len = self.get("length").as_::<usize>();
        let mut v: Vec<V> = Vec::with_capacity(len);
        for i in 0..len {
            v.push(self.at::<i32>(i as _).as_::<V>());
        }
        v
    }

    /// Calls the method `f` with `args`, can return an undefined js value
    pub fn call(&self, f: &str, args: &[Val]) -> Val {
        unsafe {
            let arr = Val::take_ownership(emlite_val_new_array());
            for arg in args {
                emlite_val_push(arr.as_handle(), arg.as_handle());
            }
            Val::take_ownership(emlite_val_obj_call_unified(
                self.as_handle(),
                f,
                arr.as_handle(),
            ))
        }
    }

    /// Calls the object's constructor with `args` constructing a new object
    pub fn new(&self, args: &[Val]) -> Val {
        unsafe {
            let arr = Val::take_ownership(emlite_val_new_array());
            for arg in args {
                emlite_val_push(arr.as_handle(), arg.as_handle());
            }
            Val::take_ownership(emlite_val_construct_new(self.as_handle(), arr.as_handle()))
        }
    }

    /// Invokes the function object with `args`, can return an undefined js value
    pub fn invoke(&self, args: &[Val]) -> Val {
        unsafe {
            let arr = Val::take_ownership(emlite_val_new_array());
            for arg in args {
                emlite_val_push(arr.as_handle(), arg.as_handle());
            }
            Val::take_ownership(emlite_val_func_call(self.as_handle(), arr.as_handle()))
        }
    }

    /// Creates a JS function from a function pointer `f` and `data` handle
    /// by packing them into a shared `EmliteCbPack` and returning the JS
    /// function Val. Works across languages in the same module.
    pub fn make_fn_raw(f: fn(Handle, Handle) -> Handle, data: Handle) -> Val {
        let idx = unsafe { emlite_register_callback_unified(f) };
        #[cfg(all(target_os = "wasi", target_env = "p2"))]
        unsafe {
            // Pin user data for the lifetime of the JS function
            emlite_val_inc_ref(data);

            // Allocate pack using module allocator so C can free it
            #[repr(C)]
            struct Pack {
                f: extern "C" fn(Handle, Handle) -> Handle,
                user_data: Handle,
            }
            unsafe extern "C" {
                fn emlite_malloc(sz: usize) -> *mut core::ffi::c_void;
            }
            let pack_ptr = emlite_malloc(core::mem::size_of::<Pack>()) as *mut Pack;
            if pack_ptr.is_null() {
                // Allocation failure: return undefined
                return Val::undefined();
            }
            (*pack_ptr).f = core::mem::transmute(f);
            (*pack_ptr).user_data = data;

            // Pass the pack pointer as a BigInt handle to JS
            let packed_handle = emlite_val_make_biguint(pack_ptr as usize as _);
            Val::take_ownership(emlite_val_make_callback(idx, packed_handle))
        }
        #[cfg(not(all(target_os = "wasi", target_env = "p2")))]
        unsafe {
            Val::take_ownership(emlite_val_make_callback(idx, data))
        }
    }

    /// Creates a js function from a Rust closure and returns a Val
    pub fn make_fn<F: FnMut(&[Val]) -> Val>(cb: F) -> Val {
        fn shim(args: Handle, data: Handle) -> Handle {
            let v = Val::take_ownership(args);
            let vals: Vec<Val> = v.to_vec();
            let func0 = Val::take_ownership(data);
            let a = func0.as_::<i32>() as usize as *mut Box<dyn FnMut(&[Val]) -> Val>;
            let f: &mut (dyn FnMut(&[Val]) -> Val) = unsafe { &mut **a };
            core::mem::forget(func0);
            f(&vals).as_handle()
        }
        #[allow(clippy::type_complexity)]
        let a: *mut Box<dyn FnMut(&[Val]) -> Val> = Box::into_raw(Box::new(Box::new(cb)));
        let data = Val::from(a as Handle);
        unsafe {
            emlite_val_inc_ref(data.as_handle());
        }
        Self::make_fn_raw(shim, data.as_handle())
    }

    /// Awaits the invoked function object
    pub fn await_(&self) -> Val {
        eval!(
            r#"
            (async () => {{
                let obj = EMLITE_VALMAP.toValue({});
                let ret = await obj;
                return EMLITE_VALMAP.toHandle(ret);
            }})()
        "#,
            self.as_handle()
        )
    }

    /// Decrements the refcount of the underlying handle
    pub fn delete(v: Val) {
        unsafe {
            emlite_val_dec_ref(v.as_handle());
        }
    }

    /// Throws a js object represented by Val
    pub fn throw(v: Val) -> ! {
        unsafe {
            emlite_val_throw(v.as_handle());
        }
    }

    /// Checks whether this Val is an instanceof `v`
    pub fn instanceof(&self, v: Val) -> bool {
        unsafe { emlite_val_instanceof(self.as_handle(), v.as_handle()) }
    }

    pub fn is_number(&self) -> bool {
        unsafe { emlite_val_is_number(self.as_handle()) }
    }

    pub fn is_bool(&self) -> bool {
        unsafe { emlite_val_is_bool(self.as_handle()) }
    }

    pub fn is_string(&self) -> bool {
        unsafe { emlite_val_is_string(self.as_handle()) }
    }

    pub fn is_null(&self) -> bool {
        self.as_handle() == EmlitePredefHandles::Null as u32
    }

    pub fn is_undefined(&self) -> bool {
        self.as_handle() == EmlitePredefHandles::Undefined as u32
    }

    pub fn is_error(&self) -> bool {
        self.instanceof(Val::global("Error"))
    }

    pub fn is_function(&self) -> bool {
        self.instanceof(Val::global("Function"))
    }

    #[inline(always)]
    pub fn as_<T>(&self) -> T
    where
        T: FromVal,
    {
        T::from_val(self)
    }

    /// Creates a Val from UTF-16 data
    pub fn from_utf16(utf16: &[u16]) -> Val {
        Val::from(utf16)
    }

    /// Extracts UTF-16 data as Option<Vec<u16>>
    pub fn to_utf16(&self) -> Option<Vec<u16>> {
        self.as_::<Option<Vec<u16>>>()
    }

    /// Extracts UTF-16 data, returning error if null or if self is error
    pub fn to_utf16_result(&self) -> Result<Vec<u16>, Val> {
        self.as_::<Result<Vec<u16>, Val>>()
    }

    /// Converts UTF-16 Vec<u16> to String, if possible
    pub fn utf16_to_string(utf16: &[u16]) -> Result<String, ()> {
        // Simple conversion that works for basic cases
        // For a full implementation, you'd want proper UTF-16 decoding
        match String::from_utf16(utf16) {
            Ok(s) => Ok(s),
            Err(_) => Err(()),
        }
    }

    /// Creates a Val from a String by first converting to UTF-16
    pub fn from_string_via_utf16(s: &str) -> Val {
        let utf16: Vec<u16> = s.encode_utf16().collect();
        Val::from_utf16(&utf16)
    }
}

impl From<bool> for Val {
    fn from(v: bool) -> Self {
        Val::take_ownership(unsafe { emlite_val_make_bool(v as _) })
    }
}

impl From<i8> for Val {
    fn from(v: i8) -> Self {
        Val::take_ownership(unsafe { emlite_val_make_int(v as _) })
    }
}

impl From<u8> for Val {
    fn from(v: u8) -> Self {
        Val::take_ownership(unsafe { emlite_val_make_int(v as _) })
    }
}

impl From<i16> for Val {
    fn from(v: i16) -> Self {
        Val::take_ownership(unsafe { emlite_val_make_int(v as _) })
    }
}

impl From<u16> for Val {
    fn from(v: u16) -> Self {
        Val::take_ownership(unsafe { emlite_val_make_int(v as _) })
    }
}

impl From<i32> for Val {
    fn from(v: i32) -> Self {
        Val::take_ownership(unsafe { emlite_val_make_int(v) })
    }
}

impl From<u32> for Val {
    fn from(v: u32) -> Self {
        Val::take_ownership(unsafe { emlite_val_make_uint(v as _) })
    }
}

impl From<i64> for Val {
    fn from(v: i64) -> Self {
        Val::take_ownership(unsafe { emlite_val_make_bigint(v as _) })
    }
}

impl From<u64> for Val {
    fn from(v: u64) -> Self {
        Val::take_ownership(unsafe { emlite_val_make_biguint(v as _) })
    }
}

impl From<usize> for Val {
    fn from(v: usize) -> Self {
        Val::take_ownership(unsafe { emlite_val_make_biguint(v as _) })
    }
}

impl From<isize> for Val {
    fn from(v: isize) -> Self {
        Val::take_ownership(unsafe { emlite_val_make_bigint(v as _) })
    }
}

impl From<f32> for Val {
    fn from(v: f32) -> Self {
        Val::take_ownership(unsafe { emlite_val_make_double(v as _) })
    }
}

impl From<f64> for Val {
    fn from(v: f64) -> Self {
        Val::take_ownership(unsafe { emlite_val_make_double(v) })
    }
}

impl From<()> for Val {
    fn from(_: ()) -> Self {
        Val::undefined()
    }
}

impl From<&str> for Val {
    fn from(s: &str) -> Self {
        Val::take_ownership(unsafe { emlite_val_make_str_unified(s) })
    }
}

impl From<String> for Val {
    fn from(s: String) -> Self {
        Val::take_ownership(unsafe { emlite_val_make_str_unified(&s) })
    }
}

impl From<&String> for Val {
    fn from(s: &String) -> Self {
        Val::take_ownership(unsafe { emlite_val_make_str_unified(s) })
    }
}

impl From<&[u16]> for Val {
    fn from(s: &[u16]) -> Self {
        Val::take_ownership(unsafe { emlite_val_make_str_utf16_unified(s) })
    }
}

impl From<Vec<u16>> for Val {
    fn from(s: Vec<u16>) -> Self {
        Val::take_ownership(unsafe { emlite_val_make_str_utf16_unified(&s) })
    }
}

impl From<&Vec<u16>> for Val {
    fn from(s: &Vec<u16>) -> Self {
        Val::take_ownership(unsafe { emlite_val_make_str_utf16_unified(s) })
    }
}

impl From<&Val> for Val {
    fn from(v: &Val) -> Self {
        v.clone()
    }
}

impl Drop for Val {
    fn drop(&mut self) {
        unsafe { emlite_val_dec_ref(self.as_handle()) }
    }
}

impl Clone for Val {
    fn clone(&self) -> Val {
        unsafe {
            emlite_val_inc_ref(self.as_handle());
        }
        Val::take_ownership(self.as_handle())
    }
}

use core::ops::{Deref, DerefMut};

/// A console wrapper
#[derive(Clone, Debug)]
pub struct Console {
    val: Val,
}

impl Console {
    /// Gets the console
    pub const fn get() -> Console {
        Console {
            val: Val {
                inner: EmlitePredefHandles::Console as _,
            },
        }
    }

    /// Logs into the console
    pub fn log(&self, args: &[Val]) {
        self.val.call("log", args);
    }

    /// console.warn
    pub fn warn(&self, args: &[Val]) {
        self.val.call("warn", args);
    }

    /// console.info
    pub fn info(&self, args: &[Val]) {
        self.val.call("info", args);
    }

    /// Returns the underlying handle of the console
    pub fn as_handle(&self) -> Handle {
        self.val.as_handle()
    }
}

impl Deref for Console {
    type Target = Val;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl DerefMut for Console {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

impl From<Console> for Val {
    fn from(val: Console) -> Self {
        Val::take_ownership(val.as_handle())
    }
}

use core::cmp::Ordering;
use core::ops::Not;

impl PartialEq for Val {
    fn eq(&self, other: &Val) -> bool {
        unsafe { emlite_val_strictly_equals(self.as_handle(), other.as_handle()) }
    }
}

impl PartialOrd for Val {
    fn partial_cmp(&self, other: &Val) -> Option<Ordering> {
        unsafe {
            if emlite_val_strictly_equals(self.as_handle(), other.as_handle()) {
                Some(Ordering::Equal)
            } else if emlite_val_gt(self.as_handle(), other.as_handle()) {
                Some(Ordering::Greater)
            } else if emlite_val_lt(self.as_handle(), other.as_handle()) {
                Some(Ordering::Less)
            } else {
                None
            }
        }
    }
}

impl Not for Val {
    type Output = bool;

    fn not(self) -> Self::Output {
        unsafe { emlite_val_not(self.as_handle()) }
    }
}

impl core::ops::Not for &Val {
    type Output = bool;

    fn not(self) -> Self::Output {
        unsafe { emlite_val_not_unified(self.as_handle()) }
    }
}

impl AsRef<Val> for Val {
    #[inline]
    fn as_ref(&self) -> &Val {
        self
    }
}

impl AsMut<Val> for Val {
    #[inline]
    fn as_mut(&mut self) -> &mut Val {
        self
    }
}

pub trait FromVal: Sized {
    /// Creates a Val object from another
    fn from_val(v: &Val) -> Self;
    /// Takes the ownership of a handle
    fn take_ownership(v: Handle) -> Self;
    /// Returns the raw js handle
    fn as_handle(&self) -> Handle;
}

impl FromVal for Val {
    fn from_val(v: &Val) -> Self {
        unsafe {
            emlite_val_inc_ref(v.inner);
        }
        Val {
            inner: v.as_handle(),
        }
    }
    fn take_ownership(v: Handle) -> Self {
        Val { inner: v }
    }
    #[inline(always)]
    fn as_handle(&self) -> Handle {
        self.inner
    }
}

impl FromVal for Result<Val, Val> {
    fn from_val(v: &Val) -> Self {
        unsafe {
            emlite_val_inc_ref(v.inner);
        }
        if v.is_error() {
            Err(v.clone())
        } else {
            Ok(v.clone())
        }
    }
    fn take_ownership(v: Handle) -> Self {
        let temp = Val::take_ownership(v);
        if temp.is_error() { Err(temp) } else { Ok(temp) }
    }
    #[inline(always)]
    fn as_handle(&self) -> Handle {
        match self {
            Ok(ok) => ok.as_handle(),
            Err(e) => e.as_handle(),
        }
    }
}

impl FromVal for bool {
    fn from_val(v: &Val) -> Self {
        unsafe { !emlite_val_not_unified(v.as_handle()) }
    }
    fn take_ownership(v: Handle) -> Self {
        Self::from_val(&Val::take_ownership(v))
    }
    fn as_handle(&self) -> Handle {
        if *self {
            EmlitePredefHandles::True as u32
        } else {
            EmlitePredefHandles::False as u32
        }
    }
}

impl FromVal for Option<bool> {
    fn from_val(v: &Val) -> Self {
        unsafe {
            if v.is_error() || v.is_null() || v.is_undefined() {
                None
            } else {
                Some(!emlite_val_not_unified(v.as_handle()))
            }
        }
    }
    fn take_ownership(v: Handle) -> Self {
        let temp = Val::take_ownership(v);
        if temp.is_error() || temp.is_null() || temp.is_undefined() {
            None
        } else {
            unsafe { Some(!emlite_val_not_unified(v)) }
        }
    }
    fn as_handle(&self) -> Handle {
        match self {
            Some(ok) => {
                if *ok {
                    EmlitePredefHandles::True as u32
                } else {
                    EmlitePredefHandles::False as u32
                }
            }
            None => EmlitePredefHandles::Undefined as u32,
        }
    }
}

impl FromVal for Result<bool, Val> {
    fn from_val(v: &Val) -> Self {
        unsafe {
            if v.is_error() {
                Err(v.clone())
            } else {
                Ok(!emlite_val_not_unified(v.as_handle()))
            }
        }
    }
    fn take_ownership(v: Handle) -> Self {
        let temp = Val::take_ownership(v);
        if temp.is_error() {
            Err(temp)
        } else {
            unsafe { Ok(!emlite_val_not_unified(v)) }
        }
    }
    fn as_handle(&self) -> Handle {
        match self {
            Ok(ok) => {
                if *ok {
                    EmlitePredefHandles::True as u32
                } else {
                    EmlitePredefHandles::False as u32
                }
            }
            Err(e) => e.as_handle(),
        }
    }
}

macro_rules! impl_int {
    ($($t:ty),*) => {$(
        impl FromVal for $t {
            fn from_val(v: &Val) -> Self {
                unsafe {
                    emlite_val_get_value_int(v.as_handle()) as Self
                }
            }
            fn take_ownership(v: Handle) -> Self {
                unsafe { emlite_val_get_value_int(v) as Self }
            }
            fn as_handle(&self) -> Handle {
                0
            }
        }
        impl FromVal for Option<$t> {
            fn from_val(v: &Val) -> Self {
                unsafe {
                    if !v.is_number() {
                        None
                    } else {
                        Some(emlite_val_get_value_int(v.as_handle()) as $t)
                    }
                }
            }
            fn take_ownership(v: Handle) -> Self {
                let temp = Val::take_ownership(v);
                if !temp.is_number() {
                    None
                } else {
                    unsafe { Some(emlite_val_get_value_int(v) as $t) }
                }
            }
            fn as_handle(&self) -> Handle {
                0
            }
        }
        impl FromVal for Result<$t, Val> {
            fn from_val(v: &Val) -> Self {
                unsafe {
                    if v.is_error() {
                        Err(v.clone())
                    } else {
                        Ok(emlite_val_get_value_int(v.as_handle()) as $t)
                    }
                }
            }
            fn take_ownership(v: Handle) -> Self {
                let temp = Val::take_ownership(v);
                if temp.is_error() {
                    Err(temp)
                } else {
                    unsafe { Ok(emlite_val_get_value_int(v) as $t) }
                }
            }
            fn as_handle(&self) -> Handle {
                0
            }
        }
    )*}
}

macro_rules! impl_uint {
    ($($t:ty),*) => {$(
        impl FromVal for $t {
            fn from_val(v: &Val) -> Self {
                unsafe {
                    emlite_val_get_value_uint(v.as_handle()) as Self
                }
            }
            fn take_ownership(v: Handle) -> Self {
                unsafe { emlite_val_get_value_uint(v) as Self }
            }
            fn as_handle(&self) -> Handle {
                0
            }
        }
        impl FromVal for Option<$t> {
            fn from_val(v: &Val) -> Self {
                unsafe {
                    if !v.is_number() {
                        None
                    } else {
                        Some(emlite_val_get_value_uint(v.as_handle()) as $t)
                    }
                }
            }
            fn take_ownership(v: Handle) -> Self {
                let temp = Val::take_ownership(v);
                if !temp.is_number() {
                    None
                } else {
                    unsafe { Some(emlite_val_get_value_uint(v) as $t) }
                }
            }
            fn as_handle(&self) -> Handle {
                0
            }
        }
        impl FromVal for Result<$t, Val> {
            fn from_val(v: &Val) -> Self {
                unsafe {
                    if v.is_error() {
                        Err(v.clone())
                    } else {
                        Ok(emlite_val_get_value_uint(v.as_handle()) as $t)
                    }
                }
            }
            fn take_ownership(v: Handle) -> Self {
                let temp = Val::take_ownership(v);
                if temp.is_error() {
                    Err(temp)
                } else {
                    unsafe { Ok(emlite_val_get_value_uint(v) as $t) }
                }
            }
            fn as_handle(&self) -> Handle {
                0
            }
        }
    )*}
}

macro_rules! impl_bigint {
    ($($t:ty),*) => {$(
        impl FromVal for $t {
            fn from_val(v: &Val) -> Self {
                unsafe {
                    emlite_val_get_value_bigint(v.as_handle()) as Self
                }
            }
            fn take_ownership(v: Handle) -> Self {
                unsafe { emlite_val_get_value_bigint(v) as Self }
            }
            fn as_handle(&self) -> Handle {
                0
            }
        }
        impl FromVal for Option<$t> {
            fn from_val(v: &Val) -> Self {
                unsafe {
                    if !v.is_number() {
                        None
                    } else {
                        Some(emlite_val_get_value_bigint(v.as_handle()) as $t)
                    }
                }
            }
            fn take_ownership(v: Handle) -> Self {
                let temp = Val::take_ownership(v);
                if !temp.is_number() {
                    None
                } else {
                    unsafe { Some(emlite_val_get_value_bigint(v) as $t) }
                }
            }
            fn as_handle(&self) -> Handle {
                0
            }
        }
        impl FromVal for Result<$t, Val> {
            fn from_val(v: &Val) -> Self {
                unsafe {
                    if v.is_error() {
                        Err(v.clone())
                    } else {
                        Ok(emlite_val_get_value_bigint(v.as_handle()) as $t)
                    }
                }
            }
            fn take_ownership(v: Handle) -> Self {
                let temp = Val::take_ownership(v);
                if temp.is_error() {
                    Err(temp)
                } else {
                    unsafe { Ok(emlite_val_get_value_bigint(v) as $t) }
                }
            }
            fn as_handle(&self) -> Handle {
                0
            }
        }
    )*}
}

macro_rules! impl_biguint {
    ($($t:ty),*) => {$(
        impl FromVal for $t {
            fn from_val(v: &Val) -> Self {
                unsafe {
                    emlite_val_get_value_biguint(v.as_handle()) as Self
                }
            }
            fn take_ownership(v: Handle) -> Self {
                unsafe { emlite_val_get_value_biguint(v) as Self }
            }
            fn as_handle(&self) -> Handle {
                0
            }
        }
        impl FromVal for Option<$t> {
            fn from_val(v: &Val) -> Self {
                unsafe {
                    if !v.is_number() {
                        None
                    } else {
                        Some(emlite_val_get_value_biguint(v.as_handle()) as $t)
                    }
                }
            }
            fn take_ownership(v: Handle) -> Self {
                let temp = Val::take_ownership(v);
                if !temp.is_number() {
                    None
                } else {
                    unsafe { Some(emlite_val_get_value_biguint(v) as $t) }
                }
            }
            fn as_handle(&self) -> Handle {
                0
            }
        }
        impl FromVal for Result<$t, Val> {
            fn from_val(v: &Val) -> Self {
                unsafe {
                    if v.is_error() {
                        Err(v.clone())
                    } else {
                        Ok(emlite_val_get_value_biguint(v.as_handle()) as $t)
                    }
                }
            }
            fn take_ownership(v: Handle) -> Self {
                let temp = Val::take_ownership(v);
                if temp.is_error() {
                    Err(temp)
                } else {
                    unsafe { Ok(emlite_val_get_value_biguint(v) as $t) }
                }
            }
            fn as_handle(&self) -> Handle {
                0
            }
        }
    )*}
}

impl_int!(i8, i16, i32);
impl_uint!(u8, u16, u32);
impl_bigint!(i64, isize);
impl_biguint!(u64, usize);

macro_rules! impl_float {
    ($($t:ty),*) => {$(
        impl FromVal for $t {
            fn from_val(v: &Val) -> Self {
                unsafe { emlite_val_get_value_double(v.as_handle()) as Self }
            }
            fn take_ownership(v: Handle) -> Self {
                unsafe { emlite_val_get_value_double(v) as Self }
            }
            fn as_handle(&self) -> Handle {
                0
            }
        }
        impl FromVal for Option<$t> {
            fn from_val(v: &Val) -> Self {
                unsafe {
                    if !v.is_number() {
                        None
                    } else {
                        Some(emlite_val_get_value_double(v.as_handle()) as $t)
                    }
                }
            }
            fn take_ownership(v: Handle) -> Self {
                let temp = Val::take_ownership(v);
                if !temp.is_number() {
                    None
                } else {
                    unsafe { Some(emlite_val_get_value_double(v) as $t) }
                }
            }
            fn as_handle(&self) -> Handle {
                0
            }
        }
        impl FromVal for Result<$t, Val> {
            fn from_val(v: &Val) -> Self {
                unsafe {
                    if v.is_error() {
                        Err(v.clone())
                    } else {
                        Ok(emlite_val_get_value_double(v.as_handle()) as $t)
                    }
                }
            }
            fn take_ownership(v: Handle) -> Self {
                let temp = Val::take_ownership(v);
                if temp.is_error() {
                    Err(temp)
                } else {
                    unsafe { Ok(emlite_val_get_value_double(v) as $t) }
                }
            }
            fn as_handle(&self) -> Handle {
                0
            }
        }
    )*}
}

impl_float!(f32, f64);

impl FromVal for () {
    fn from_val(_v: &Val) -> Self {
        // Unit type doesn't carry any data, so we just return ()
        ()
    }
    fn take_ownership(_v: Handle) -> Self {
        ()
    }
    fn as_handle(&self) -> Handle {
        EmlitePredefHandles::Undefined as u32
    }
}

impl FromVal for Option<String> {
    fn from_val(v: &Val) -> Self {
        unsafe {
            if !v.is_string() {
                return None;
            }
            emlite_val_get_value_string_unified(v.as_handle())
        }
    }
    fn take_ownership(v: Handle) -> Self {
        unsafe {
            if !emlite_val_is_string(v) {
                return None;
            }
            emlite_val_get_value_string_unified(v)
        }
    }
    fn as_handle(&self) -> Handle {
        0
    }
}

impl FromVal for Result<String, Val> {
    fn from_val(v: &Val) -> Self {
        unsafe {
            if v.is_error() {
                Err(v.clone())
            } else if !v.is_string() {
                Err(Val::global("Error").new(&["Expected string".into()]))
            } else {
                match emlite_val_get_value_string_unified(v.as_handle()) {
                    Some(s) => Ok(s),
                    None => Err(Val::global("Error").new(&["Null string".into()])),
                }
            }
        }
    }
    fn take_ownership(v: Handle) -> Self {
        unsafe {
            let temp = Val::take_ownership(v);
            if temp.is_error() {
                Err(temp)
            } else if !temp.is_string() {
                Err(Val::global("Error").new(&["Expected string".into()]))
            } else {
                match emlite_val_get_value_string_unified(v) {
                    Some(s) => Ok(s),
                    None => Err(Val::global("Error").new(&["Null string".into()])),
                }
            }
        }
    }
    fn as_handle(&self) -> Handle {
        0
    }
}

impl FromVal for Option<Vec<u16>> {
    fn from_val(v: &Val) -> Self {
        unsafe { emlite_val_get_value_string_utf16_unified(v.as_handle()) }
    }
    fn take_ownership(v: Handle) -> Self {
        unsafe { emlite_val_get_value_string_utf16_unified(v) }
    }
    fn as_handle(&self) -> Handle {
        0
    }
}

impl FromVal for Result<Vec<u16>, Val> {
    fn from_val(v: &Val) -> Self {
        unsafe {
            if v.is_error() {
                Err(v.clone())
            } else {
                match emlite_val_get_value_string_utf16_unified(v.as_handle()) {
                    Some(vec) => Ok(vec),
                    None => Err(Val::global("Error").new(&["Null UTF-16 string".into()])),
                }
            }
        }
    }
    fn take_ownership(v: Handle) -> Self {
        unsafe {
            let temp = Val::take_ownership(v);
            if temp.is_error() {
                Err(temp)
            } else {
                match emlite_val_get_value_string_utf16_unified(v) {
                    Some(vec) => Ok(vec),
                    None => Err(Val::global("Error").new(&["Null UTF-16 string".into()])),
                }
            }
        }
    }
    fn as_handle(&self) -> Handle {
        0
    }
}

/// A marker trait for types that can be constructed from JavaScript error values.
/// This allows Result<T, E> implementations where E: FromJsError.
pub trait FromJsError {
    fn from_js_error(val: &Val) -> Self;
}

/// Implementation for Result<T, E> where T: FromVal and E: FromJsError.
/// This allows clean conversion using as_::<Result<T, E>>() for JavaScript error handling.
impl<T, E> FromVal for Result<T, E>
where
    T: FromVal,
    E: FromJsError,
{
    fn from_val(v: &Val) -> Self {
        if v.is_error() {
            Err(E::from_js_error(v))
        } else {
            Ok(T::from_val(v))
        }
    }
    fn take_ownership(handle: Handle) -> Self {
        let temp = Val::take_ownership(handle);
        if temp.is_error() {
            Err(E::from_js_error(&temp))
        } else {
            Ok(T::take_ownership(handle))
        }
    }
    fn as_handle(&self) -> Handle {
        match self {
            Ok(ok) => ok.as_handle(),
            Err(_) => 0, // Errors don't have meaningful handles in this context
        }
    }
}
