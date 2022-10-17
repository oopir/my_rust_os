use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector:  SegmentSelector,
}


lazy_static! {
    // the TSS is a legacy structure that stores the IST (interrupt stack table).
    // the IST is a store of 7 'good/usable' stacks.
    // we need them, fior example, to function when handling a stack overflow.
    static ref TSS: TaskStateSegment = {

        let mut tss = TaskStateSegment::new();

        // the following block creates a stack of length STACK_SIZE, 
        // and then initializes the TSS's first entry as the stack's start
        // (technically its end, 'cause our stack is 'top-down')
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end   = stack_start + STACK_SIZE;
            stack_end
        };

        tss
    };
}


lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector  = gdt.add_entry(Descriptor::tss_segment(&TSS));

        (gdt, Selectors { code_selector, tss_selector })
    };
}


pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};
    
    GDT.0.load();   // load GDT
    
    // block is unsafe because 'set_reg', 'load_tss' are marked unsafe
    unsafe {
        CS::set_reg(GDT.1.code_selector);   // reload code segment to CS register
        load_tss(GDT.1.tss_selector);       // load our new TSS
    }
}


