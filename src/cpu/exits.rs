#![allow(non_camel_case_types)]

#[derive(Debug)]
pub enum VcpuExit<'a> {
    /// An out port instruction was run on the given port with the given data.
    IoOut(u16 /* port */, &'a [u8] /* data */),
    /// An in port instruction was run on the given port.
    ///
    /// The given slice should be filled in before `Vcpu::run` is called again.
    IoIn(u16 /* port */, &'a mut [u8] /* data */),
    /// A read instruction was run against the given MMIO address.
    ///
    /// The given slice should be filled in before `Vcpu::run` is called again.
    MmioRead(u64 /* address */, &'a mut [u8]),
    /// A write instruction was run against the given MMIO address with the given data.
    MmioWrite(u64 /* address */, &'a [u8]),
    Hlt,
    Shutdown
}
