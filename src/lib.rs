#![doc = include_str!("../README.md")]

use memmap2::{MmapMut, MmapOptions};
use std::{marker::PhantomData, path::Path};

/// A wrapper wrapper for a memory-mapped file with data of type `T`.
///
/// # Safety
///
/// `T` must have a consistent memory layout to ensure that the data is casted correctly.
///
/// Use `#[repr(transparent)]` if `T` is a newtype wrapper around a single field otherwise `#[repr(C)]`.
///
/// # Example
/// ```rust
/// use mmapcell::MmapCell;
///
/// #[repr(C)]
/// struct MyStruct {
///    thing1: i32,
///    thing2: f64,
/// }
///
/// let cell = unsafe {
///     MmapCell::<MyStruct>::new_named("/tmp/mystruct-mmap-test.bin")
/// }.unwrap();
///
/// let mmap_backed_mystruct = cell.get_mut();
///
/// mmap_backed_mystruct.thing1 = 3;
/// ```
#[repr(transparent)]
pub struct MmapCell<T> {
    raw: MmapMut,
    _inner: PhantomData<T>,
}

impl<T> Drop for MmapCell<T> {
    fn drop(&mut self) {
        // this probably happens anyways but just in case
        let _ = self.raw.flush();
    }
}

// WARN:
// i'm not sure i want to leave these in because
// it isn't super clear that these are VERY unsafe to call
//
//impl<T> TryFrom<Mmap> for MmapCell<T> {
//    type Error = std::io::Error;
//
//    fn try_from(m: Mmap) -> Result<MmapCell<T>, std::io::Error> {
//        Ok(unsafe { MmapCell::new(m.make_mut()?) })
//    }
//}
//
//impl<T> From<MmapMut> for MmapCell<T> {
//    fn from(m: MmapMut) -> MmapCell<T> {
//        unsafe { MmapCell::new(m) }
//    }
//}

impl<T> MmapCell<T> {
    /// # Safety
    /// the backing mmap pointer must point to valid
    /// memory for type T [T likely has to be repr(C)]
    pub unsafe fn new(m: MmapMut) -> MmapCell<T> {
        // check that size of m matches
        // size of inner type
        MmapCell {
            raw: m,
            _inner: PhantomData,
        }
    }

    pub fn new_anon() -> Result<MmapCell<T>, std::io::Error> {
        Ok(unsafe { MmapCell::new(MmapOptions::new().len(size_of::<T>()).map_anon()?) })
    }

    /// # Safety
    /// the backing mmap pointer must point to valid
    /// memory for type T [T likely has to be repr(C)]
    pub unsafe fn new_named<P: AsRef<Path>>(path: P) -> Result<MmapCell<T>, std::io::Error> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        file.set_len(size_of::<T>() as u64)?;

        let m = unsafe { MmapMut::map_mut(&file)? };
        Ok(unsafe { MmapCell::new(m) })
    }

    /// # Safety
    /// the backing mmap pointer must point to valid
    /// memory for type T [T likely has to be repr(C)]
    pub unsafe fn open_named<P: AsRef<Path>>(path: P) -> Result<MmapCell<T>, std::io::Error> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .truncate(false)
            .open(path)?;

        let m = unsafe { MmapMut::map_mut(&file)? };

        Ok(unsafe { MmapCell::new(m) })
    }

    pub fn get<'a>(&self) -> &'a T {
        unsafe { &*self.raw.as_ptr().cast::<T>() }
    }

    pub fn get_mut<'a>(&self) -> &'a mut T {
        unsafe { &mut *self.raw.as_ptr().cast_mut().cast::<T>() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestStruct {
        thing1: i32,
    }

    #[test]
    fn anon_mmapcell() {
        let anon_cell = MmapCell::<TestStruct>::new_anon().unwrap();
        anon_cell.get_mut().thing1 = 3;

        assert!(anon_cell.get().thing1 == 3);
    }
}
