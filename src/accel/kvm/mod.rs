extern crate clap;
extern crate libkvm;
extern crate std;

use libkvm::linux::kvm_bindings::*;
use libkvm::system::*;
use libkvm::vcpu::VirtualCPU;
use libkvm::vm::VirtualMachine;

use ::cpu::exits::VcpuExit;
use ::memory::MmapMemorySlot;
use super::base::Accelerator;

pub struct KVMAccelerator {
    kvm: KVMSystem,
    vm: VirtualMachine,
    vcpus: Vec<VirtualCPU>
}

impl KVMAccelerator {
    pub fn new() -> Self {
        let kvm = KVMSystem::new().unwrap();

        let api = kvm.api_version().unwrap();
        println!("KVM API version: {}", api);

        let vm = kvm.create_vm().unwrap();

        let accel = KVMAccelerator {
            kvm: kvm,
            vm: vm,
            vcpus: Vec::new()
        };

        accel.init_vm();

        accel
    }

    fn init_vm(&self) {
        let identity_base = 0xfeffc000;
        let tss_addr = identity_base + 0x1000;

        self.vm.set_identity_map_addr(identity_base).unwrap();

        println!("Setting TSS address: {:x}", tss_addr);
        self.vm.set_tss_address(tss_addr as u32).unwrap();
    }

    fn setup_cpuid(&self, vcpu: &VirtualCPU) {
        let mut kvm_cpuid_entries = self.kvm.get_supported_cpuid().unwrap();

        let i = kvm_cpuid_entries
            .iter()
            .position(|&r| r.function == 0x40000000)
            .unwrap();

        let mut id_reg_values: [u32; 3] = [0; 3];
        let id = "insula\0";
        unsafe {
            std::ptr::copy_nonoverlapping(
                id.as_ptr(), id_reg_values.as_mut_ptr() as *mut u8, id.len());
        }
        kvm_cpuid_entries[i].ebx = id_reg_values[0];
        kvm_cpuid_entries[i].ecx = id_reg_values[1];
        kvm_cpuid_entries[i].edx = id_reg_values[2];

        let i = kvm_cpuid_entries
            .iter()
            .position(|&r| r.function == 1)
            .unwrap();

        kvm_cpuid_entries[i].ecx |= ::cpu::constants::CPUID_EXT_HYPERVISOR;

        vcpu.set_cpuid(&kvm_cpuid_entries).unwrap();
    }

    fn setup_msrs(&self, vcpu: &VirtualCPU) {
        let msr_list = self.kvm.get_msr_index_list().unwrap();
        let ignored_msrs = [0x40000020, 0x40000022, 0x40000023];

        let msr_entries = msr_list
            .iter().filter(|i| !ignored_msrs.contains(i))
            .map(|i| kvm_msr_entry {
                index: *i,
                data: 0,
                ..Default::default()
            })
            .collect::<Vec<_>>();

        vcpu.set_msrs(&msr_entries).unwrap();
    }

    fn init_regs(&self, vcpu: &VirtualCPU) {
        // We'll probably add an accelerator independent
        // cpu structure, storing the resgisters and move this
        // logic out of the accelerator code.
        let mut sregs = vcpu.get_kvm_sregs().unwrap();

        sregs.cr0 = 0x60000010;
        let mut seg = kvm_segment {
            base: 0xffff0000,
            limit: 0xffff,
            selector: 0xf000,
            present: 1,
            type_: 11,
            dpl: 0,
            db: 0,
            s: 1,
            l: 0,
            g: 0,
            ..Default::default()
        };

        sregs.cs = seg;

        seg.base = 0;
        seg.type_ = 3;
        seg.selector = 0;
        sregs.ds = seg;
        sregs.es = seg;
        sregs.fs = seg;
        sregs.gs = seg;
        sregs.ss = seg;

        vcpu.set_kvm_sregs(&sregs).unwrap();

        let mut regs = vcpu.get_kvm_regs().unwrap();
        regs.rdx = 0x663;  // cpuid version
        regs.rip = 0xfff0;

        regs.rflags = 0x2;

        vcpu.set_kvm_regs(&regs).unwrap();
    }
}

impl Accelerator for KVMAccelerator {
    fn init_vcpu(&mut self) {
        let vcpu = self.vm.create_vcpu().unwrap();

        self.setup_cpuid(&vcpu);
        self.setup_msrs(&vcpu);
        self.init_regs(&vcpu);

        self.vcpus.push(vcpu);
    }

    fn vcpu_run(&mut self, vcpu_index: usize) -> VcpuExit {
        let ref mut vcpu = self.vcpus[vcpu_index];

        vcpu.run().unwrap();
        let kvm_run = vcpu.kvm_run_mut();
        match kvm_run.exit_reason {
            KVM_EXIT_HLT => {
                VcpuExit::Hlt
            }
            KVM_EXIT_MMIO => {
                let mmio = unsafe { &mut kvm_run.__bindgen_anon_1.mmio };
                let addr = mmio.phys_addr;
                let len = mmio.len as usize;
                let data = &mut mmio.data[..len];
                if mmio.is_write != 0 {
                    VcpuExit::MmioWrite(addr, data)
                }
                else {
                    VcpuExit::MmioRead(addr, data)
                }
            }
            KVM_EXIT_IO => {
                let io = unsafe { kvm_run.__bindgen_anon_1.io };
                // todo: validate offset
                let port = io.port;
                let addr = kvm_run as *mut _ as u64 + io.data_offset;
                let data_size = io.size as u32 * io.count;
                let data = unsafe {
                    std::slice::from_raw_parts_mut(
                        addr as *mut u8, data_size as usize)
                };

                match io.direction as u32 {
                    KVM_EXIT_IO_IN => VcpuExit::IoIn(port, data),
                    KVM_EXIT_IO_OUT => VcpuExit::IoOut(port, data),
                    _ => panic!("Invalid IO direction.")
                }
            }
            KVM_EXIT_SHUTDOWN => {
                VcpuExit::Shutdown
            }
            KVM_EXIT_INTERNAL_ERROR => {
                unsafe {
                    let suberr = kvm_run.__bindgen_anon_1.internal.suberror;
                    let data_len = kvm_run.__bindgen_anon_1.internal.ndata
                                   as usize;

                    panic!(
                        "KVM internal error: {}. Extra data: {:#?}",
                        suberr,
                        &mut kvm_run.__bindgen_anon_1.internal.data[..data_len]);
                }
            }
            _ => {
                panic!("Not supported exit reason: {}", kvm_run.exit_reason);
            }
        }
    }

    fn memory_region_add(&self, mem: &MmapMemorySlot) {
        self.vm.set_user_memory_region(mem).unwrap();
    }
}
