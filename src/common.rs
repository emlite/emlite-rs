pub type Handle = u32;

#[unsafe(export_name = "emlite_target")]
pub extern "C" fn emlite_target() -> i32 {
    1040
}
