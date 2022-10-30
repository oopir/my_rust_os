
use x86_64::{ 
    structures::paging::PageTable, VirtAddr, PhysAddr,
    structures::paging::OffsetPageTable,
    structures::paging::{Page, PhysFrame, Mapper, Size4KiB, FrameAllocator} };

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};


// read about the frame allocator & the memory_map type here:
// https://os.phil-opp.com/paging-implementation/#allocating-frames
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}
// ---
impl BootInfoFrameAllocator {
    // function is unsafe since user must guarantee validity of the given memorymap
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    // returns an iterator over the usable frames
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get iterator of MemoryRegion objects ; filter out used regions
        let usable_regions = 
            self.memory_map.iter().filter(|r| r.region_type == MemoryRegionType::Usable);

        // convert prev iterator to an iterator of address ranges
        let addr_ranges = 
            usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        
        // 1. for each addr range, create an iterator of the range's *frames* 
        //    (by setting a step-size of 4096)
        // 2. use 'flat_map' to aggregate all the created iterators as one 
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));

        // map the frames to a PhysFrame object
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}
// ---
unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
        fn allocate_frame(&mut self) -> Option<PhysFrame> {
            let frame = self.usable_frames().nth(self.next);
            self.next += 1;
            frame
        }
}



/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable
{
    use x86_64::registers::control::Cr3;
    
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();

    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr    // remember - pgae_table_ptr is unsafe
}


// returns an initialized PageTable variable
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}


pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe { mapper.map_to(page, frame, flags, frame_allocator) };

    map_to_result.expect("map_to failed").flush();
}



/*

// the following code was replaced by a more generic code from the x86_64 module,
// but left here to remind me the 'core' of how address translation is imeplemented

// This function is unsafe for the same reason as active_level_4_table.
// Providing an 'envelope' function allows most of our code to be 'safe'
pub unsafe fn translate_addr(virt: VirtAddr, physical_memory_offset: VirtAddr) 
    -> Option<PhysAddr>
{    
    translate_addr_inner(virt, physical_memory_offset)
}


fn translate_addr_inner(virt: VirtAddr, physical_memory_offset: VirtAddr)
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
        let curr_table = unsafe { &*curr_table_ptr };   // read about '&*' in stackoverflow
        let entry = &curr_table[ind];

        curr_frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };
    }

    // append the page to the address' offset (the 12 LSB)
    Some(curr_frame.start_address() + u64::from(virt.page_offset()))
}

*/
