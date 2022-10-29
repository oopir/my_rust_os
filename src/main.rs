#![no_std]      // get rid of standard library, which uses os-specific code
#![no_main]     // get rid of 'c runtime zero', which usually calls main()
#![feature(abi_x86_interrupt)]  // allow use of needed rust feature that is still 'unstable'

use core::panic::PanicInfo;

use bootloader::{BootInfo,entry_point};

mod vga_buffer;
pub mod interrupts;
pub mod gdt;
pub mod memory;


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}

/* this macro defines a _start function that calls our function.
 * also, it verifies the correctness of our function's signature.
 * had we defined _start ourselves, the compiler won't know what 
 * to check for; thus we implement the verification in the macro */
 entry_point!(kernel_main);

// #[no_mangle]    // makes sure the compiler does not change the function's name
// pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use crate::memory::translate_addr;
    use x86_64::VirtAddr;
    use x86_64::structures::paging::PageTable;

    println!("System is up!");
    
    init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe { translate_addr(virt, phys_mem_offset) };
        println!("{:?} -> {:?}", virt, phys);
    }

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