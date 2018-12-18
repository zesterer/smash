// Standard
use core::ptr;

// Library
use allocator_api::RawVec;

pub trait RawVecAccess<T> {
    #[inline(always)]
    unsafe fn get(&self, idx: usize) -> T;
    #[inline(always)]
    unsafe fn set(&self, idx: usize, val: T);
    #[inline(always)]
    unsafe fn entry(&self, idx: usize) -> &T;
    #[inline(always)]
    unsafe fn entry_mut(&self, idx: usize) -> &mut T;
}

impl<T> RawVecAccess<T> for RawVec<T> {
    #[inline(always)]
    unsafe fn get(&self, idx: usize) -> T {
        ptr::read((self.ptr() as *const T).add(idx))
    }

    #[inline(always)]
    unsafe fn set(&self, idx: usize, val: T) {
        ptr::write((self.ptr() as *mut T).add(idx), val)
    }

    #[inline(always)]
    unsafe fn entry(&self, idx: usize) -> &T {
        &*(self.ptr() as *const T).add(idx)
    }

    #[inline(always)]
    unsafe fn entry_mut(&self, idx: usize) -> &mut T {
        &mut *(self.ptr() as *mut T).add(idx)
    }
}
