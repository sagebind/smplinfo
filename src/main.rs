#![allow(dead_code)]
#![windows_subsystem = "windows"]

use std::env;

use anyhow::Result;

mod cli;
mod futures;
mod midi;
mod sample;
mod ui;
mod wav;
mod workspace;

fn main() -> Result<()> {
    if env::args().len() <= 1 {
        ui::main();
        Ok(())
    } else {
        cli::main()
    }
}
