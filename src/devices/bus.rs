// Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Handles routing to devices in an address space.

#![allow(dead_code)]

use std::cmp::{Ord, PartialOrd, PartialEq, Ordering};
use std::collections::btree_map::BTreeMap;
use std::result;
use std::sync::{Arc, Mutex};

/// Trait for devices that respond to reads or writes in an arbitrary address space.
///n offset
/// into its allocated por
/// The device does not care where it exists in address space as each method is only given ation of address space.
#[allow(unused_variables)]
pub trait BusDevice: Send {
    /// Reads at `offset` from this device
    fn read(&mut self, offset: u64, data: &mut [u8]) {}
    /// Writes at `offset` into this device
    fn write(&mut self, offset: u64, data: &[u8]) {}
    /// Sets a register in the configuration space. Only used by PCI.
    /// * `reg_idx` - The index of the config register to modify.
    /// * `offset` - Offset in to the register.
    fn config_register_write(&mut self, reg_idx: usize, offset: u64, data: &[u8]) {}
    /// Gets a register from the configuration space. Only used by PCI.
    /// * `reg_idx` - The index of the config register to read.
    fn config_register_read(&self, reg_idx: usize) -> u32 { 0 }
}

#[derive(Debug)]
pub enum Error {
    /// The insertion failed because the new device overlapped with an old device.
    Overlap,
}

pub type Result<T> = result::Result<T, Error>;

/// Holds a base and length representing the address space occupied by a `BusDevice`.
///
/// * base - The address at which the range start.
/// * len - The length of the range in bytes.
/// * full_addr - If true, return the full address from `get_device`, otherwise return the offset
///               from `base`
#[derive(Debug, Copy, Clone)]
pub struct BusRange {
    pub base: u64,
    pub len: u64,
    pub full_addr: bool,
}

impl BusRange {
    /// Returns true if `addr` is within the range.
    pub fn contains(&self, addr: u64) -> bool {
        self.base <= addr && addr < self.base + self.len
    }

    /// Returns true if there is overlap with the given range.
    pub fn overlaps(&self, base: u64, len: u64) -> bool {
        self.base < (base + len) && base < self.base + self.len
    }
}

impl Eq for BusRange {}

impl PartialEq for BusRange {
    fn eq(&self, other: &BusRange) -> bool {
        self.base == other.base
    }
}

impl Ord for BusRange {
    fn cmp(&self, other: &BusRange) -> Ordering {
        self.base.cmp(&other.base)
    }
}

impl PartialOrd for BusRange {
    fn partial_cmp(&self, other: &BusRange) -> Option<Ordering> {
        self.base.partial_cmp(&other.base)
    }
}

/// A device container for routing reads and writes over some address space.
///
/// This doesn't have any restrictions on what kind of device or address space this applies to. The
/// only restriction is that no two devices can overlap in this address space.
#[derive(Clone)]
pub struct Bus {
    devices: BTreeMap<BusRange, Arc<Mutex<BusDevice>>>,
}

impl Bus {
    /// Constructs an a bus with an empty address space.
    pub fn new() -> Bus {
        Bus { devices: BTreeMap::new() }
    }

    fn first_before(&self, addr: u64) -> Option<(BusRange, &Mutex<BusDevice>)> {
        let(range, dev) =  self.devices.range(..=BusRange {base:addr, len:1, full_addr: false})
                                       .rev()
                                       .next()?;
        Some((*range, dev))
    }

    fn get_device(&self, addr: u64) -> Option<(u64, &Mutex<BusDevice>)> {
        if let Some((range, dev)) = self.first_before(addr) {
            let offset = addr - range.base;
            if offset < range.len {
                if range.full_addr {
                    return Some((addr, dev));
                } else {
                    return Some((offset, dev));
                }
            }
        }
        None
    }

    /// Puts the given device at the given address space.
    pub fn insert(&mut self, device: Arc<Mutex<BusDevice>>,
                  base: u64, len: u64, full_addr: bool)
        -> Result<()>
    {
        if len == 0 {
            return Err(Error::Overlap);
        }

        // Reject all cases where the new device's range overlaps with an existing device.
        if self.devices.iter().any(|(range, _dev)| range.overlaps(base, len)) {
            return Err(Error::Overlap);
        }

        if self.devices
               .insert(BusRange{base, len, full_addr}, device)
               .is_some() {
            return Err(Error::Overlap);
        }

        Ok(())
    }

    /// Reads data from the device that owns the range containing `addr` and puts it into `data`.
    ///
    /// Returns true on success, otherwise `data` is untouched.
    pub fn read(&self, addr: u64, data: &mut [u8]) -> bool {
        if let Some((offset, dev)) = self.get_device(addr) {
            dev.lock().unwrap().read(offset, data);
            true
        } else {
            false
        }
    }

    /// Writes `data` to the device that owns the range containing `addr`.
    ///
    /// Returns true on success, otherwise `data` is untouched.
    pub fn write(&self, addr: u64, data: &[u8]) -> bool {
        if let Some((offset, dev)) = self.get_device(addr) {
            dev.lock().unwrap().write(offset, data);
            true
        } else {
            false
        }
    }
}

