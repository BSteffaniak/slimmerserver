#![allow(clippy::module_name_repetitions)]

use quickcheck::{Arbitrary, Gen};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct XmlString(pub String);

impl Arbitrary for XmlString {
    fn arbitrary(g: &mut Gen) -> Self {
        let string = loop {
            let string = String::arbitrary(g);
            if string.chars().all(is_valid_xml_char) {
                break string;
            }
        };

        Self(string)
    }
}

#[must_use]
pub const fn is_invalid_xml_char(c: char) -> bool {
    matches!(c, '\u{0000}'..='\u{001F}' | '\u{FFFE}'..='\u{FFFF}')
}

#[must_use]
pub const fn is_valid_xml_char(c: char) -> bool {
    !is_invalid_xml_char(c)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct XmlPropNameString(pub String);

impl Arbitrary for XmlPropNameString {
    fn arbitrary(g: &mut Gen) -> Self {
        let string = loop {
            let string = String::arbitrary(g);
            if string
                .chars()
                .all(|c| matches!(c, '0'..='9' | 'A'..='Z' | 'a'..='z' | '-' | '_'))
            {
                break string;
            }
        };

        Self(string)
    }
}
