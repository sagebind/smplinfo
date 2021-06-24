use std::{borrow::Cow, fs::File, path::{Path, PathBuf}};

use anyhow::Result;

use crate::{midi::Note, wav::Wav};

/// A WAV file along with sample attributes.
#[derive(Debug)]
pub struct Sample {
    path: PathBuf,
    note: Option<Note>,
}

impl Sample {
    /// Read a sample from a file.
    ///
    /// This opens the file, validates that it is a WAV file, and scrapes some
    /// metadata, but doesn't load the whole sample into memory.
    pub fn read(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let mut wav = Wav::new(File::open(&path)?)?;
        let mut note = None;

        if let Some(chunk) = wav.get_sampler_chunk()? {
            note = Some(chunk.midi_unity_note());
        }

        Ok(Self { path, note })
    }

    pub fn name(&self) -> Cow<str> {
        self.path.file_name().map(|s| s.to_string_lossy()).unwrap_or(Cow::Borrowed("<unknown>"))
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn note(&self) -> Option<&Note> {
        self.note.as_ref()
    }
}
