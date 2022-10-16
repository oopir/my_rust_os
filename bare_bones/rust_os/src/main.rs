#![no_std]      // get rid of standard library, which uses os-specific code
#![no_main]     // get rid of 'c runtime zero', which usually calls main()

use core::panic::PanicInfo;

mod vga_buffer;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]    // makes sure the compiler does not change the function's name
pub extern "C" fn _start() -> ! {

    println!("Hello World{}", "!");
    
    loop {}
}