use std::io::{self, Read, Write};

use crate::{read::BitReader, write::WordWriter};

pub fn write_gamma_elias(writer: &mut WordWriter<impl Write>, word: u64) -> io::Result<()> {
    let length = (u64::BITS - (word + 1).leading_zeros() - 1) as usize;
    writer.write((0u64, length))?;
    writer.write((word + 1, length + 1))?;

    Ok(())
}

pub fn read_gamma_elias(reader: &mut BitReader<impl Read>) -> io::Result<u64> {
    let mut last_bit: u64 = reader.read(1)?;
    let mut word_len = 0usize;
    while last_bit == 0 {
        word_len += 1;
        last_bit = reader.read(1)?;
    }

    if word_len == 0 {
        Ok(0)
    } else {
        let word = reader.read(word_len)?;
        let word = (1 << word_len) | word;
        Ok(word - 1)
    }
}

#[cfg(test)]
mod test {
    use crate::{elias::read_gamma_elias, read::BitReader, write::WordWriter};

    use super::write_gamma_elias;

    #[test]
    fn test_write_0() {
        let mut buffer = Vec::new();
        let mut writer = WordWriter::new(&mut buffer);

        write_gamma_elias(&mut writer, 0).unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0b10000000]);
    }

    #[test]
    fn test_write_1() {
        let mut buffer = Vec::new();
        let mut writer = WordWriter::new(&mut buffer);

        write_gamma_elias(&mut writer, 1).unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0b01000000]);
    }

    #[test]
    fn test_write_2() {
        let mut buffer = Vec::new();
        let mut writer = WordWriter::new(&mut buffer);

        write_gamma_elias(&mut writer, 2).unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0b01100000]);    
    }

    #[test]
    fn test_write_3() {
        let mut buffer = Vec::new();
        let mut writer = WordWriter::new(&mut buffer);

        write_gamma_elias(&mut writer, 3).unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0b00100000]);    
    }

    #[test]
    fn test_write_4() {
        let mut buffer = Vec::new();
        let mut writer = WordWriter::new(&mut buffer);

        write_gamma_elias(&mut writer, 4).unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0b00101000]);    
    }

    #[test]
    fn test_write_5() {
        let mut buffer = Vec::new();
        let mut writer = WordWriter::new(&mut buffer);

        write_gamma_elias(&mut writer, 5).unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0b00110000]);    
    }

    #[test]
    fn test_write_6() {
        let mut buffer = Vec::new();
        let mut writer = WordWriter::new(&mut buffer);

        write_gamma_elias(&mut writer, 6).unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0b00111000]);
    }

    #[test]
    fn test_write_14() {
        let mut buffer = Vec::new();
        let mut writer = WordWriter::new(&mut buffer);

        write_gamma_elias(&mut writer, 14).unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0b00011110]);    
    }

    #[test]
    fn test_write_16() {
        let mut buffer = Vec::new();
        let mut writer = WordWriter::new(&mut buffer);

        write_gamma_elias(&mut writer, 16).unwrap();
        drop(writer);
        assert_eq!(buffer, vec![0b00001000, 0b10000000]);    
    }

    #[test]
    fn test_read_0() {
        let buffer = vec![0b10000000];
        let mut reader = BitReader::new(&buffer[..]);
        let result = read_gamma_elias(&mut reader).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_read_1() {
        let buffer = vec![0b01000000];
        let mut reader = BitReader::new(&buffer[..]);
        let result = read_gamma_elias(&mut reader).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_read_2() {
        let buffer = vec![0b01100000];
        let mut reader = BitReader::new(&buffer[..]);

        let result = read_gamma_elias(&mut reader).unwrap();
        assert_eq!(result, 2);    
    }

    #[test]
    fn test_read_3() {
        let buffer = vec![0b00100000];
        let mut reader = BitReader::new(&buffer[..]);

        let result = read_gamma_elias(&mut reader).unwrap();
        assert_eq!(result, 3);    
    }

    #[test]
    fn test_read_4() {
        let buffer = vec![0b00101000];
        let mut reader = BitReader::new(&buffer[..]);

        let result = read_gamma_elias(&mut reader).unwrap();
        assert_eq!(result, 4);    
    }

    #[test]
    fn test_read_5() {
        let buffer = vec![0b00110000];
        let mut reader = BitReader::new(&buffer[..]);

        let result = read_gamma_elias(&mut reader).unwrap();
        assert_eq!(result, 5);    
    }

    #[test]
    fn test_read_6() {
        let buffer = vec![0b00111000];
        let mut reader = BitReader::new(&buffer[..]);

        let result = read_gamma_elias(&mut reader).unwrap();
        assert_eq!(result, 6);
    }

    #[test]
    fn test_read_14() {
        let buffer = vec![0b00011110];
        let mut reader = BitReader::new(&buffer[..]);

        let result = read_gamma_elias(&mut reader).unwrap();
        assert_eq!(result, 14);    
    }

    #[test]
    fn test_read_16() {
        let buffer = vec![0b00001000, 0b10000000];
        let mut reader = BitReader::new(&buffer[..]);

        let result = read_gamma_elias(&mut reader).unwrap();
        assert_eq!(result, 16);    
    }
}