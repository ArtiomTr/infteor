use std::io::{self, Write};

pub const fn get_mask(count: u32) -> u64 {
    if count > u64::BITS {
        panic!("unable to make larger");
    }

    if count == u64::BITS {
        u64::MAX
    } else {
        (1 << count) - 1
    }
}

pub struct WordWriter<W: Write> {
    writer: W,
    buffer: u64,
    buffer_length: usize,
}

trait SaturatingShl {
    fn saturating_shl(self, other: usize) -> Self;
}

impl SaturatingShl for u64 {
    fn saturating_shl(self, other: usize) -> Self {
        if other >= Self::BITS as usize {
            0
        } else {
            self << other
        }
    }
}

impl<W: Write> WordWriter<W> {
    pub fn new(writer: W) -> Self {
        WordWriter {
            writer,
            buffer: 0,
            buffer_length: 0,
        }
    }

    pub fn write(&mut self, word: (u64, usize)) -> Result<(), io::Error> {
        let (mut word, mut size) = word;

        let total_len = self.buffer_length + size;

        if total_len / 8 >= 1 {
            let bytes_to_write = total_len.min(u64::BITS as usize) / 8;
            let bits_to_copy = bytes_to_write * 8 - self.buffer_length;
            let to_write = self.buffer.saturating_shl(bits_to_copy)
                | (word >> (size - bits_to_copy)) & get_mask(bits_to_copy as u32);
            self.writer.write_all(
                &to_write
                    .to_be_bytes()
                    .into_iter()
                    .skip(u64::BITS as usize / 8 - bytes_to_write)
                    .collect::<Vec<_>>(),
            )?;
            word &= get_mask(size as u32 - bits_to_copy as u32);
            size -= bits_to_copy;
            self.buffer = 0;
            self.buffer_length = 0;
        }

        if size > 0 {
            self.buffer = (self.buffer << size) | (word & get_mask(size as u32));
            self.buffer_length += size;
        }

        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), io::Error> {
        if self.buffer_length == 0 {
            return self.writer.flush();
        }

        let byte_count = (self.buffer_length + 7) / 8;
        let leftover = byte_count * 8 - self.buffer_length;
        self.buffer <<= leftover;
        self.writer.write_all(
            &self
                .buffer
                .to_be_bytes()
                .into_iter()
                .skip(u64::BITS as usize / 8 - byte_count)
                .collect::<Vec<_>>(),
        )?;

        self.writer.flush()
    }
}

impl<W: Write> Drop for WordWriter<W> {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

impl<W: Write> From<W> for WordWriter<W> {
    fn from(value: W) -> Self {
        WordWriter::new(value)
    }
}

#[cfg(test)]
mod test {
    use crate::write::WordWriter;

    #[test]
    fn should_correctly_write_1_bit() {
        let mut buffer = Vec::new();

        let mut writer = WordWriter::new(&mut buffer);
        writer.write((1, 1)).unwrap();
        writer.write((0, 1)).unwrap();
        writer.write((1, 1)).unwrap();
        drop(writer);

        assert_eq!(buffer, vec![0b10100000]);
    }

    #[test]
    fn should_correctly_write_byte() {
        let mut buffer = Vec::new();

        let mut writer = WordWriter::new(&mut buffer);
        writer.write((1, 1)).unwrap();
        writer.write((0, 1)).unwrap();
        writer.write((1, 1)).unwrap();
        writer.write((1, 1)).unwrap();
        writer.write((1, 1)).unwrap();
        writer.write((1, 1)).unwrap();
        writer.write((1, 1)).unwrap();
        writer.write((1, 1)).unwrap();
        drop(writer);

        assert_eq!(buffer, vec![0b10111111]);
    }

    #[test]
    fn should_correctly_write_multiple_bytes() {
        let mut buffer = Vec::new();

        let mut writer = WordWriter::new(&mut buffer);
        writer.write((1, 1)).unwrap();
        writer.write((0, 1)).unwrap();
        writer.write((1, 1)).unwrap();
        writer.write((1, 1)).unwrap();
        writer.write((1, 1)).unwrap();
        writer.write((1, 1)).unwrap();
        writer.write((1, 1)).unwrap();
        writer.write((1, 1)).unwrap();
        writer.write((1, 1)).unwrap();
        drop(writer);

        assert_eq!(buffer, vec![0b10111111, 0b10000000]);
    }

    #[test]
    fn should_correctly_write_multiple_bytes_via_batch() {
        let mut buffer = Vec::new();

        let mut writer = WordWriter::new(&mut buffer);
        writer.write((1, 8)).unwrap();
        writer.write((1, 16)).unwrap();
        writer.write((1, 32)).unwrap();
        writer.write((1, 64)).unwrap();
        drop(writer);

        assert_eq!(buffer, vec![1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1]);
    }

    #[test]
    fn should_correctly_write_multiple_bytes_with_buffer() {
        let mut buffer = Vec::new();

        let mut writer = WordWriter::new(&mut buffer);
        writer.write((1, 1)).unwrap();
        writer.write((1, 64)).unwrap();
        drop(writer);

        assert_eq!(buffer, vec![0b10000000, 0, 0, 0, 0, 0, 0, 0, 0b10000000]);
    }

    #[test]
    fn should_correctly_write_multiple_bytes_with_buffer_2() {
        let mut buffer = Vec::new();

        let mut writer = WordWriter::new(&mut buffer);
        writer.write((1, 7)).unwrap();
        writer.write((1, 64)).unwrap();
        drop(writer);

        assert_eq!(buffer, vec![0b00000010, 0, 0, 0, 0, 0, 0, 0, 0b00000010]);
    }
}
