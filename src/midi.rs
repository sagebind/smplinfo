//! MIDI spec type definitions.

use std::{fmt, str::FromStr};

const NOTE_NAMES: &'static [&'static str] = &["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

/// A MIDI note number.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Note(u8);

impl From<u8> for Note {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<Note> for u8 {
    fn from(note: Note) -> Self {
        note.0
    }
}

impl FromStr for Note {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(number) = s.parse::<u8>() {
            return Ok(number.into());
        }

        for (offset, name) in NOTE_NAMES.into_iter().enumerate() {
            if let Some(suffix) = s.strip_prefix(name) {
                if let Ok(number) = suffix.parse::<u8>() {
                    return Ok(Note::from(((number as usize + 1) * NOTE_NAMES.len() + offset) as u8));
                }
            }
        }

        Err("invalid MIDI note".into())
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 >= 12 {
            let index = (self.0 - 12) as usize;
            f.write_str(NOTE_NAMES[index % NOTE_NAMES.len()])?;

            ((self.0 as usize - NOTE_NAMES.len()) / NOTE_NAMES.len()).fmt(f)
        } else {
            self.0.fmt(f)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        assert_eq!(Note::from_str("0").unwrap(), Note::from(0));
        assert_eq!(Note::from_str("51").unwrap(), Note::from(51));
        assert_eq!(Note::from_str("A0").unwrap(), Note::from(21));
        assert_eq!(Note::from_str("C1").unwrap(), Note::from(24));
    }

    #[test]
    fn display_name() {
        assert_eq!(Note::from(0).to_string(), "0");
        assert_eq!(Note::from(11).to_string(), "11");
        assert_eq!(Note::from(12).to_string(), "C0");
        assert_eq!(Note::from(20).to_string(), "G#0");
        assert_eq!(Note::from(21).to_string(), "A0");
        assert_eq!(Note::from(22).to_string(), "A#0");
        assert_eq!(Note::from(23).to_string(), "B0");
        assert_eq!(Note::from(24).to_string(), "C1");
        assert_eq!(Note::from(127).to_string(), "G9");
    }
}
