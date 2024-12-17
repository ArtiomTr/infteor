use std::io::{BufReader, BufWriter, Read, Write};

use anyhow::{bail, Result};
use bit_utils::{elias, write::WordWriter, zigzag};

use crate::dictionary::Dictionary;

pub fn encode(reader: impl Read, writer: impl Write, strategy: i64) -> Result<()> {
    let mut reader = BufReader::new(reader);
    let mut writer = WordWriter::new(BufWriter::new(writer));
    elias::write_gamma_elias(&mut writer, zigzag::encode(strategy))?;

    let mut buf = vec![0u8];
    let mut dictionary = Dictionary::new(strategy.into());

    let mut word_buf = Vec::new();
    loop {
        let len = reader.read(&mut buf)?;

        if len == 0 {
            break;
        }

        word_buf.push(buf[0]);

        if let Some(w) = dictionary.add(&word_buf) {
            elias::write_gamma_elias(&mut writer, w.0 as u64)?;
            writer.write((w.1 as u64, u8::BITS as usize))?;
            word_buf.clear();
        }
    }

    if word_buf.len() == 0 {
        elias::write_gamma_elias(&mut writer, dictionary.len() as u64 + 1)?;
    } else {
        elias::write_gamma_elias(&mut writer, dictionary.len() as u64 + 2)?;
        let Some(index) = dictionary.find(&word_buf) else {
            bail!("Something wrong happen");
        };
        elias::write_gamma_elias(&mut writer, index as u64)?;
    }

    Ok(())
}

