/// 比特流，处理一次，向前流动，不能回退。
pub struct BitStream{
    bs: bytes::Bytes,
    cursor: usize,    // 要读取的要读取的字节所在索引 
    bits_left: u8, // 当前读取的字节剩余位数。
}

impl BitStream {
    pub fn new(bs: bytes::Bytes) -> Self{
        BitStream{
            bs,
            cursor: 0,
            bits_left: 8,
        }
    }

    pub fn read_byte(&self) -> u8{
        *self.bs.get(self.cursor).unwrap()
    }

    pub fn read_u1(&mut self) -> u8{
        self.bits_left -= 1;
        let res = (self.read_byte() >> self.bits_left) & 0x01;

        // println!("byte: {:#x}", self.read_byte());
        if self.bits_left == 0 {
            self.cursor += 1;
            self.bits_left = 8;
        }
        res
    } 

    pub fn read_u(&mut self, n: u8) -> usize{
        let mut res: usize = 0;
        for _ in 0..n{
            res <<= 1;
            let cur_bit = self.read_u1() as usize;
            res |= cur_bit;
        }
        res
    }

    pub fn read_ue(&mut self) -> u32 {
        let mut zeros: u8 = 0;
        while self.read_u1() == 0 {
            zeros += 1;
        }

        debug_assert!(zeros <= 32);
        let res: u32 = (1 << zeros | self.read_u(zeros) as u32) - 1;
        res
    }

    /// se = (-1)^(k + 1)*Ceil(K / 2)
    pub fn read_se(&mut self) -> i32 {
        let ue: u32 = self.read_ue();
        println!("ue: {}", ue);
        let negative = if ue & 0x1 == 1 {1} else {-1};
        let res = (ue as i64 + 1 >> 1) as i32 * negative;
        res    
    }
}

