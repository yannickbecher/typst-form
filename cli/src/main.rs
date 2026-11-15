mod appearance;
mod embedded;
mod fonts;
mod form;
mod install;
mod model;
mod pipeline;
mod tagging;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Result, bail, ensure};
use clap::{Parser, Subcommand};

/// Turns PDFs compiled with the `typst-form` Typst package into fillable forms.
///
/// Set up once with `typst-form install` (installs the Typst package).
///
/// The package embeds all field definitions into the PDF; this tool adds the
/// real form fields in place. Let Typst run it after compiling:
///
///   typst compile form.typ --open typst-form
///
/// or keep it running next to `typst watch`:
///
///   typst-form watch form.pdf
#[derive(Parser)]
#[command(version = env!("TYPST_FORM_VERSION"), about, args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// PDFs written by `typst compile` (changed in place).
    pdfs: Vec<PathBuf>,

    /// Write to this file instead (only with a single PDF).
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Open the result in the default PDF viewer.
    #[arg(long, global = true, env = "TYPST_FORM_VIEW")]
    view: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Add the form every time Typst rewrites the PDF.
    Watch { pdf: PathBuf },
    /// Install the bundled Typst package as @local/typst-form.
    Install,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    if let Some(Command::Install) = cli.command {
        let dir = install::install()?;
        eprintln!("installed to {}", dir.display());
        println!("#import \"@local/typst-form:{}\": *", install::version());
        return Ok(());
    }
    if let Some(Command::Watch { pdf }) = cli.command {
        eprintln!("watching {} (Ctrl+C to stop)", pdf.display());
        let mut view = cli.view;
        pipeline::watch(&pdf, || {
            let time = pipeline::clock();
            match pipeline::process_file(&pdf, &pdf) {
                Ok(Some(stats)) => {
                    report(&format!("[{time}]"), &stats);
                    if std::mem::take(&mut view) {
                        open(&pdf);
                    }
                }
                Ok(None) => {}
                Err(err) => eprintln!("[{time}] error: {err:#}"),
            }
        });
    }

    ensure!(!cli.pdfs.is_empty(), "no PDF given (see --help)");
    ensure!(
        cli.output.is_none() || cli.pdfs.len() == 1,
        "--output needs exactly one PDF"
    );
    for pdf in &cli.pdfs {
        if pdf.extension().is_some_and(|e| e == "typ") {
            bail!(
                "{} is a Typst source; compile it with `typst compile {} --open typst-form`",
                pdf.display(),
                pdf.display()
            );
        }
        let output = cli.output.as_deref().unwrap_or(pdf);
        match pipeline::process_file(pdf, output)? {
            Some(stats) => report(&output.display().to_string(), &stats),
            None => eprintln!(
                "{}: no form fields found (does it use the typst-form package?), unchanged",
                pdf.display()
            ),
        }
        if cli.view {
            open(output);
        }
    }
    Ok(())
}

fn report(prefix: &str, stats: &form::Stats) {
    eprintln!(
        "{prefix} {} fields, {} widgets, {} scripts",
        stats.fields, stats.widgets, stats.scripts
    );
}

fn open(path: &Path) {
    if let Err(err) = open::that_detached(path) {
        eprintln!("warning: could not open viewer: {err}");
    }
}
