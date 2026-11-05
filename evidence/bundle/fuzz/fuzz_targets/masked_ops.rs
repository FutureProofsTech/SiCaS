#![no_main]

use libfuzzer_sys::fuzz_target;
use sicas_core::Masked;

fn read_u32(data: &[u8], offset: usize) -> u32 {
    let mut bytes = [0_u8; 4];
    bytes.copy_from_slice(&data[offset..offset + 4]);
    u32::from_le_bytes(bytes)
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 20 {
        return;
    }

    let value = read_u32(data, 0);
    let mask = read_u32(data, 4);
    let other = read_u32(data, 8);
    let other_mask = read_u32(data, 12);
    let fresh_mask = read_u32(data, 16);
    let amount = read_u32(data, 16) % 32;

    let masked = Masked::<u32>::mask(value, mask);
    let masked_other = Masked::<u32>::mask(other, other_mask);

    assert_eq!(*masked.unmask().expose_secret(), value);
    assert_eq!(
        *masked.xor(&masked_other).unmask().expose_secret(),
        value ^ other
    );
    assert_eq!(*masked.complement().unmask().expose_secret(), !value);
    assert_eq!(
        *masked.and(&masked_other, fresh_mask).unmask().expose_secret(),
        value & other
    );
    assert_eq!(
        *masked.rotate_left(amount).unmask().expose_secret(),
        value.rotate_left(amount)
    );
    assert_eq!(
        *masked.rotate_right(amount).unmask().expose_secret(),
        value.rotate_right(amount)
    );
    assert_eq!(
        *masked.shift_left(amount).unmask().expose_secret(),
        value.checked_shl(amount).unwrap_or(0)
    );
    assert_eq!(
        *masked.shift_right(amount).unmask().expose_secret(),
        value.checked_shr(amount).unwrap_or(0)
    );
    assert_eq!(
        masked.ct_eq(&masked_other).as_u8(),
        u8::from(value == other)
    );
});
