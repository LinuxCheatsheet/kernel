//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
use core::mem;

/// A page table entry.
pub struct Entry(u64);

bitflags! {
    pub flags Flags: u64 {
        /// Present flag.
        /// Must be 1 to map a 2-MByte page or reference a page table.
        const PRESENT =         1 << 0,
        /// Writable flag.
        /// If 0, writes may not be allowed to the 2-MB region controlled
        /// by this entry
        const WRITABLE =        1 << 1
      , const USER_ACCESSIBLE = 1 << 2
      , const WRITE_THROUGH =   1 << 3
      , const NO_CACHE =        1 << 4
      , const ACCESSED =        1 << 5
      , const DIRTY =           1 << 6
      , const HUGE_PAGE =       1 << 7
      , const GLOBAL =          1 << 8
      , const NO_EXECUTE =      1 << 63
    }
}

impl Entry {

    /// Returns true if this is an unused entry
    #[inline]
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    /// Sets this entry to be unused
    #[inline]
    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    /// Returns true if this page is huge
    #[inline]
    pub fn is_huge(&self) -> bool {
        self.flags().contains(HUGE_PAGE)
    }

    /// Access the entry's bitflags.
    #[inline]
    pub fn flags(&self) -> Flags {
        Flags::from_bits_truncate(self.0)
    }

    pub fn pointed_frame(&self) -> Option<*mut u8> {
        unsafe {
            if self.flags().contains(PRESENT) {
                Some(mem::transmute(self.0 & 0x000fffff_fffff000))
            } else { None }
        }
    }

    pub fn set(&mut self, frame: *mut u8, flags: Flags) {
        assert!(frame as u64 & !0x000fffff_fffff000 == 0);
        self.0 = (frame as u64) | flags.bits();
    }

}