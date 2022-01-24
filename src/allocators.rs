use core::alloc::Layout;
use core::mem::MaybeUninit;

pub struct BoundArena<'data, const SIZE: usize> {
    data: &'data mut [MaybeUninit<u8>; SIZE],
    end: usize,
}

impl<'data, const SIZE: usize> BoundArena<'data, SIZE> {
    pub const fn new(data: &'data mut [MaybeUninit<u8>; SIZE]) -> Self {
        BoundArena { data, end: 0 }
    }

    pub fn alloc<'a, T>(&mut self, val: T) -> Option<&'a mut T>
    where
        'data: 'a,
    {
        Some(self.raw_alloc()?.write(val))
    }

    pub fn raw_alloc<'a, T>(&mut self) -> Option<&'a mut MaybeUninit<T>>
    where
        'data: 'a,
    {
        let layout = Layout::new::<T>();
        let size = layout.size();
        let align = layout.align();
        let align_mask = !(align - 1);
        let start = (self.end + align - 1) & align_mask;

        if start + size > self.data.len() {
            None
        } else {
            unsafe {
                // SAFETY: We have asserted in the if statement that `start` is in bounds.
                let ptr = self.data.as_mut_ptr().add(start).cast::<MaybeUninit<T>>();
                self.end = start + size;
                Some(&mut *ptr)
            }
        }
    }
}

const KERNEL_ARENA_SIZE: usize = 4 * 1024;

pub static mut KERNEL_HEAP: [MaybeUninit<u8>; KERNEL_ARENA_SIZE] = MaybeUninit::uninit_array();
pub static mut KERNEL_ARENA: BoundArena<'static, KERNEL_ARENA_SIZE> = unsafe { BoundArena::new(&mut KERNEL_HEAP) };

#[derive(Debug)]
pub struct KBox<'a, T: ?Sized>(&'a mut T);

impl<'a, T> KBox<'a, T> {
    /// # Safety
    ///
    /// This operation is unsafe because it uses the global `KERNEL_ARENA` and
    /// may cause a race condition if run in a multithreaded context.
    pub unsafe fn new(val: T) -> Self {
        KBox(
            KERNEL_ARENA
                .alloc(val)
                .expect("failed to allocate `KBox` (out of memory)")
                .into(),
        )
    }
}

use core::fmt::{ self, Display };
use core::ops::{ CoerceUnsized, DispatchFromDyn, Deref, DerefMut };
use core::convert::{ AsRef, AsMut };
use core::marker::Unsize;

impl<'a, T: ?Sized + Display> Display for KBox<'a, T> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self.as_ref(), f)
    }
}

impl<'a, T: ?Sized> Deref for KBox<'a, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        self.0
    }
}

impl<'a, T: ?Sized> DerefMut for KBox<'a, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}

impl<'a, T: ?Sized> AsRef<T> for KBox<'a, T> {
    #[inline(always)]
    fn as_ref(&self) -> &T {
        self.0
    }
}

impl<'a, T: ?Sized> AsMut<T> for KBox<'a, T> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut T {
        self.0
    }
}

impl<'a, T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<KBox<'a, U>> for KBox<'a, T> {}

// FIXME: I don't really know what this is for. So there might be a bug here.
impl<'a, T: ?Sized + Unsize<U>, U: ?Sized> DispatchFromDyn<KBox<'a, U>> for KBox<'a, T> {}
