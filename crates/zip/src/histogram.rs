use crate::read::ToWordIter;
use anyhow::Result;
use indicatif::ProgressBar;
use std::{convert::TryFrom, fmt::Display, io::Read};

#[derive(Debug, Clone)]
pub struct Histogram {
    freq: Vec<u64>,
    word_size: u8,
}

const STEP: u64 = 600000;

impl Histogram {
    pub fn read(reader: &mut impl Read, word_size: u8, bar: Option<&ProgressBar>) -> Result<Self> {
        let mut histogram = vec![0u64; 1usize << word_size];

        let mut step = 0;
        for word in reader.word_iter(word_size) {
            histogram[word as usize] += 1;
            step += 1;
            if step >= STEP {
                bar.inspect(|v| v.inc(word_size as u64 * STEP));
                step = 0;
            }
        }

        Ok(Histogram {
            freq: histogram,
            word_size,
        })
    }

    pub fn get_freq(&self) -> &[u64] {
        &self.freq
    }

    pub fn get_word_size(&self) -> u8 {
        self.word_size
    }
}

impl<'a> TryFrom<&'a [u64]> for Histogram {
    type Error = anyhow::Error;

    fn try_from(value: &'a [u64]) -> std::result::Result<Self, Self::Error> {
        if !value.len().is_power_of_two() {
            anyhow::bail!("Slice is not a valid histogram");
        }

        let word_size = value.len().trailing_zeros();

        return Ok(Self {
            freq: value.to_vec(),
            word_size: word_size as u8,
        });
    }
}

impl TryFrom<Vec<u64>> for Histogram {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u64>) -> std::result::Result<Self, Self::Error> {
        if !value.len().is_power_of_two() {
            anyhow::bail!("Slice is not a valid histogram");
        }

        let word_size = value.len().trailing_zeros();

        return Ok(Self {
            freq: value,
            word_size: word_size as u8,
        });
    }
}

#[cfg(test)]
mod test {
    use crate::histogram::Histogram;

    #[test]
    fn should_correctly_collect_histogram() {
        let buffer = [
            0b01110011u8,
            0b01101001u8,
            0b01110010u8,
            0b01110011u8,
            0b01100101u8,
            0b00100001u8,
        ];

        let mut reader = &buffer[..];

        let histogram = Histogram::read(&mut reader, 2, None).unwrap();
        assert_eq!(histogram.freq, vec![5, 9, 5, 5]);
    }
}
