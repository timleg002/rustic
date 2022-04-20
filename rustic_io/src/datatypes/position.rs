#[derive(Debug, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32
}

impl Position {
    pub fn from_u64(val: u64) -> Self {
        let mut x = (val >> 38) as i32;
        let mut y = (val & 0xFFF) as i32;
        let mut z = ((val >> 12) & 0x3FFFFFF) as i32;

        if x >= 2 << 25-1 { 
            x -= 2 << 26-1; 
        }

        if y >= 2 << 11-1 { 
            y -= 2 << 12-1; 
        }

        if z >= 2 << 25-1 { 
            z -= 2 << 26-1; 
        };


        Self { x, y, z }
    }

    pub fn to_u64(&self) -> u64 {
        let (x, y, z) = (self.x, self.y, self.z);

        let val = (((x & 0x3FFFFFF) as u64) << 38) as u64 | (((z & 0x3FFFFFF) << 12 as u64)) as u64 | (y & 0xFFF) as u64;

        val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn de_ser_position_positive() {
        let position = Position { x: 111560, y: 333, z: 47 };

        let ulong = position.to_u64();

        assert_eq!(position, Position::from_u64(ulong));
    }

    #[test]
    fn de_ser_position_negative() {
        // TODO: negative positions don't pass!!
        let position = Position { x: -1560, y: -333, z: -9696 };

        let ulong = position.to_u64();

        assert_eq!(position, Position::from_u64(ulong));
    }
}