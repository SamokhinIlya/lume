use std::alloc::{alloc, realloc, Layout, LayoutError, handle_alloc_error};
use std::mem::{size_of, align_of};
use std::ops::{Index, IndexMut};
use std::ffi::c_void;

use num_traits::cast;
use windows::Win32::Graphics::Gdi::{BITMAPINFO, BITMAPINFOHEADER, BI_RGB};

pub type BitmapData = u32;

pub struct Bitmap {
    ptr: *mut BitmapData,
    width: usize,
    height: usize,
    info: BITMAPINFO,
}

pub trait RawCanvas: IndexMut<usize, Output=u32> {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

impl RawCanvas for Bitmap {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl Index<usize> for Bitmap {
    type Output = BitmapData;

    fn index(&self, index: usize) -> &Self::Output {
        let size = self.size();
        assert!(index <= size, "index = {index}, size = {size}");

        unsafe { &*self.ptr.add(index) }
    }
}

impl IndexMut<usize> for Bitmap {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let size = self.size();
        assert!(index <= size, "index = {index}, size = {size}");

        unsafe { &mut *self.ptr.add(index) }
    }
}

impl Bitmap {
    pub fn with_size(width: usize, height: usize) -> Result<Self, LayoutError> {
        let layout = Layout::from_size_align(width * height * size_of::<BitmapData>(), align_of::<BitmapData>())?;
        let ptr: *mut BitmapData = unsafe { alloc(layout) }.cast();
        if ptr.is_null() {
            handle_alloc_error(layout);
        }

        Ok(Self {
            ptr,
            width,
            height,
            info: BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: cast(size_of::<BITMAPINFOHEADER>()).unwrap(),
                    biWidth: cast(width).unwrap(),
                    biHeight: -cast::<_, i32>(height).unwrap(),
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB as _,
                    ..Default::default()
                },
                ..Default::default()
            },
        })
    }

    pub fn width(&self) -> i32 {
        cast(self.width).unwrap()
    }

    pub fn height(&self) -> i32 {
        cast(self.height).unwrap()
    }

    pub fn data(&self) -> Option<*const c_void> {
        Some(self.ptr.cast())
    }

    pub fn info(&self) -> *const BITMAPINFO {
        &self.info
    }

    fn size(&self) -> usize {
        self.width * self.height
    }

    pub fn resize(&mut self, width: usize, height: usize) -> Result<(), LayoutError> {
        let layout = Layout::from_size_align(self.width * self.height * size_of::<BitmapData>(), align_of::<BitmapData>())?;
        let ptr: *mut BitmapData =
            unsafe { realloc(self.ptr.cast(), layout, width * height * size_of::<BitmapData>()) }.cast();
        if ptr.is_null() {
            handle_alloc_error(layout);
        }

        self.ptr = ptr;
        self.width = width;
        self.height = height;
        self.info.bmiHeader.biWidth = cast(width).unwrap();
        self.info.bmiHeader.biHeight = -cast::<_, i32>(height).unwrap();
        Ok(())
    }
}