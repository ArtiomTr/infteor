use std::{
    convert::TryInto,
    io::{self, Read},
    iter::repeat,
};

use crate::write::get_mask;

#[cfg(test)]
const BUFFER_SIZE: usize = 2;
#[cfg(not(test))]
const BUFFER_SIZE: usize = 256;

pub struct BitReader<R: Read> {
    reader: R,
    remainder: u64,
    remainder_length: usize,
}

impl<R: Read> BitReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            remainder: 0,
            remainder_length: 0,
        }
    }

    pub fn read(&mut self, count: usize) -> Result<u64, io::Error> {
        let mut pull_bits = count.saturating_sub(self.remainder_length);
        let remainder_bits = count.min(self.remainder_length);
        let mask = get_mask(remainder_bits as u32);
        let mut result = if remainder_bits == 0 {
            0
        } else {
            (self.remainder >> (u64::BITS as usize - remainder_bits)) & mask
        };
        if count == u64::BITS as usize {
            self.remainder = 0;
        } else {
            self.remainder <<= remainder_bits;
        }
        self.remainder_length -= remainder_bits;

        while pull_bits > 0 {
            let mut buffer = [0; 8];
            let len = self.reader.read(&mut buffer)?;
            if len == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "file end reached",
                ));
            }

            let word = u64::from_be_bytes(
                buffer[0..len]
                    .iter()
                    .copied()
                    .chain(repeat(0u8).take(8 - len))
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            );

            let bits = (len * 8).min(pull_bits);
            pull_bits -= bits;
            result = if bits == u64::BITS as usize {
                word
            } else {
                (result << bits) | ((word >> (u64::BITS as usize - bits)) & get_mask(bits as u32))
            };
            let word = word << bits;
            self.remainder = word;
            self.remainder_length = len * 8 - bits;
        }

        Ok(result)
    }
}

pub struct WordIter<R: Read> {
    reader: R,
    word_size: usize,
    words: Vec<u64>,
    remainder: u64,
    remainder_length: usize,
}

impl<R: Read> WordIter<R> {
    pub fn new(reader: R, word_size: u8) -> Self {
        Self {
            reader,
            word_size: word_size as usize,
            words: vec![],
            remainder: 0,
            remainder_length: 0,
        }
    }
}

impl<R: Read> Iterator for WordIter<R> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        while self.words.is_empty() {
            let mut buffer = [0u8; BUFFER_SIZE];
            let Ok(length) = self.reader.read(&mut buffer) else {
                return None;
            };

            if length == 0 {
                if self.remainder_length != 0 {
                    self.remainder_length = 0;
                    return Some(self.remainder);
                } else {
                    return None;
                }
            }

            let mut word = self.remainder;
            let mut word_length = self.remainder_length;
            let mut cursor = 0;

            self.remainder = 0;
            self.remainder_length = 0;

            while cursor / 8 < length {
                let mut left_bits = self.word_size - word_length;

                while left_bits > 0 && cursor / 8 < length {
                    let byte_index = cursor / 8;
                    let bits_to_take = (8 - cursor % 8).min(left_bits);
                    let mask = ((1u64 << bits_to_take) - 1) as u8;
                    word = (word << bits_to_take)
                        | ((buffer[byte_index] >> (8 - (cursor % 8) - bits_to_take)) & mask) as u64;
                    left_bits -= bits_to_take;
                    cursor += bits_to_take;
                    word_length += bits_to_take;
                }

                if left_bits > 0 {
                    self.remainder = word;
                    self.remainder_length = word_length;
                } else {
                    self.words.push(word);
                    word = 0;
                    word_length = 0;
                }
            }

            self.words.reverse();
        }

        return self.words.pop();
    }
}

pub trait ToWordIter {
    fn word_iter(self, word_size: u8) -> WordIter<impl Read>;
}

impl<R: Read> ToWordIter for R {
    fn word_iter(self, word_size: u8) -> WordIter<impl Read> {
        WordIter::new(self, word_size)
    }
}

#[cfg(test)]
mod test {
    use crate::read::WordIter;

    use super::BitReader;

