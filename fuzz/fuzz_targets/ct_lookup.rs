#![no_main]

use libfuzzer_sys::fuzz_target;
use sicas_core::ct;

fuzz_target!(|data: &[u8]| {
    if data.len() < 257 {
        return;
    }

    let mut table = [0_u8; 256];
    table.copy_from_slice(&data[..256]);
    let index = data[256];

    assert_eq!(ct::lookup_u8(&table, index), table[usize::from(index)]);
});
