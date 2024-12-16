#![no_main]

use libfuzzer_sys::fuzz_target;
use bit_utils::{read::BitReader, write::WordWriter};

fuzz_target!(|data: &[u8]| {
    let mut output = Vec::new();
    let mut reader = BitReader::new(data);
    let mut writer = WordWriter::new(&mut output);

    let data_len = data.len() * 8;

    let mut cursor = 0;
    let mut to_read = 3;

    while cursor < data_len {
        if cursor + to_read >= data_len {
            to_read = data_len - cursor;
        }

        let word = reader.read(to_read).unwrap();
        writer.write((word, to_read)).unwrap();
        cursor += to_read;

        to_read = (word as usize % 18).clamp(2, 17);
    }

    drop(writer);

    assert_eq!(data, output);
});