    #[test]
    fn should_correctly_read_words_with_size_2() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let iter = WordIter::new(&buffer[..], 2);
        let words = iter.collect::<Vec<_>>();
        assert_eq!(
            words,
            vec![
                0b01, 0b11, 0b00, 0b11, 0b01, 0b10, 0b10, 0b01, 0b01, 0b11, 0b00, 0b10, 0b01, 0b11,
                0b00, 0b11, 0b01, 0b10, 0b01, 0b01, 0b00, 0b10, 0b00, 0b01
            ]
        );
    }

    #[test]
    fn should_correctly_read_words_with_size_3() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let iter = WordIter::new(&buffer[..], 3);
        let words = iter.collect::<Vec<_>>();
        assert_eq!(
            words,
            vec![
                0b011, 0b100, 0b110, 0b110, 0b100, 0b101, 0b110, 0b010, 0b011, 0b100, 0b110, 0b110,
                0b010, 0b100, 0b100, 0b001
            ]
        );
    }

    #[test]
    fn should_correctly_read_words_with_size_4() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let iter = WordIter::new(&buffer[..], 4);
        let words = iter.collect::<Vec<_>>();
        assert_eq!(
            words,
            vec![
                0b0111, 0b0011, 0b0110, 0b1001, 0b0111, 0b0010, 0b0111, 0b0011, 0b0110, 0b0101,
                0b0010, 0b0001
            ]
        );
    }

    #[test]
    fn should_correctly_read_words_with_size_5() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let iter = WordIter::new(&buffer[..], 5);
        let words = iter.collect::<Vec<_>>();
        assert_eq!(
            words,
            vec![
                0b01110, 0b01101, 0b10100, 0b10111, 0b00100, 0b11100, 0b11011, 0b00101, 0b00100,
                0b001
            ]
        );
    }

    #[test]
    fn should_correctly_read_words_with_size_6() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let iter = WordIter::new(&buffer[..], 6);
        let words = iter.collect::<Vec<_>>();
        assert_eq!(
            words,
            vec![0b011100, 0b110110, 0b100101, 0b110010, 0b011100, 0b110110, 0b010100, 0b100001]
        );
    }

    #[test]
    fn should_correctly_read_words_with_size_7() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let iter = WordIter::new(&buffer[..], 7);
        let words = iter.collect::<Vec<_>>();
        assert_eq!(
            words,
            vec![0b0111001, 0b1011010, 0b0101110, 0b0100111, 0b0011011, 0b0010100, 0b100001]
        );
    }

    #[test]
    fn should_correctly_read_words_with_size_8() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let iter = WordIter::new(&buffer[..], 8);
        let words = iter.collect::<Vec<_>>();
        assert_eq!(
            words,
            vec![0b01110011, 0b01101001, 0b01110010, 0b01110011, 0b01100101, 0b00100001]
        );
    }

    #[test]
    fn should_correctly_read_words_with_size_9() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let iter = WordIter::new(&buffer[..], 9);
        let words = iter.collect::<Vec<_>>();
        assert_eq!(
            words,
            vec![
                0b011100110,
                0b110100101,
                0b110010011,
                0b100110110,
                0b010100100,
                0b001
            ]
        );
    }

    #[test]
    fn should_correctly_read_words_with_size_12() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let iter = WordIter::new(&buffer[..], 12);
        let words = iter.collect::<Vec<_>>();
        assert_eq!(
            words,
            vec![
                0b011100110110,
                0b100101110010,
                0b011100110110,
                0b010100100001
            ]
        );
    }

    #[test]
    fn should_correctly_read_words_with_size_13() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let iter = WordIter::new(&buffer[..], 13);
        let words = iter.collect::<Vec<_>>();
        assert_eq!(
            words,
            vec![
                0b0111001101101,
                0b0010111001001,
                0b1100110110010,
                0b100100001
            ]
        );
    }

    #[test]
    fn should_correctly_read_words_with_size_16() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let iter = WordIter::new(&buffer[..], 16);
        let words = iter.collect::<Vec<_>>();
        assert_eq!(
            words,
            vec![0b0111001101101001, 0b0111001001110011, 0b0110010100100001]
        );
    }

    #[test]
    fn should_correctly_read_words_with_size_17() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let iter = WordIter::new(&buffer[..], 17);
        let words = iter.collect::<Vec<_>>();
        assert_eq!(
            words,
            vec![0b01110011011010010, 0b11100100111001101, 0b10010100100001]
        );
    }

    #[test]
    fn should_correctly_read_words_with_size_47() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let iter = WordIter::new(&buffer[..], 47);
        let words = iter.collect::<Vec<_>>();
        assert_eq!(
            words,
            vec![0b01110011011010010111001001110011011001010010000, 0b1]
        );
    }

    #[test]
    fn should_correctly_read_words_with_size_48() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let iter = WordIter::new(&buffer[..], 48);
        let words = iter.collect::<Vec<_>>();
        assert_eq!(
            words,
            vec![0b011100110110100101110010011100110110010100100001]
        );
    }

    #[test]
    fn should_correctly_read_words_with_size_63() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let iter = WordIter::new(&buffer[..], 63);
        let words = iter.collect::<Vec<_>>();
        assert_eq!(
            words,
            vec![0b011100110110100101110010011100110110010100100001]
        );
    }

    #[test]
    fn should_correctly_read_words_with_size_64() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let iter = WordIter::new(&buffer[..], 64);
        let words = iter.collect::<Vec<_>>();
        assert_eq!(
            words,
            vec![0b011100110110100101110010011100110110010100100001]
        );
    }

    #[test]
    fn should_correctly_read_1_bit() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let mut reader = BitReader::new(&buffer[..]);

        assert_eq!(reader.read(1).unwrap(), 0);
        assert_eq!(reader.read(1).unwrap(), 1);
        assert_eq!(reader.read(1).unwrap(), 1);
        assert_eq!(reader.read(1).unwrap(), 1);
        assert_eq!(reader.read(1).unwrap(), 0);
        assert_eq!(reader.read(1).unwrap(), 0);
        assert_eq!(reader.read(1).unwrap(), 1);
        assert_eq!(reader.read(1).unwrap(), 1);
        assert_eq!(reader.read(1).unwrap(), 0);
        assert_eq!(reader.read(1).unwrap(), 1);
        assert_eq!(reader.read(1).unwrap(), 1);
    }

    #[test]
    fn should_correctly_read_2_bits() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let mut reader = BitReader::new(&buffer[..]);

        assert_eq!(reader.read(2).unwrap(), 0b01);
        assert_eq!(reader.read(2).unwrap(), 0b11);
        assert_eq!(reader.read(2).unwrap(), 0b00);
        assert_eq!(reader.read(2).unwrap(), 0b11);
        assert_eq!(reader.read(2).unwrap(), 0b01);
        assert_eq!(reader.read(2).unwrap(), 0b10);
        assert_eq!(reader.read(2).unwrap(), 0b10);
        assert_eq!(reader.read(2).unwrap(), 0b01);
    }

    #[test]
    fn should_correctly_read_3_bits() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let mut reader = BitReader::new(&buffer[..]);

        assert_eq!(reader.read(3).unwrap(), 0b011);
        assert_eq!(reader.read(3).unwrap(), 0b100);
        assert_eq!(reader.read(3).unwrap(), 0b110);
        assert_eq!(reader.read(3).unwrap(), 0b110);
        assert_eq!(reader.read(3).unwrap(), 0b100);
        assert_eq!(reader.read(3).unwrap(), 0b101);
        assert_eq!(reader.read(3).unwrap(), 0b110);
        assert_eq!(reader.read(3).unwrap(), 0b010);
    }

    #[test]
    fn should_correctly_read_5_bits() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let mut reader = BitReader::new(&buffer[..]);

        assert_eq!(reader.read(5).unwrap(), 0b01110);
        assert_eq!(reader.read(5).unwrap(), 0b01101);
        assert_eq!(reader.read(5).unwrap(), 0b10100);
        assert_eq!(reader.read(5).unwrap(), 0b10111);
        assert_eq!(reader.read(5).unwrap(), 0b00100);
        assert_eq!(reader.read(5).unwrap(), 0b11100);
        assert_eq!(reader.read(5).unwrap(), 0b11011);
        assert_eq!(reader.read(5).unwrap(), 0b00101);
        assert_eq!(reader.read(5).unwrap(), 0b00100);
        assert_eq!(reader.read(3).unwrap(), 0b001);
    }

    #[test]
    fn should_correctly_read_7_bits() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let mut reader = BitReader::new(&buffer[..]);

        assert_eq!(reader.read(7).unwrap(), 0b0111001);
        assert_eq!(reader.read(7).unwrap(), 0b1011010);
        assert_eq!(reader.read(7).unwrap(), 0b0101110);
        assert_eq!(reader.read(7).unwrap(), 0b0100111);
        assert_eq!(reader.read(7).unwrap(), 0b0011011);
        assert_eq!(reader.read(7).unwrap(), 0b0010100);
        assert_eq!(reader.read(6).unwrap(), 0b100001);
    }

    #[test]
    fn should_correctly_read_8_bits() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let mut reader = BitReader::new(&buffer[..]);

        assert_eq!(reader.read(8).unwrap(), 0b01110011);
        assert_eq!(reader.read(8).unwrap(), 0b01101001);
        assert_eq!(reader.read(8).unwrap(), 0b01110010);
        assert_eq!(reader.read(8).unwrap(), 0b01110011);
        assert_eq!(reader.read(8).unwrap(), 0b01100101);
        assert_eq!(reader.read(8).unwrap(), 0b00100001);
    }

    #[test]
    fn should_correctly_read_9_bits() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let mut reader = BitReader::new(&buffer[..]);

        assert_eq!(reader.read(9).unwrap(), 0b011100110);
        assert_eq!(reader.read(9).unwrap(), 0b110100101);
        assert_eq!(reader.read(9).unwrap(), 0b110010011);
        assert_eq!(reader.read(9).unwrap(), 0b100110110);
        assert_eq!(reader.read(9).unwrap(), 0b010100100);
        assert_eq!(reader.read(3).unwrap(), 0b001);
    }

    #[test]
    fn should_correctly_read_13_bits() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let mut reader = BitReader::new(&buffer[..]);

        assert_eq!(reader.read(13).unwrap(), 0b0111001101101);
        assert_eq!(reader.read(13).unwrap(), 0b0010111001001);
        assert_eq!(reader.read(13).unwrap(), 0b1100110110010);
        assert_eq!(reader.read(9).unwrap(), 0b100100001);
    }

    #[test]
    fn should_correctly_read_17_bits() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let mut reader = BitReader::new(&buffer[..]);

        assert_eq!(reader.read(17).unwrap(), 0b01110011011010010);
        assert_eq!(reader.read(17).unwrap(), 0b11100100111001101);
        assert_eq!(reader.read(14).unwrap(), 0b10010100100001);
    }

    #[test]
    fn should_correctly_read_23_bits() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let mut reader = BitReader::new(&buffer[..]);

        assert_eq!(reader.read(23).unwrap(), 0b01110011011010010111001);
        assert_eq!(reader.read(23).unwrap(), 0b00111001101100101001000);
        assert_eq!(reader.read(2).unwrap(), 0b01);
    }

    #[test]
    fn should_correctly_read_25_bits() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let mut reader = BitReader::new(&buffer[..]);

        assert_eq!(reader.read(25).unwrap(), 0b0111001101101001011100100);
        assert_eq!(reader.read(23).unwrap(), 0b11100110110010100100001);
    }

    #[test]
    fn should_correctly_read_47_bits() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let mut reader = BitReader::new(&buffer[..]);

        assert_eq!(
            reader.read(47).unwrap(),
            0b01110011011010010111001001110011011001010010000
        );
        assert_eq!(reader.read(1).unwrap(), 0b1);
    }

    #[test]
    fn should_correctly_read_48_bits() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];
        let mut reader = BitReader::new(&buffer[..]);

        assert_eq!(
            reader.read(48).unwrap(),
            0b011100110110100101110010011100110110010100100001
        );
    }
}
