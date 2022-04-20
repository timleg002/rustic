use scroll::{ctx, Pread, Endian, Pwrite};
use anyhow::Result;

const SEGMENT_BITS: u8 = 0x7F;
const CONTINUE_BIT: u8 = 0x80;

#[derive(Debug, PartialEq)]
pub struct VarInt(pub i32);

impl<'a> ctx::TryFromCtx<'a> for VarInt {
    type Error = scroll::Error;

    // the `usize` returned here is the amount of bytes read. 
    fn try_from_ctx(src: &'a [u8], _: ()) -> Result<(Self, usize), Self::Error> {
        let mut value = 0;
        let mut byte_position = 0;
        let mut offset = 0;
        
        loop {
            // VarInts are always LE.
            let byte = src.gread_with::<u8>(&mut offset, Endian::Little)?;

            value |= ((byte & SEGMENT_BITS) as i32) << byte_position;

            if (byte & CONTINUE_BIT) == 0 {
                break;
            }

            byte_position += 7;

            if byte_position >= 32 {
                return Err(Self::Error::TooBig {
                    size: value as usize, 
                    len: src.len() 
                });
            }
        } 

        Ok((VarInt(value), offset))
    }
}

impl<'a> ctx::TryIntoCtx for VarInt {
    type Error = scroll::Error;

    fn try_into_ctx(self, output: &mut [u8], _: ()) -> Result<usize, Self::Error> {
        let mut value = { // This is because we need the value bit by bit, including the sign
            let bytes = self.0.to_ne_bytes();
            u32::from_ne_bytes(bytes)
        };
        let mut offset = 0;
        
        // We need to negate all the extra bits too.
        while value & !(SEGMENT_BITS as u32) != 0 {
            let val_to_write = (value as u8 & SEGMENT_BITS) | CONTINUE_BIT;
            offset += output.pwrite_with(val_to_write, offset, Endian::Little)?;

            value >>= 7;
        }

        // Remember to always cast the byte to an u8, or else we'll be writing a value that'll be too big
        offset += output.pwrite_with(value as u8, offset, Endian::Little)?;

        Ok(offset)
    }
}
#[derive(Debug, PartialEq)]
pub struct VarLong(pub i64);

impl<'a> ctx::TryFromCtx<'a> for VarLong {
    type Error = scroll::Error;

    // the `usize` returned here is the amount of bytes read. 
    fn try_from_ctx(src: &'a [u8], _: ()) -> Result<(Self, usize), Self::Error> {
        let mut value = 0;
        let mut byte_position = 0;
        let mut offset = 0;
        
        loop {
            let byte = src.gread_with::<u8>(&mut offset, Endian::Little)?;

            value |= ((byte & SEGMENT_BITS) as i64) << byte_position;

            if byte & CONTINUE_BIT == 0 {
                break;
            }

            byte_position += 7;

            if byte_position >= 64 {
                return Err(Self::Error::TooBig {
                    size: value as usize, 
                    len: src.len() 
                });
            }
        } 

        Ok((VarLong(value), offset))
    }
}

impl<'a> ctx::TryIntoCtx for VarLong {
    type Error = scroll::Error;

    fn try_into_ctx(self, output: &mut [u8], _: ()) -> Result<usize, Self::Error> {
        let mut value = { // This is because we need the value bit by bit, including the sign
            let bytes = self.0.to_ne_bytes();
            u64::from_ne_bytes(bytes)
        };
        let mut offset = 0;
        
        // We need to negate all the extra bits too.
        while value & !(SEGMENT_BITS as u64) != 0 {
            let val_to_write = (value as u8 & SEGMENT_BITS) | CONTINUE_BIT;
            offset += output.pwrite_with(val_to_write, offset, Endian::Little)?;

            value >>= 7;
        }

        // Remember to always cast the byte to an u8, or else we'll be writing a value that'll be too big
        offset += output.pwrite_with(value as u8, offset, Endian::Little)?;

        Ok(offset)
    }
}

#[cfg(test)]
mod tests {
    use scroll::*;
    use anyhow::Result;

    use super::*;

    #[test]
    fn varints_read_test() -> Result<()> {
        // Taken from https://wiki.vg/Protocol#VarInt_and_VarLong
        let vals = [
            (0, vec![0x00]),	
            (1, vec![0x01]),	
            (2, vec![0x02]),	
            (127, vec![0x7f]),	
            (128, vec![0x80, 0x01]),
            (255, vec![0xff, 0x01]),
            (25565, vec![0xdd, 0xc7, 0x01]),
            (2097151, vec![0xff, 0xff, 0x7f]),
            (2147483647, vec![0xff, 0xff, 0xff, 0xff, 0x07]),
            (-1, vec![0xff, 0xff, 0xff, 0xff, 0x0f]),
            (-2147483648, vec![0x80, 0x80, 0x80, 0x80, 0x08])
        ];

        for (expected_value, bytes) in vals {
            let result = bytes.pread::<VarInt>(0)?;

            assert_eq!(expected_value, result.0);
        }

        Ok(())
    }

    #[test]
    fn varints_write_test() -> Result<()> {
        let vals = [
            (0, vec![0x00]),	
            (1, vec![0x01]),	
            (2, vec![0x02]),	
            (127, vec![0x7f]),	
            (128, vec![0x80, 0x01]),
            (255, vec![0xff, 0x01]),
            (25565, vec![0xdd, 0xc7, 0x01]),
            (2097151, vec![0xff, 0xff, 0x7f]),
            (2147483647, vec![0xff, 0xff, 0xff, 0xff, 0x07]),
            (-1, vec![0xff, 0xff, 0xff, 0xff, 0x0f]),
            (-2147483648, vec![0x80, 0x80, 0x80, 0x80, 0x08])
        ];

        for (result, expected_value) in vals {
            let mut bytes = vec![0; expected_value.len()];

            bytes.pwrite(VarInt(result), 0)?;

            assert_eq!(bytes, expected_value);
        }

        Ok(())
    }

    #[test]
    fn varlongs_read_test() -> Result<()> {
        // Taken from https://wiki.vg/Protocol#VarInt_and_VarLong
        let vals = [
            (0, vec![0x00]),	
            (1, vec![0x01]),
            (2, vec![0x02]),
            (127, vec![0x7f]),
            (128, vec![0x80, 0x01]),
            (255, vec![0xff, 0x01]),
            (2147483647, vec![0xff, 0xff, 0xff, 0xff, 0x07]),
            (9223372036854775807, vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f]),
            (-1, vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]),
            (-2147483648, vec![0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01]),
            (-9223372036854775808, vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01])
        ];

        for (expected_value, bytes) in vals {
            let result = bytes.pread::<VarLong>(0)?;

            assert_eq!(expected_value, result.0);
        }

        Ok(())
    }

    #[test]
    fn varlongs_write_test() -> Result<()> {
        // Taken from https://wiki.vg/Protocol#VarInt_and_VarLong
        let vals = [
            (0, vec![0x00]),	
            (1, vec![0x01]),
            (2, vec![0x02]),
            (127, vec![0x7f]),
            (128, vec![0x80, 0x01]),
            (255, vec![0xff, 0x01]),
            (2147483647, vec![0xff, 0xff, 0xff, 0xff, 0x07]),
            (9223372036854775807, vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f]),
            (-1, vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]),
            (-2147483648, vec![0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01]),
            (-9223372036854775808, vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01])
        ];

        for (result, expected_value) in vals {
            let mut bytes = vec![0; expected_value.len()];

            bytes.pwrite(VarLong(result), 0)?;

            assert_eq!(bytes, expected_value);
        }

        Ok(())
    }



}