use std::{rc::Rc, sync::LazyLock};

use color_eyre::Report;
use indicatif::ProgressIterator;

mod consts;
mod funcs;
mod init;

mod main_functions;

mod parser;

const FFMPEG_SCALE: &str =
    r#"scale='if(lt(iw,ih),min(1080,iw),-1)':'if(lt(iw,ih),-1,min(1080,ih))'"#;

static MPB: LazyLock<indicatif::MultiProgress> = LazyLock::new(indicatif::MultiProgress::new);
static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
});

#[tracing::instrument]
fn main() -> Result<(), Report> {
    let args = Rc::new(init::initialize()?);

    if let Some(path) = &args.global_args.target_dir {
        if path.exists() && !path.is_dir() {
            panic!("Target path exists and is not a directory");
        }

        if !path.exists() {
            tracing::info!("Target directory not found, creating...");
            std::fs::create_dir_all(path)?;
        }
    }

    let op: Option<opendal::BlockingOperator> = if let Some(ref key) = args.global_args.b2args {
        Some(funcs::setup_opendal(key)?)
    } else {
        None
    };

    let playlist_str = match &args.command {
        init::Subcommands::Playlist { playlist } => playlist.clone().contents()?,
        init::Subcommands::Single { url } => url.clone(),
    };

    let vids = playlist_str
        .lines()
        .filter(parser::line_filter)
        .map(parser::parse_line)
        .collect::<Result<Vec<_>, Report>>()?;

    let total_pb = if vids.len() > 1 {
        MPB.add(funcs::get_progbar(
            vids.len() as u64,
            consts::MAIN_BAR_FMT,
            consts::MAIN_BAR_CHARSET,
        )?)
    } else {
        indicatif::ProgressBar::hidden()
    };

    for (i, (ty, x)) in vids.iter().progress_with(total_pb).enumerate() {
        let i: Option<usize> = if vids.len() > 1 { Some(i) } else { None };

        for _ in 0..args.global_args.retry {
            let run_result = match ty {
                parser::DlTypes::YtDlp => main_functions::handle_ytdlp(&args, i, x, op.clone()),
                parser::DlTypes::DirectLink => {
                    main_functions::handle_direct(&args, i, x, op.clone())
                }
            };

            if run_result.is_ok() {
                break;
            }
        }
    }

    Ok(())
}
