unsafe extern "C" {
    fn _sbss();
    fn _ebss();
}

/// Fills the `.bss` section with zeros.
///
/// It requires the symbols `_sbss` and `_ebss` to be defined in the linker script.
///
/// # Safety
/// This function is unsafe because it writes `.bss` section directly.
pub fn clear_bss() {
    unsafe {
        core::slice::from_raw_parts_mut(
            _sbss as *const () as usize as *mut u8,
            _ebss as *const () as usize - _sbss as *const () as usize,
        )
        .fill(0);
    }
}
