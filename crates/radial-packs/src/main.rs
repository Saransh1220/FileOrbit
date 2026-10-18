//! `radial`: choose and install the converters you want.

use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use indicatif::{ProgressBar, ProgressStyle};
use radial_packs::catalog::PACKS;
use radial_packs::installer::{Action, Event, Installer, Plan};
use radial_packs::state::{Source, State};
use std::io::IsTerminal;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "radial", version, about = "Choose which converters Radial Convert installs")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Install converter packs (interactive picker when no packs are given)
    Install {
        /// Packs to install, comma separated (e.g. audio,images)
        #[arg(long, value_delimiter = ',')]
        packs: Vec<String>,
        /// Install every pack
        #[arg(long)]
        all: bool,
        /// Skip the confirmation question
        #[arg(short, long)]
        yes: bool,
    },
    /// Show packs and whether they are installed
    List,
    /// Remove packs (engines no other pack needs are deleted too)
    Remove {
        #[arg(required = true)]
        packs: Vec<String>,
    },
    /// Run every installed engine and report what works
    Doctor,
    /// Print the install folder
    Where,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let installer = Installer::for_this_machine()?;
    match cli.command {
        Command::Install { packs, all, yes } => install(&installer, packs, all, yes),
        Command::List => list(&installer),
        Command::Remove { packs } => remove(&installer, &packs),
        Command::Doctor => doctor(&installer),
        Command::Where => {
            println!("{}", installer.home().display());
            Ok(())
        }
    }
}

fn human(bytes: u64) -> String {
    const MB: f64 = 1024.0 * 1024.0;
    if bytes as f64 >= 1024.0 * MB {
        format!("{:.1} GB", bytes as f64 / (1024.0 * MB))
    } else {
        format!("{:.0} MB", bytes as f64 / MB)
    }
}

fn list(installer: &Installer) -> Result<(), String> {
    let state = installer.state()?;
    println!("Install folder: {}\n", installer.home().display());
    for pack in PACKS {
        let mark = if state.packs.contains(pack.id) { "installed" } else { "-" };
        println!("  {:<10} {:<14} {}  [needs {}]", pack.id, mark, pack.description, pack.engines.join(", "));
    }
    println!("\nInstalled engines:");
    if state.engines.is_empty() {
        println!("  none yet. Run `radial install`.");
    }
    for (id, engine) in &state.engines {
        let source = match engine.source {
            Source::Downloaded => "downloaded",
            Source::System => "system copy",
        };
        println!("  {id:<12} {} ({source}) {}", engine.version, engine.path.display());
    }
    Ok(())
}

fn choose_packs(state: &State) -> Result<Vec<String>, String> {
    if !std::io::stdin().is_terminal() {
        return Err("no terminal to ask in; pass --packs audio,images or --all".into());
    }
    let labels: Vec<String> = PACKS
        .iter()
        .map(|p| {
            let tag = if state.packs.contains(p.id) { "  (installed)" } else { "" };
            format!("{:<14} {}{tag}", p.title, p.description)
        })
        .collect();
    let defaults: Vec<bool> = PACKS.iter().map(|p| state.packs.contains(p.id)).collect();
    let picked = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Which converters do you want? (space to toggle, enter to confirm)")
        .items(&labels)
        .defaults(&defaults)
        .interact()
        .map_err(|e| format!("selection cancelled: {e}"))?;
    Ok(picked
        .into_iter()
        .map(|i| PACKS[i].id.to_string())
        .filter(|id| !state.packs.contains(id))
        .collect())
}

fn describe(plan: &Plan) {
    println!("\nThis will set up:");
    for item in &plan.items {
        let what = match &item.action {
            Action::AlreadyInstalled => "already installed".to_string(),
            Action::Adopt { path } => format!("use the copy on this machine ({})", path.display()),
            Action::Download { version, artifact } => {
                format!("download version {version} ({})", human(artifact.size_bytes))
            }
            Action::Unavailable { reason } => format!("cannot install: {reason}"),
        };
        println!("  {:<12} {what}", item.engine);
    }
    if plan.download_bytes() > 0 {
        println!("\nTotal download: {}", human(plan.download_bytes()));
    }
}

fn install(installer: &Installer, packs: Vec<String>, all: bool, yes: bool) -> Result<(), String> {
    let state = installer.state()?;
    let selected: Vec<String> = if all {
        PACKS.iter().map(|p| p.id.to_string()).collect()
    } else if packs.is_empty() {
        choose_packs(&state)?
    } else {
        packs
    };
    if selected.is_empty() {
        println!("Nothing selected.");
        return Ok(());
    }

    let plan = installer.plan(&selected)?;
    describe(&plan);
    if !plan.blockers().is_empty() {
        return Err("some engines cannot be installed (see above)".into());
    }
    let interactive = std::io::stdin().is_terminal();
    if !yes && interactive {
        let go = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Continue?")
            .default(true)
            .interact()
            .map_err(|e| e.to_string())?;
        if !go {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let bar = ProgressBar::hidden();
    let style = ProgressStyle::with_template("{prefix:<12} [{bar:30}] {bytes}/{total_bytes}")
        .map_err(|e| e.to_string())?
        .progress_chars("=> ");
    let mut on_event = |event: Event| match event {
        Event::Downloading { engine, done, total } => {
            if bar.is_hidden() {
                bar.set_draw_target(indicatif::ProgressDrawTarget::stderr());
            }
            bar.set_style(style.clone());
            bar.set_prefix(engine);
            bar.set_length(total.unwrap_or(done));
            bar.set_position(done);
        }
        Event::Verifying { engine } => {
            bar.finish_and_clear();
            println!("{engine}: verifying checksum");
        }
        Event::Extracting { engine } => println!("{engine}: unpacking"),
        Event::Adopted { engine, path } => println!("{engine}: using {}", path.display()),
        Event::Installed { engine } => println!("{engine}: installed"),
    };
    installer.install(&selected, &mut on_event)?;
    println!("\nInstalled: {}", selected.join(", "));
    Ok(())
}

fn remove(installer: &Installer, packs: &[String]) -> Result<(), String> {
    for pack in packs {
        let engines = installer.remove(pack)?;
        if engines.is_empty() {
            println!("Removed pack {pack}.");
        } else {
            println!("Removed pack {pack} and engine(s): {}.", engines.join(", "));
        }
    }
    Ok(())
}

fn doctor(installer: &Installer) -> Result<(), String> {
    let lines = installer.doctor()?;
    if lines.is_empty() {
        println!("No engines installed.");
        return Ok(());
    }
    let mut failed = false;
    for line in &lines {
        let mark = if line.ok { "ok  " } else { "FAIL" };
        println!("  {mark} {:<12} {}", line.engine, line.detail);
        failed |= !line.ok;
    }
    if failed {
        Err("some engines do not run; reinstall their packs".into())
    } else {
        Ok(())
    }
}
