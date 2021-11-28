use std::{
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
};

use anyhow::Result;
use structopt::StructOpt;
use walkdir::WalkDir;

use smplinfo::{midi::Note, wav::Wav};

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

    /// Set the root note
    #[structopt(long)]
    root_note: Option<Note>,

    /// Files and directories to read/write
    paths: Vec<PathBuf>,
}

impl Options {
    fn write(&self) -> bool {
        self.root_note.is_some()
    }
}

pub fn main() -> Result<()> {
    let options = Options::from_args();
    log::debug!("parsed arguments: {:?}", options);

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

    println!("Filename: {}", path.file_name().unwrap().to_string_lossy());
    println!("Path: {}", path.to_string_lossy());

    if let Some(sampler) = wav.get_sampler_chunk()? {
        println!("Root note: {}", sampler.midi_unity_note());
    }

    if options.write() {
        if let Some(note) = options.root_note {
            if options.dry_run {
                println!("Would set root note to {}", note);
            } else {
                wav.update_sampler_chunk(|chunk| {
                    chunk.set_midi_unity_note(note);

                    println!("Set root note to {}", note);
                })?;
            }
        }
    }

    Ok(())
}
