//! A module for writing integers encoded as LEB128.

use super::{CONTINUATION_BIT, low_bits_of_u64};
use std::io;

/// Write the given unsigned number using the LEB128 encoding to the given
/// `std::io::Write`able. Returns the number of bytes written to `w`, or an
/// error if writing failed.
pub fn unsigned<W>(w: &mut W, mut val: u64) -> Result<usize, io::Error>
    where W: io::Write
{
    let mut bytes_written = 0;
    loop {
        let mut byte = low_bits_of_u64(val);
        val >>= 7;
        if val != 0 {
            // More bytes to come, so set the continuation bit.
            byte |= CONTINUATION_BIT;
        }

        let buf = [byte];
        try!(w.write_all(&buf));
        bytes_written += 1;

        if val == 0 {
            return Ok(bytes_written);
        }
    }
}

/// Write the given signed number using the LEB128 encoding to the given
/// `std::io::Write`able. Returns the number of bytes written to `w`, or an
/// error if writing failed.
pub fn signed<W>(w: &mut W, mut val: i64) -> Result<usize, io::Error>
    where W: io::Write
{
    let mut bytes_written = 0;
    loop {
        let mut byte = val as u8;
        // Keep the sign bit for testing
        val >>= 6;
        let done = val == 0 || val == -1;
        if done {
            byte &= !CONTINUATION_BIT;
        } else {
            // Remove the sign bit
            val >>= 1;
            // More bytes to come, so set the continuation bit.
            byte |= CONTINUATION_BIT;
        }

        let buf = [byte];
        try!(w.write_all(&buf));
        bytes_written += 1;

        if done {
            return Ok(bytes_written);
        }
    }
}
