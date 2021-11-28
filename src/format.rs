use std::{fmt::Write, str::FromStr};

use once_cell::sync::Lazy;
use regex::Regex;
use smplinfo::midi::Note;

/// Format string for a sample filename.
#[derive(Debug)]
pub struct FormatString {
    parts: Vec<FormatPart>,
}

/// A component of a parsed format string.
#[derive(Debug)]
enum FormatPart {
    Literal(String),
    MidiNote,
    Note,
}

impl FormatString {
    /// Format a filename using the given properties.
    pub fn format(&self, root_note: Option<Note>) -> String {
        let mut string = String::new();

        for part in self.parts.iter() {
            match part {
                FormatPart::Literal(literal) => string.push_str(literal.as_str()),
                FormatPart::MidiNote => {
                    if let Some(note) = root_note {
                        write!(string, "{:03}", u8::from(note)).unwrap();
                    }
                }
                FormatPart::Note => {
                    if let Some(note) = root_note {
                        write!(string, "{}", note).unwrap();
                    }
                }
            }
        }

        string
    }
}

impl FromStr for FormatString {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"%.|[^%]+|%$").unwrap());

        let mut parts = Vec::new();

        for m in REGEX.find_iter(s) {
            match m.as_str() {
                ms @ "%%" => {
                    parts.push(FormatPart::Literal(ms.to_owned()));
                }
                "%m" => {
                    parts.push(FormatPart::MidiNote);
                }
                "%n" => {
                    parts.push(FormatPart::Note);
                }
                ms => {
                    if ms.starts_with("%") {
                        panic!("invalid format specifier: {}", ms);
                    } else {
                        parts.push(FormatPart::Literal(ms.to_owned()));
                    }
                }
            }
        }

        Ok(Self { parts })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_produces_expected_string() {
        fn format(format_string: &str, root_note: Option<Note>) -> String {
            FormatString::from_str(format_string)
                .unwrap()
                .format(root_note)
        }

        assert_eq!(format("hello", None), "hello");
        assert_eq!(format("%n", None), "");
        assert_eq!(format("%n", Some(Note::from(60))), "C3");
    }
}
