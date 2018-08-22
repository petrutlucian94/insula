use ::cpu::exits::VcpuExit;
use ::memory::MmapMemorySlot;

pub trait Accelerator {
    fn init_vcpu(&mut self);
    fn memory_region_add(&self, mem: &MmapMemorySlot);
    fn vcpu_run(&mut self, vcpu_index: usize) -> VcpuExit;
}
