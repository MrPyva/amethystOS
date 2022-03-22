use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

static mut TSS: Option<TaskStateSegment> = None;

pub fn load() {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[0] = {
        const STACK_SIZE: usize = 4096 *5;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

        let stack_start = VirtAddr::from_ptr(unsafe {&STACK});
        let stacK_end = stack_start + STACK_SIZE;
        stacK_end
    };
    unsafe {TSS = Some(tss)}
}