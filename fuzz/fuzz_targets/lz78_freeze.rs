#![no_main]

use libfuzzer_sys::fuzz_target;
use lz78::{encode, decode};

fuzz_target!(|data: &[u8]| {
    if data.len() > 0 {
        let mut compressed_output = Vec::new();

        encode(data, &mut compressed_output, -(data[0] as i64 + 1)).unwrap();

        let mut received_output = Vec::new();

        decode(&compressed_output[..], &mut received_output).unwrap();

        assert_eq!(data, received_output);
    };
});
