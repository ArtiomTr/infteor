use anyhow::anyhow;
use std::io::{self, Read, Seek, SeekFrom};

pub struct SeekableSliceReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> SeekableSliceReader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }
}

impl<'a> Read for SeekableSliceReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let count = buf.len().min(self.buf.len() - self.pos);

        buf[0..count].copy_from_slice(&self.buf[self.pos..(self.pos + count)]);

        self.pos += count;

        Ok(count)
    }
}

impl<'a> Seek for SeekableSliceReader<'a> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(pos) => {
                self.pos = pos as usize;
                Ok(pos)
            }
            SeekFrom::End(pos) => {
                if pos < 0 {
                    let (new_pos, is_overflow) = self.buf.len().overflowing_sub(pos.abs() as usize);

                    if is_overflow {
                        return Err(io::Error::new(io::ErrorKind::NotSeekable, anyhow!("Error")));
                    }

                    self.pos = new_pos;

                    Ok(new_pos as u64)
                } else {
                    self.pos = self.buf.len().saturating_add(pos as usize);

                    Ok(self.pos as u64)
                }
            }
            SeekFrom::Current(pos) => {
                if pos < 0 {
                    let (new_pos, is_overflow) = self.pos.overflowing_sub(pos.abs() as usize);

                    if is_overflow {
                        return Err(io::Error::new(io::ErrorKind::NotSeekable, anyhow!("Error")));
                    }

                    self.pos = new_pos;

                    Ok(new_pos as u64)
                } else {
                    self.pos = self.pos.saturating_add(pos as usize);

                    Ok(self.pos as u64)
                }
            }
        }
    }
}
