//! A module for reading signed and unsigned integers that have been LEB128
//! encoded.

use super::{CONTINUATION_BIT, SIGN_BIT, low_bits_of_byte};
use std::fmt;
use std::io;

/// An enumeration of the possible errors that can occur when reading a
/// number encoded with LEB128.
#[derive(Debug)]
pub enum Error {
    /// There was an underlying IO error.
    IoError(io::Error),
    /// The number being read is larger than can be represented.
    Overflow,
    /// Not enough input data.
    NotEnoughData,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IoError(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f,
               "leb128::read::Error: {}",
               ::std::error::Error::description(self))
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IoError(ref e) => e.description(),
            Error::Overflow => "The number being read is larger than can be represented",
            Error::NotEnoughData => "Not enough input data",
        }
    }

    fn cause(&self) -> Option<&::std::error::Error> {
        match *self {
            Error::IoError(ref e) => Some(e),
            Error::Overflow => None,
            Error::NotEnoughData => None,
        }
    }
}

#[inline]
fn num_with_continuation_bit(x: u64) -> u32 {
    let y = x | 0x7f7f7f7f7f7f7f7f;
    let y = !y;
    y.trailing_zeros() >> 3
}

/// TODO
#[inline]
pub fn unsigned(input: &[u8]) -> Result<(u64, &[u8]), Error> {
    use std::arch::x86_64;

    if input.len() < 8 {
        return unsigned_slow(input);
    }

    let mut word: u64 = 0;
    {
        let word: &mut [u8; 8] = unsafe { &mut *(&mut word as *mut u64 as *mut [u8; 8]) };
        word.copy_from_slice(&input[..8]);
    }

    // // This code is slower, despite being less branchy
    //
    // let n = num_with_continuation_bit(word);
    // if n == 8 {
    //     return unsigned_slow(input);
    // }
    //
    // let mask = 0x7f7f7f7f7f7f7f7f;
    // let mask = mask >> (7 * 8 - n * 8);
    // let rest = &input[n as usize + 1..];

    let (mask, rest) = match num_with_continuation_bit(word) {
        0 => (0x000000000000007f, &input[1..]),
        1 => (0x0000000000007f7f, &input[2..]),
        2 => (0x00000000007f7f7f, &input[3..]),
        3 => (0x000000007f7f7f7f, &input[4..]),
        4 => (0x0000007f7f7f7f7f, &input[5..]),
        5 => (0x00007f7f7f7f7f7f, &input[6..]),
        6 => (0x007f7f7f7f7f7f7f, &input[7..]),
        7 => (0x7f7f7f7f7f7f7f7f, &input[8..]),
        _ => return unsigned_slow(input),
    };


    let value = unsafe { x86_64::_pext_u64(word, mask) };
    Ok((value, rest))
}

/// Read an unsigned LEB128 number from the given `std::io::Read`able and
/// return it or an error if reading failed.
#[inline(never)]
fn unsigned_slow(mut input: &[u8]) -> Result<(u64, &[u8]), Error> {
    let mut result = 0;
    let mut shift = 0;

    loop {
        if input.is_empty() {
            return Err(Error::NotEnoughData);
        }

        let (buf, rest) = input.split_at(1);
        input = rest;

        if shift == 63 && buf[0] != 0x00 && buf[0] != 0x01 {
            return Err(Error::Overflow);
        }

        let low_bits = low_bits_of_byte(buf[0]) as u64;
        result |= low_bits << shift;

        if buf[0] & CONTINUATION_BIT == 0 {
            return Ok((result, input));
        }

        shift += 7;
    }
}

/// Read a signed LEB128 number from the given `std::io::Read`able and
/// return it or an error if reading failed.
pub fn signed<R>(r: &mut R) -> Result<i64, Error>
    where R: io::Read
{
    let mut result = 0;
    let mut shift = 0;
    let size = 64;
    let mut byte;

    loop {
        let mut buf = [0];
        try!(r.read_exact(&mut buf));

        byte = buf[0];
        if shift == 63 && byte != 0x00 && byte != 0x7f {
            return Err(Error::Overflow);
        }

        let low_bits = low_bits_of_byte(byte) as i64;
        result |= low_bits << shift;
        shift += 7;

        if byte & CONTINUATION_BIT == 0 {
            break;
        }
    }

    if shift < size && (SIGN_BIT & byte) == SIGN_BIT {
        // Sign extend the result.
        result |= !0 << shift;
    }

    Ok(result)
}
