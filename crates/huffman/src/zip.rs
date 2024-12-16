use anyhow::Result;
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use bit_utils::{
    write::WordWriter,
    read::{BitReader, ToWordIter}
};

use crate::{
    histogram::Histogram,
    tree::HuffmanTree,
};

pub fn compress(word_size: u8, input: impl Read + Seek, output: impl Write) -> Result<()> {
    let mut reader = BufReader::new(input);
    let file_size = reader.seek(std::io::SeekFrom::End(0))?;
    reader.rewind()?;
    let histogram = Histogram::read(&mut reader, word_size, None)?;
    let tree = HuffmanTree::from(histogram);
    reader.rewind()?;

    let mut writer = BufWriter::new(output);
    writer.write(&file_size.to_be_bytes())?;
    let mut word_writer = WordWriter::new(writer);
    tree.write(&mut word_writer)?;

    for word in reader.word_iter(word_size) {
        word_writer.write(tree.encode_word(word))?;
    }

    Ok(())
}

pub fn decompress(input: impl Read, output: impl Write) -> Result<()> {
    let mut reader = BufReader::new(input);
    let mut file_size = [0u8; 8];
    reader.read_exact(&mut file_size)?;
    let file_size = u64::from_be_bytes(file_size);
    let mut reader = BitReader::new(reader);
    let tree = HuffmanTree::read(&mut reader)?;

    let writer = BufWriter::new(output);
    let mut writer = WordWriter::new(writer);
    let mut cursor = file_size * 8;

    while cursor > 0 {
        let word = tree.decode_next_word(&mut reader)?;
        writer.write((word, (tree.get_word_size() as usize).min(cursor as usize)))?;
        cursor = cursor.saturating_sub(tree.get_word_size() as u64);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::utils::SeekableSliceReader;

    use super::{compress, decompress};

    #[test]
    fn roundtrip_1() {
        let mut compressed = Vec::new();
        compress(10, SeekableSliceReader::new(&[10]), &mut compressed).unwrap();
        let mut output = Vec::new();
        decompress(&compressed[..], &mut output).unwrap();

        assert_eq!(output, vec![10]);
    }

    #[test]
    fn roundtrip_2() {
        let mut compressed = Vec::new();
        compress(10, SeekableSliceReader::new(&[10, 10]), &mut compressed).unwrap();
        let mut output = Vec::new();
        decompress(&compressed[..], &mut output).unwrap();

        assert_eq!(output, vec![10, 10]);
    }
}
