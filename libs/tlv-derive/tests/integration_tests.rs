use proptest::proptest;
use tlv::{Decode, Encode};
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

#[derive(Debug, PartialEq, Encode)]
struct Nesting {
    #[tlv(tag = 0x99)]
    a: u8,
    #[tlv(tag = 0x00)]
    b: Test,
}

#[derive(Debug, PartialEq, Encode)]
struct StringContainer {
    #[tlv(tag = 0x00)]
    s: String,
}

#[test]
fn test_derive() {
    let mut buf = Vec::new();
    let mut encoder = tlv::Encoder::new(&mut buf);
    let value = Test {
        a: 1,
        b: 2,
        c: 3,
        d: 4,
    };
    value.encode(&mut encoder).unwrap();

    assert_eq!(
        buf,
        vec![
            /* a TAG = */ 0x99, /* a LENGTH = */ 0x01, /* a VALUE = */ 0x01,
            /* b TAG = */ 0x00, /* b LENGTH = */ 0x02, /* b VALUE = */ 0x00, 0x02,
            /* c TAG = */ 0x01, /* c LENGTH = */ 0x04, /* c VALUE = */ 0x00, 0x00,
            0x00, 0x03, /* d TAG = */ 0x02, /* d LENGTH = */ 0x08, /* d VALUE = */
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04,
        ]
    );

    let mut decoder = tlv::Decoder::new(&buf[..]);
    let decoded_value = Test::decode(&mut decoder, &tlv::Length::new(buf.len() as u16)).unwrap();

    assert_eq!(decoded_value, value);
}

#[test]
fn test_nesting() {
    let mut buf = Vec::new();
    let mut encoder = tlv::Encoder::new(&mut buf);
    let test = Nesting {
        a: 1,
        b: Test {
            a: 2,
            b: 3,
            c: 4,
            d: 5,
        },
    };
    test.encode(&mut encoder).unwrap();

    assert_eq!(
        buf,
        vec![
            /* a TAG = */ 0x99, /* a LENGTH = */ 0x01, /* a VALUE = */ 0x01,
            /* b TAG = */ 0x00, /* b LENGTH = */ 0x0f, /* b VALUE = */
            /* b.a TAG = */ 0x99, /* b.a LENGTH = */ 0x01, /* b.a VALUE = */ 0x02,
            /* b.b TAG = */ 0x00, /* b.b LENGTH = */ 0x02, /* b.b VALUE = */ 0x00,
            0x03, /* b.c TAG = */ 0x01, /* b.c LENGTH = */ 0x04,
            /* b.c VALUE = */ 0x00, 0x00, 0x00, 0x04, /* b.d TAG = */ 0x02,
            /* b.d LENGTH = */ 0x08, /* b.d VALUE = */
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05,
        ]
    )
}

proptest! {
    #[test]
    fn test_string(value: String) {
        let mut buf = Vec::new();
        let mut encoder = tlv::Encoder::new(&mut buf);
        let container = StringContainer { s: value.clone() };
        container.encode(&mut encoder).unwrap();

        let mut decoder = tlv::Decoder::new(&buf[..]);
        let decoded: String = decoder.decode(&tlv::Tag::U8(0x00)).unwrap();
        assert_eq!(decoded, value);
        assert_eq!(decoder.remaining().unwrap(), vec![]);
    }
}
