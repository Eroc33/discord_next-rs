use std::{
    io::{self,Read},
    ffi::OsStr
};
use byteorder::{LittleEndian,ByteOrder};
use crate::voice::AudioStream;

/// Attribution: Copied from the "byteorder" crate (public domain)
/// 
/// Convert a slice of T (where T is plain old data) to its mutable binary
/// representation.
///
/// This function is wildly unsafe because it permits arbitrary modification of
/// the binary representation of any `Copy` type. Use with care.
unsafe fn slice_to_u8_mut<T: Copy>(slice: &mut [T]) -> &mut [u8] {
    use std::mem::size_of;

    let len = size_of::<T>() * slice.len();
    std::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut u8, len)
}

//like read exact, but does not throw UnexpectedEof if the read does fill the buffer
//also will return the number of bytes read
fn try_fill_u8_from(r: &mut dyn Read, mut buf: &mut [u8]) -> Result<usize,io::Error>{
    let buf_cap = buf.len();
    while !buf.is_empty() {
        match r.read(buf) {
            Ok(0) => break,
            Ok(n) => { let tmp = buf; buf = &mut tmp[n..]; }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    let buf_remaining = buf.len();
    Ok(buf_cap-buf_remaining)
}

//like byteorder ReadBytesExt::read_i16_into, but does not throw UnexpectedEof if the read does fill the buffer
fn try_fill_i16_from<T: ByteOrder>(r: &mut dyn Read, dst: &mut[i16]) -> Result<usize, io::Error>{
    let read = {
        let buf = unsafe{ slice_to_u8_mut(dst) };
        try_fill_u8_from(r,buf)?
    };
    //TODO: make this a result of some sort
    //read bytes must be a multiple of 2 for slicing as an i16 to make sense
    assert!(read%2==0);
    T::from_slice_i16(dst);
    Ok(read/2)
}

pub struct FfmpegStream{
    process: std::process::Child,
    is_stereo: bool,
}

impl AudioStream for FfmpegStream {
	fn read_frame(&mut self, buffer: &mut [i16]) -> Result<usize,io::Error>{
		try_fill_i16_from::<LittleEndian>(self.process.stdout.as_mut().expect("missing stdout"),buffer)
    }
    fn is_stereo(&self) -> bool{
        self.is_stereo
    }
}

impl FfmpegStream{
    pub fn open<P: AsRef<OsStr>>(path: P, volume: Option<f32>, is_stereo: bool) -> Result<Self,io::Error>
    {
        use std::process::{Command, Stdio};
        let path = path.as_ref();
        let child = Command::new("ffmpeg")
            .arg("-i").arg(path)
            .args(&[
                "-af", format!("volume={}",volume.unwrap_or(1.0)).as_str(),
                "-f", "s16le",
                "-ac", if is_stereo { "2" } else { "1" } ,
                "-ar", "48000",
                "-acodec", "pcm_s16le",
                "-"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(Self{
            process: child,
            is_stereo,
        })
    }
}

impl Drop for FfmpegStream {
	fn drop(&mut self) {
		// If we can't kill it, it's dead already or out of our hands
		let _ = self.process.kill();
		// To avoid zombie processes, we must also wait on it
		let _ = self.process.wait();
	}
}