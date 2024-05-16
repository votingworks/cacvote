# `tlv-derive`

# Example

```rust
use tlv::{Decode, Encode};
use tlv_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Decode, Encode)]
struct Test {
    #[tlv(tag = 0x01)]
    a: u8,
    #[tlv(tag = 0x02)]
    b: u16,
}

fn main() -> std::io::Result<()> {
    let value = Test { a: 0x99, b: 0xabcd };
    let encoded = tlv::to_vec(&value)?;
    assert_eq!(encoded, [
        0x01, // `a` tag
        0x01, // `a` length
        0x99, // `a` value
        0x02, // `b` tag
        0x02, // `b` length
        0xab, // `b` value
        0xcd  // `b` value
    ]);
    let decoded: Test = tlv::from_slice(&encoded)?;
    assert_eq!(decoded, value);
    Ok(())
}
```
