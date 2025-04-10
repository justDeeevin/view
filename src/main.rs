use clap::Parser;
use color_eyre::eyre::{Context, Result};
use console::{Key, Term};
use viuer::{Config, print_from_file};

use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about)]
/// View an image file.
struct Opts {
    /// The image file to print
    file: PathBuf,

    /// The scale factor of the displayed image. Cannot be used in combination with --width or
    /// --height
    #[arg(long, conflicts_with_all = ["width", "height"])]
    scale: Option<f32>,

    /// The width of the displayed image
    #[arg(long, conflicts_with = "scale")]
    width: Option<u32>,

    /// The height of the displayed image
    #[arg(long, conflicts_with = "scale")]
    height: Option<u32>,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let term = Term::stdout();

    let args = Opts::parse();
    let config = Config::default();

    term.clear_screen().wrap_err("Error clearing screen")?;

    print_from_file(&args.file, &config).wrap_err("Error printing image")?;

    while !matches!(
        term.read_key().wrap_err("Error reading key")?,
        Key::Enter | Key::Escape
    ) {}

    term.clear_screen().wrap_err("Error clearing screen")?;

    Ok(())
}
