pub type Handle = u32;

#[unsafe(export_name = "emlite_target")]
pub extern "C" fn emlite_target() -> i32 {
    1040
}

#[unsafe(export_name = "emlite_malloc")]
pub extern "C" fn emlite_malloc(sz: usize) -> *mut core::ffi::c_void {
    use core::alloc::Layout;
    let size = core::cmp::max(sz, 1);
    let layout = Layout::from_size_align(size, 1).unwrap();
    unsafe { alloc::alloc::alloc(layout) as _ }
}