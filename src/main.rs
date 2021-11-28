use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use stderrlog::new;
use std::{fmt::Write, fs::{self, OpenOptions, rename}, mem::take, path::{Path, PathBuf}, str::FromStr};
use structopt::StructOpt;
use walkdir::WalkDir;

use smplinfo::{midi::Note, wav::Wav};

static ROOT_NOTE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b[A-G]#?-?\d\b").unwrap());

/// WAV sample data reader and writer.
///
/// If no arguments are provided, the GUI will launch.
#[derive(Debug, StructOpt)]
struct Options {
    /// Silence all command output
    #[structopt(short, long)]
    quiet: bool,

    /// Verbose mode (-v, -vv, -vvv, etc)
    #[structopt(short = "v", long, parse(from_occurrences))]
    verbose: usize,

    /// Read/write files in directories recursively
    #[structopt(short, long)]
    recursive: bool,

    /// Don't actually edit any files
    #[structopt(short = "n", long)]
    dry_run: bool,

    /// Rename files using a format string
    ///
    /// The following format characters are supported:
    ///
    /// - %m: MIDI note number of the sample root note
    /// - %n: Root note name
    /// - %%: Percent literal
    #[structopt(long, verbatim_doc_comment)]
    rename: Option<Format>,

    /// Set the root note
    #[structopt(long)]
    root_note: Option<Note>,

    /// Set the root note based on filename
    #[structopt(long)]
    root_note_from_filename: bool,

    /// Files and directories to read/write
    paths: Vec<PathBuf>,
}

impl Options {
    fn write(&self) -> bool {
        self.root_note.is_some() || self.root_note_from_filename
    }
}

pub fn main() -> Result<()> {
    let options = Options::from_args();

    stderrlog::new()
        .quiet(options.quiet)
        .verbosity(options.verbose)
        .init()
        .unwrap();

    for path in &options.paths {
        let metadata = fs::metadata(&path)?;

        if metadata.is_file() {
            process_file(&options, &path)?;
        } else if metadata.is_dir() {
            if options.recursive {
                for entry in WalkDir::new(path) {
                    let entry = entry?;

                    if entry.file_type().is_file() {
                        process_file(&options, entry.path())?;
                    }
                }
            } else {
                log::error!("{:?} is not a file!", path);
            }
        }
    }

    Ok(())
}

fn process_file(options: &Options, path: &Path) -> Result<()> {
    let file = OpenOptions::new()
        .read(true)
        .write(options.write() && !options.dry_run)
        .open(path)?;

    let mut wav = Wav::new(file)?;
    let mut current_root_note = None;
    let mut new_root_note = options.root_note;

    println!("Filename: {}", path.file_name().unwrap().to_string_lossy());
    println!("Path: {}", path.to_string_lossy());

    if let Some(sampler) = wav.get_sampler_chunk()? {
        let note = sampler.midi_unity_note();

        println!("Root note: {} (MIDI {})", note, u8::from(note));

        current_root_note = Some(note);
    }

    if options.root_note_from_filename {
        let filename = path.file_name().unwrap().to_string_lossy();
        let matches = ROOT_NOTE_REGEX
            .find_iter(filename.as_ref())
            .collect::<Vec<_>>();

        if matches.len() == 1 {
            let note = Note::from_str(matches[0].as_str()).unwrap();
            new_root_note = Some(note);
        }
    }

    if let Some(note) = new_root_note {
        if options.dry_run {
            println!("Would set root note to {}", note);
        } else {
            wav.update_sampler_chunk(|chunk| {
                chunk.set_midi_unity_note(note);

                println!("Set root note to {}", note);
            })?;
        }
    }

    if let Some(format) = options.rename.as_ref() {
        let new_name = format.format(new_root_note.or(current_root_note));

        if new_name.as_str() != path.file_name().unwrap() {
            if options.dry_run {
                println!(
                    "Would rename file: {} -> {}",
                    path.file_name().unwrap().to_string_lossy(),
                    new_name
                );
            } else {
                rename(path, path.with_file_name(new_name))?;
                drop(wav);
            }
        }
    }

    println!();

    Ok(())
}

#[derive(Debug)]
struct Format {
    parts: Vec<FormatPart>,
}

#[derive(Debug)]
enum FormatPart {
    Literal(String),
    MidiNote,
    Note,
}

impl Format {
    fn format(&self, root_note: Option<Note>) -> String {
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

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = Vec::new();
        let mut chars = s.chars();
        let mut buf = String::new();

        while let Some(c) = chars.next() {
            if c == '%' {
                match chars.next() {
                    Some('%') | None => buf.push('%'),
                    Some('m') => {
                        if !buf.is_empty() {
                            parts.push(FormatPart::Literal(take(&mut buf)));
                        }
                        parts.push(FormatPart::MidiNote);
                    }
                    Some('n') => {
                        if !buf.is_empty() {
                            parts.push(FormatPart::Literal(take(&mut buf)));
                        }
                        parts.push(FormatPart::Note);
                    }
                    Some(c2) => {
                        buf.push('%');
                        buf.push(c2);
                    }
                }
            } else {
                buf.push(c);
            }
        }

        if !buf.is_empty() {
            parts.push(FormatPart::Literal(buf));
        }

        Ok(Self { parts })
    }
}
