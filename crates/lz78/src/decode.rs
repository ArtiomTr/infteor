use std::io::{BufReader, BufWriter, Read, Write};

use anyhow::{bail, Result};
use bit_utils::{elias, read::BitReader, zigzag};

use crate::dictionary::Dictionary;

pub fn decode(reader: impl Read, writer: impl Write) -> Result<()> {
    let mut reader = BitReader::new(BufReader::new(reader));
    let mut writer = BufWriter::new(writer);

    let strategy = elias::read_gamma_elias(&mut reader)?;
    let strategy = zigzag::decode(strategy);

    let mut dictionary = Dictionary::new(strategy.into());

    loop {
        let index = elias::read_gamma_elias(&mut reader)?;

        if index == dictionary.len() as u64 + 1 {
            break;
        }

        if index == dictionary.len() as u64 + 2 {
            let index = elias::read_gamma_elias(&mut reader)?;
            let Some(sentence) = dictionary.get(index as usize) else {
                bail!("Not valid LZ78 encoded file");
            };
            writer.write(&sentence)?;
            break;
        }
        
        let word = reader.read(8)? as u8;
        
        if index == 0 {
            writer.write(&[word])?;
            dictionary.add(&[word]).unwrap();
        } else {
            let Some(mut sentence) = dictionary.get(index as usize) else {
                bail!("Not valid LZ78 encoded file");
            };

            sentence.push(word);
            writer.write(&sentence)?;
            dictionary.add(&sentence).unwrap();
        }
    }

    Ok(())
}

