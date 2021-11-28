//! WAV format reading and writing routines.

use crate::midi;
use riff::Chunk;
use std::io::{self, Read, Seek, SeekFrom, Write};

pub struct Wav<F> {
    file: F,
    header: Chunk,
}

impl<F: Read + Seek> Wav<F> {
    pub fn new(mut file: F) -> io::Result<Self> {
        let header = Chunk::read(&mut file, 0)?;

        if header.read_type(&mut file)?.as_str() != "WAVE" {
            return Err(io::Error::new(io::ErrorKind::Other, "not a WAV file"));
        }

        Ok(Self { file, header })
    }

    pub fn get_sampler_chunk(&mut self) -> io::Result<Option<SamplerChunk>> {
        if let Some(offset) = self.find_chunk_offset("smpl")? {
            self.file.seek(SeekFrom::Start(offset))?;

            Ok(Some(SamplerChunk::read(&mut self.file)?))
        } else {
            Ok(None)
        }
    }

    fn find_chunk_offset(&mut self, id: &str) -> io::Result<Option<u64>> {
        let chunk = Chunk::read(&mut self.file, 0)?;

        for child in chunk.iter(&mut self.file) {
            if child.id().as_str() == id {
                return Ok(Some(child.offset()));
            }
        }

        Ok(None)
    }
}

impl<F: Read + Seek + Write> Wav<F> {
    pub fn update_sampler_chunk(&mut self, f: impl FnOnce(&mut SamplerChunk)) -> io::Result<()> {
        if let Some(offset) = self.find_chunk_offset("smpl")? {
            self.file.seek(SeekFrom::Start(offset))?;
            let mut chunk = SamplerChunk::read(&mut self.file)?;
            f(&mut chunk);

            self.file.seek(SeekFrom::Start(offset))?;
            self.file.write_all(chunk.as_ref())?;
        } else {
            let mut chunk = SamplerChunk::default();
            f(&mut chunk);

            self.file.seek(SeekFrom::End(0))?;
            self.file.write_all(chunk.as_ref())?;

            let new_size = self.header.len() + chunk.as_ref().len() as u32;
            self.file.seek(SeekFrom::Start(4))?;
            self.file.write_all(&new_size.to_le_bytes())?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct SamplerChunk {
    header: [u8; 44],
}

impl Default for SamplerChunk {
    fn default() -> Self {
        let mut header = [0; 44];

        (&mut header[..4]).copy_from_slice(Self::ID);
        header[5] = 36;

        Self { header }
    }
}

impl SamplerChunk {
    const ID: &'static [u8] = b"smpl";

    fn read<R>(mut reader: R) -> io::Result<Self>
    where
        R: Read + Seek,
    {
        let mut header = [0; 44];
        reader.read_exact(&mut header)?;

        if &header[..4] != Self::ID {
            return Err(io::Error::new(io::ErrorKind::Other, "invalid smpl chunk"));
        }

        Ok(Self { header })
    }

    pub fn midi_unity_note(&self) -> midi::Note {
        self.header[0x14].into()
    }

    pub fn set_midi_unity_note(&mut self, note: midi::Note) {
        self.header[0x14] = note.into();
    }
}

impl AsRef<[u8]> for SamplerChunk {
    fn as_ref(&self) -> &[u8] {
        &self.header
    }
}
