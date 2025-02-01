pub mod bitstream;
pub mod parse;
 

#[cfg(test)]
mod tests {
    use super::bitstream::BitStream;

    #[test]
    fn it_works() {
        let bytes = bytes::Bytes::from_static(&[0x10, 0x42, 0x30, 0xd0]);
        let mut bs = BitStream::new(bytes);
        assert_eq!(bs.read_byte(), 0x10);
        assert_eq!(bs.read_u1(), 0);
        assert_eq!(bs.read_u1(), 0);
        assert_eq!(bs.read_u1(), 0);
        assert_eq!(bs.read_u1(), 1);
        assert_eq!(bs.read_u1(), 0);
        assert_eq!(bs.read_u1(), 0);
        assert_eq!(bs.read_u1(), 0);
        assert_eq!(bs.read_u1(), 0);
        
        assert_eq!(bs.read_byte(), 0x42);
        assert_eq!(bs.read_u1(), 0);
        assert_eq!(bs.read_u1(), 1);
        assert_eq!(bs.read_u1(), 0);
        assert_eq!(bs.read_u1(), 0);
        assert_eq!(bs.read_u1(), 0);
        assert_eq!(bs.read_u1(), 0);
        assert_eq!(bs.read_u1(), 1);
        assert_eq!(bs.read_u1(), 0);

        assert_eq!(bs.read_byte(), 0x30);
        assert_eq!(bs.read_ue(), 0x5);

        assert_eq!(bs.read_se(), -5);
    }
}
