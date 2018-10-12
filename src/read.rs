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

/// Read an unsigned LEB128 number from the given `std::io::Read`able and
/// return it or an error if reading failed.
#[inline]
pub fn unsigned(mut input: &[u8]) -> Result<(u64, &[u8]), Error> {
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
