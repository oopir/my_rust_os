#![no_std]      // get rid of standard library, which uses os-specific code
#![no_main]     // get rid of 'c runtime zero', which usually calls main()
#![feature(abi_x86_interrupt)]  // allow use of needed rust feature that is still 'unstable'

use core::panic::PanicInfo;

mod vga_buffer;
pub mod interrupts;
pub mod gdt;


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}


#[no_mangle]    // makes sure the compiler does not change the function's name
pub extern "C" fn _start() -> ! {
    
    println!("System is up!");
    
    init();

    println!("About to enter infinite loop...");
    
    hlt_loop();
}


fn init() {
    gdt::init();
    interrupts::init_idt();
    // the PIC initialization is unsafe since it's undefined when misconfigured
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable(); // required to have IO interrupts
}


pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}