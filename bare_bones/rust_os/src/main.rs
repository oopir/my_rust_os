#![no_std]      // get rid of standard library, which uses os-specific code
#![no_main]     // get rid of 'c runtime zero', which usually calls main()
#![feature(abi_x86_interrupt)]  // allow use of needed rust feature that is still 'unstable'

use core::panic::PanicInfo;

mod vga_buffer;
pub mod interrupts;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]    // makes sure the compiler does not change the function's name
pub extern "C" fn _start() -> ! {

    println!("Hello World{}", "!");
    
    init();

    x86_64::instructions::interrupts::int3();
    
    println!("About to enter infinite loop...");

    loop {}
}

fn init() {
    interrupts::init_idt();
}