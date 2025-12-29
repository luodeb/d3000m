#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate log;

extern crate alloc;
extern crate axplat_aarch64_d3000m_n80_laptop;

mod config;
mod utils;

mod test;

fn init_kernel(cpu_id: usize, arg: usize) {
    // Initialize trap, console, time.
    axplat::init::init_early(cpu_id, arg);

    // Initialize platform peripherals (not used in this example).
    axplat::init::init_later(cpu_id, arg);
}

// UART0 基地址 (QEMU virt 机器)
const UART0_BASE: usize = 0xffff000018002000;
// const UART0_BASE: usize = 0x09000000;

/// 向 UART 写入一个字符
fn uart_putc(c: u8) {
    unsafe {
        let uart = UART0_BASE as *mut u8;
        core::ptr::write_volatile(uart, c);
    }
}

/// 打印字符串到 UART
fn uart_puts(s: &str) {
    for c in s.bytes() {
        uart_putc(c);
    }
}

#[axplat::main]
pub fn rust_main(cpu_id: usize, arg: usize) -> ! {
    utils::mem::clear_bss();

    uart_puts("Hello, RSTiny World JUHGJHG!\n");

    loop {}
    // init_kernel(cpu_id, arg);

    // axplat::console_println!("Hello, RSTiny!");

    // utils::logging::log_init();

    // info!("Logging initialized. This is an info message.");

    // test::run_allocator_tests();

    // axplat::power::system_off()
}

#[cfg(all(target_os = "none", not(test)))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    axplat::console_println!("{info}");
    axplat::power::system_off()
}
