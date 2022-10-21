
use x86_64::{ structures::paging::PageTable, VirtAddr, PhysAddr };


/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable
{
    use x86_64::registers::control::Cr3;
    
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();

    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr    // remember - pgae_table_ptr is unsafe
}


pub unsafe fn virt_to_phys(virt: VirtAddr, physical_memory_offset: VirtAddr) 
    -> Option<PhysAddr>
{    
    phys_to_virt_inner(virt, physical_memory_offset)
}

fn phys_to_virt_inner(virt: VirtAddr, physical_memory_offset: VirtAddr)
    -> Option<PhysAddr>
{
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;

    let indices = [virt.p4_index(), virt.p3_index(), virt.p2_index(), virt.p1_index()];

    let (level_4_table_frame, _) = Cr3::read();

    let mut curr_frame = level_4_table_frame;

    for &ind in &indices {
        let curr_virt = physical_memory_offset + curr_frame.start_address().as_u64();
        let curr_table_ptr: *const PageTable = curr_virt.as_ptr();
        let curr_table = unsafe { &*curr_table_ptr };
        let entry = &curr_table[ind];

        curr_frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };
    }

    Some(curr_frame.start_address() + u64::from(virt.page_offset()))
}