mod kvm;
mod base;

use self::base::Accelerator;
use self::kvm::KVMAccelerator;

pub fn new() -> Box<Accelerator> {
    // We only support KVM atm.
    Box::new(KVMAccelerator::new())
}
