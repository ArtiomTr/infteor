#![no_main]

use std::u8;

use libfuzzer_sys::fuzz_target;
use huffman::{compress, decompress, utils::SeekableSliceReader};

fuzz_target!(|data: &[u8]| {
    if data.len() > 0 {
        let word_size = (data[0] % 18).clamp(2, 17);
        let mut compressed_output = Vec::new();

        compress(
            word_size,
            SeekableSliceReader::new(data),
            &mut compressed_output,
        )
        .unwrap();

        let mut received_output = Vec::new();

        decompress(
            SeekableSliceReader::new(&compressed_output),
            &mut received_output,
        )
        .unwrap();

        assert_eq!(data, received_output);
    };
});
