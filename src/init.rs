use clap::Parser;
use color_eyre::Report;

mod progressbar_logwriter;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Verbosity log
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Use (or create) cookie file
    #[arg(long)]
    pub cookies: std::path::PathBuf,

    /// yt-dlp path. Will use the environment PATH if not provided
    #[arg(long)]
    pub yt_dlp: Option<std::path::PathBuf>,

    /// Conversion target directory.
    /// Will create dir if not exist, and set ffmpeg conversion target to this dir.
    /// If using OpenDAL, will set the target to the OpenDAL directory.
    #[arg(long)]
    pub target_dir: Option<std::path::PathBuf>,

    /// Format: B2;Key ID;App Key;Bucket;BucketID;Root path
    #[arg(long)]
    pub b2arg: Option<String>,

    /// Retry amount
    #[arg(short, long, default_value_t = 3)]
    pub retry: usize,

    // TODO: Expand to be able to use single and playlist
    /// Playlist file
    pub playlist: std::path::PathBuf,
}

const VERBOSE_LEVEL: &[&str] = &["info", "debug", "trace"];

macro_rules! get_this_pkg_name {
    () => {
        env!("CARGO_PKG_NAME").replace('-', "_")
    };
}

pub fn initialize() -> Result<Args, Report> {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    color_eyre::install()?;
    let args = Args::parse();

    let verbosity = match args.verbose {
        1..=3 => Some(VERBOSE_LEVEL[(args.verbose as usize) - 1]),
        _ => None,
    };

    let env_filter = EnvFilter::from_default_env()
        .add_directive(tracing::level_filters::LevelFilter::WARN.into());
    let env_filter = match verbosity {
        Some(v) => env_filter.add_directive(
            format!("{}={}", get_this_pkg_name!(), v)
                .parse()
                .expect("Failed to parse log parameter"),
        ),
        None => env_filter,
    };

    let fmt_layer = fmt::layer().with_writer(move || -> Box<dyn std::io::Write> {
        Box::new(progressbar_logwriter::ProgressBarLogWriter::new(
            std::io::stderr(),
            &crate::MPB,
        ))
    });

    let fmt_layer = match verbosity {
        Some(_) => {
            // construct a layer that prints formatted traces to stderr
            fmt_layer
                .with_level(true) // include levels in formatted output
                .with_thread_ids(true) // include the thread ID of the current thread
                .with_thread_names(true) // include the name of the current thread
        }
        None => {
            // construct a layer that prints formatted traces to stderr
            fmt_layer
        }
    };

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env_filter)
        .with(ErrorLayer::default())
        .init();

    Ok(args)
}
