use std::{
    fmt::Debug,
    io::{self, BufRead, BufReader, Read, Seek},
    marker::PhantomData,
    mem, slice,
};

use glam::{Vec2, Vec3, Vec4};

pub trait BinaryData {
    fn read<R: Read + Seek>(buffer: &mut BufReader<R>, _max_size: Option<usize>) -> io::Result<Self>
    where
        Self: Sized,
    {
        let mut data = unsafe { mem::zeroed() };

        let size = mem::size_of::<Self>();
        unsafe {
            let slice = slice::from_raw_parts_mut(&mut data as *mut _ as *mut u8, size);
            // `read_exact()` comes from `Read` impl for `&[u8]`
            buffer.read_exact(slice)?;
        }
        Ok(data)
    }

    fn read_array<R: Read + Seek>(
        buffer: &mut BufReader<R>,
        count: usize,
        _max_size: Option<usize>,
    ) -> io::Result<Box<[Self]>>
    where
        Self: Sized + bytemuck::Zeroable + bytemuck::Pod,
    {
        let mut header = bytemuck::zeroed_slice_box(count);

        if count > 0 {
            unsafe {
                let size = count * mem::size_of::<Self>();
                let slice = slice::from_raw_parts_mut(&mut header[0] as *mut _ as *mut u8, size);
                // `read_exact()` comes from `Read` impl for `&[u8]`
                buffer.read_exact(slice)?;
            }
        }

        Ok(header)
    }
}

impl<T: bytemuck::Zeroable> BinaryData for T {}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable)]
pub struct BinOffset {
    pub index: u32,
}

impl BinOffset {
    pub fn seek_start<R: Read + Seek>(
        &self,
        buffer: &mut BufReader<R>,
        start: i64,
        pos: &mut i64,
    ) -> io::Result<()> {
        let p = self.index as i64 + start;
        buffer.seek_relative(p - *pos)?;
        *pos = p;
        Ok(())
    }
    pub fn read_str<R: Read + Seek>(
        &self,
        buffer: &mut BufReader<R>,
        start: i64,
        pos: &mut i64,
    ) -> io::Result<String> {
        self.seek_start(buffer, start, pos)?;
        //let mut b = [0, 0, 0, 0];
        //buffer.read_exact(&mut b)?;
        //println!("{b:?}");
        let mut data = Default::default();

        *pos += buffer.read_until(0, &mut data)? as i64;

        // Remove trailing 0
        data.pop();

        Ok(String::from_utf8(data).unwrap())
    }
    //TODO: choose name
    pub fn read_array_f<
        T: Sized + bytemuck::Zeroable + bytemuck::Pod + BinaryData,
        R: Read + Seek,
    >(
        &self,
        buffer: &mut BufReader<R>,
        start: i64,
        pos: &mut i64,
        count: usize,
    ) -> io::Result<Box<[T]>> {
        self.seek_start(buffer, start, pos)?;

        //let mut b = [0, 0, 0, 0];
        //buffer.read_exact(&mut b)?;
        //println!("{b:?}");

        let b = T::read_array(buffer, count, None)?;

        *pos += (count * mem::size_of::<T>()) as i64;
        Ok(b)
    }
    pub fn read_array<T: Sized + BinaryData, R: Read + Seek>(
        &self,
        buffer: &mut BufReader<R>,
        start: i64,
        pos: &mut i64,
        count: u32,
    ) -> io::Result<Vec<(i64, T)>> {
        self.seek_start(buffer, start, pos)?;

        //let mut b = [0, 0, 0, 0];
        //buffer.read_exact(&mut b)?;
        //println!("{b:?}");

        let mut v = Vec::new();
        v.reserve_exact(self.index as usize);

        for _ in 0..count {
            v.push((*pos, T::read(buffer, None)?));
            *pos += mem::size_of::<T>() as i64;
        }

        Ok(v)
    }
}

/// Struct of (count, offset) for reading an array of items from an mdl
#[repr(C, packed)]
#[derive(Debug, bytemuck::Zeroable)]
pub struct BinArray<T: Sized + BinaryData> {
    pub count: u32,
    pub offset: BinOffset,
    _p: PhantomData<T>,
}

impl<T: Sized + BinaryData> BinArray<T> {
    pub fn read<R: Read + Seek>(
        &self,
        buffer: &mut BufReader<R>,
        start: i64,
        pos: &mut i64,
    ) -> io::Result<Vec<(i64, T)>> {
        self.offset.read_array(buffer, start, pos, self.count)
    }
}
impl<T: Sized + BinaryData + bytemuck::Pod> BinArray<T> {
    pub fn read_f<R: Read + Seek>(
        &self,
        buffer: &mut BufReader<R>,
        start: i64,
        pos: &mut i64,
    ) -> io::Result<Box<[T]>> {
        self.offset
            .read_array_f(buffer, start, pos, self.count as usize)
    }
}
