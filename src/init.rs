use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_stdin::MaybeStdin;
use color_eyre::eyre::{bail, Context, Error};

pub mod db;
mod progressbar_logwriter;

#[derive(Debug, Clone, clap::Args)]
pub struct GlobalArgs {
    /// Verbosity log
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct AppArgs {
    #[clap(flatten)]
    pub global_args: GlobalArgs,

    /// Command
    #[command(subcommand)]
    pub command: Subcommands,
}

#[derive(Subcommand, Clone)]
pub enum Subcommands {
    Download(DownloadOpts),
    Authenticate {
        #[command(subcommand)]
        service: AuthorizeCommands,
    },
    Database {
        #[command(subcommand)]
        command: db::DbCommands,
    },
}

#[derive(Subcommand, Clone)]
pub enum AuthorizeCommands {
    GoogleDrive {
        client_id: String,
        client_secret: String,
    },
    Dropbox {
        client_id: String,
    },
}

#[derive(Debug, Clone, clap::Args)]
pub struct DownloadOpts {
    /// Use (or create) cookie file on custom path. Defaults to projects data folder. Varies by OS.
    #[arg(long)]
    pub cookies: Option<std::path::PathBuf>,

    /// yt-dlp path. Will use the environment PATH if not provided.
    #[arg(long)]
    pub yt_dlp: Option<std::path::PathBuf>,

    /// Conversion target directory.
    /// Will create dir if not exist, and set ffmpeg conversion target to this dir.
    #[arg(long, short = 'd', verbatim_doc_comment)]
    pub target_dir: Option<std::path::PathBuf>,

    /// Format: B2;Key ID;App Key;Bucket;BucketID;Root path
    /// Upload downloaded path to OpenDAL supported service instead
    /// Will delete downloaded video unless specified with --skip-video-delete
    #[arg(long, verbatim_doc_comment)]
    pub b2args: Option<String>,

    /// Skip video deletion on upload stage
    #[arg(long, action)]
    pub skip_video_delete: bool,

    /// Removes index number from file name
    #[arg(long, action)]
    pub no_index_filename: bool,

    /// Retry amount
    #[arg(short, long, default_value_t = 5)]
    pub retry: usize,

    /// Download the file first instead of passing url to ffmpeg
    /// Some services will ignore this option due to how their service work.
    /// e.g. Google Drive
    #[arg(long, action)]
    pub download_first: bool,

    /// Path to the input file
    #[arg(
        short,
        long,
        value_name = "FILE",
        conflicts_with = "input_string",
        required_unless_present = "input_string"
    )]
    input_file: Option<PathBuf>,

    /// Input string
    #[arg(
        value_name = "STRING",
        conflicts_with = "input_file",
        required_unless_present = "input_file"
    )]
    input_string: Option<MaybeStdin<String>>,
}

impl DownloadOpts {
    pub fn get_cookie_path(&self) -> PathBuf {
        if let Some(c) = self.cookies.clone() {
            c
        } else {
            crate::statics::PROJECT_DIR_PATH.join("cookie.txt")
        }
    }

    pub fn contents(self) -> Result<String, Error> {
        if let Some(p) = self.input_file {
            std::fs::read_to_string(p).wrap_err("Failed to read file")
        } else if let Some(c) = self.input_string {
            Ok(c.to_string())
        } else {
            bail!("Failed to get contents")
        }
    }
}

const VERBOSE_LEVEL: &[&str] = &["info", "debug", "trace"];

macro_rules! get_this_pkg_name {
    () => {
        env!("CARGO_PKG_NAME").replace('-', "_")
    };
}

pub fn initialize() -> Result<AppArgs, Error> {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    color_eyre::install()?;
    let args = AppArgs::parse();

    let verbosity = match args.global_args.verbose {
        1..=3 => Some(VERBOSE_LEVEL[(args.global_args.verbose as usize) - 1]),
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
            &crate::statics::MPB,
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
