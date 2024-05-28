use proptest::proptest;
use tlv::Encode;
use tlv_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Test {
    #[tlv(tag = 0x99)]
    a: u8,
    #[tlv(tag = 0x00)]
    b: u16,
    #[tlv(tag = 0x01)]
    c: u32,
    #[tlv(tag = 0x02)]
    d: u64,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Nesting {
    #[tlv(tag = 0x99)]
    a: u8,
    #[tlv(tag = 0x00)]
    b: Test,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct StringContainer {
    #[tlv(tag = 0x00)]
    s: String,
}

#[test]
fn test_derive() {
    let value = Test {
        a: 1,
        b: 2,
        c: 3,
        d: 4,
    };
    let encoded = tlv::to_vec(&value).unwrap();
    assert_eq!(
        value.encoded_length().unwrap().value(),
        encoded.len() as u16
    );

    assert_eq!(
        encoded,
        [
            /* a TAG = */ 0x99, /* a LENGTH = */ 0x01, /* a VALUE = */ 0x01,
            /* b TAG = */ 0x00, /* b LENGTH = */ 0x02, /* b VALUE = */ 0x00, 0x02,
            /* c TAG = */ 0x01, /* c LENGTH = */ 0x04, /* c VALUE = */ 0x00, 0x00,
            0x00, 0x03, /* d TAG = */ 0x02, /* d LENGTH = */ 0x08, /* d VALUE = */
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04,
        ]
    );

    let decoded_value: Test = tlv::from_slice(&encoded).unwrap();
    assert_eq!(decoded_value, value);
}

#[test]
fn test_nesting() {
    let value = Nesting {
        a: 1,
        b: Test {
            a: 2,
            b: 3,
            c: 4,
            d: 5,
        },
    };
    let encoded = tlv::to_vec(&value).unwrap();
    assert_eq!(
        value.encoded_length().unwrap().value(),
        encoded.len() as u16
    );

    assert_eq!(
        encoded,
        [
            /* a TAG = */ 0x99, /* a LENGTH = */ 0x01, /* a VALUE = */ 0x01,
            /* b TAG = */ 0x00, /* b LENGTH = */ 0x17, /* b VALUE = */
            /* b.a TAG = */ 0x99, /* b.a LENGTH = */ 0x01, /* b.a VALUE = */ 0x02,
            /* b.b TAG = */ 0x00, /* b.b LENGTH = */ 0x02, /* b.b VALUE = */ 0x00,
            0x03, /* b.c TAG = */ 0x01, /* b.c LENGTH = */ 0x04,
            /* b.c VALUE = */ 0x00, 0x00, 0x00, 0x04, /* b.d TAG = */ 0x02,
            /* b.d LENGTH = */ 0x08, /* b.d VALUE = */
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05,
        ]
    );

    let decoded_value: Nesting = tlv::from_slice(&encoded).unwrap();
    assert_eq!(decoded_value, value);
}

proptest! {
    #[test]
    fn test_string(s: String) {
        let value = StringContainer { s };
        let encoded = tlv::to_vec(&value).unwrap();

        let decoded_value: StringContainer = tlv::from_slice(&encoded).unwrap();
        assert_eq!(decoded_value, value);
    }
}
