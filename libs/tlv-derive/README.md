# `tlv-derive`

This crate provides derive macros for the `tlv` crate.

# Example

```rust
use tlv::{Decode, Encode};
use tlv_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Decode, Encode)]
struct Test {
    #[tlv(tag = 0x20)]
    a: u8,
    #[tlv(tag = 0x21)]
    b: u16,
}

fn main() -> std::io::Result<()> {
    let value = Test { a: 0x99, b: 0xabcd };
    let encoded = tlv::to_vec(&value)?;
    assert_eq!(encoded, [
        0x20, // `value.a` tag
        0x01, // `value.a` length
        0x99, // `value.a` value
        0x21, // `value.b` tag
        0x02, // `value.b` length
        0xab, // `value.b` value
        0xcd  // `value.b` value
    ]);
    let decoded: Test = tlv::from_slice(&encoded)?;
    assert_eq!(decoded, value);
    Ok(())
}
```
