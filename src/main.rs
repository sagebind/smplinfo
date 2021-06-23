#![allow(unused)]

use std::env;

use anyhow::Result;

mod cli;
mod midi;
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
