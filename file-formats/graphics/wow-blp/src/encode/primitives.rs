pub fn push_le_u32(value: u32, output: &mut Vec<u8>) {
    let mut buff: [u8; 4] = Default::default();
    buff[0] = (value & 0xFF) as u8;
    buff[1] = ((value >> 8) & 0xFF) as u8;
    buff[2] = ((value >> 16) & 0xFF) as u8;
    buff[3] = ((value >> 24) & 0xFF) as u8;
    output.extend(buff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_le_u32() {
        let mut output = vec![];
        push_le_u32(0x1A2B3C4D, &mut output);
        assert_eq!(output, [0x4D, 0x3c, 0x2B, 0x1A]);
    }
}
