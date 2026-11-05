#![no_main]

use std::io::Cursor;

use libfuzzer_sys::fuzz_target;
use sicas_harness::trace::TraceSet;

fuzz_target!(|data: &[u8]| {
    let _ = TraceSet::read_from(Cursor::new(data));
});
