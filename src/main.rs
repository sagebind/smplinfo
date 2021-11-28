use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    fs::{self, rename, OpenOptions},
    path::{Path, PathBuf},
    str::FromStr,
};
use structopt::StructOpt;
use walkdir::WalkDir;

use crate::format::FormatString;
use smplinfo::{midi::Note, wav::Wav};

mod format;

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
    rename: Option<FormatString>,

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
        let notes = find_notes_in_string(filename.as_ref()).collect::<Vec<_>>();

        if notes.len() == 1 {
            new_root_note = Some(notes[0]);
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
                drop(wav);
                rename(path, path.with_file_name(new_name))?;
            }
        }
    }

    println!();

    Ok(())
}

fn find_notes_in_string(s: &str) -> impl Iterator<Item = Note> + '_ {
    static REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?:^|[\-_.\s])([A-G]#?-?\d)(?:$|[\-_.\s])").unwrap());

    REGEX
        .captures_iter(s)
        .filter_map(|capture| Note::from_str(&capture[1]).ok())
}
