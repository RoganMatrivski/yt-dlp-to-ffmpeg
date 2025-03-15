use std::{rc::Rc, sync::LazyLock};

use color_eyre::Report;
use indicatif::ProgressIterator;

mod consts;
mod init;

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

static TOKEN_DIR_PATH: LazyLock<std::path::PathBuf> = LazyLock::new(|| {
    let dirpath = match directories::ProjectDirs::from(
        consts::APP_ID[0],
        consts::APP_ID[1],
        std::env!("CARGO_PKG_NAME"),
    ) {
        Some(x) => x.data_dir().to_path_buf(),
        None => std::env::current_dir().expect("Failed to get current directory"),
    };

    std::fs::create_dir_all(&dirpath).unwrap();

    dirpath
});

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<(), Report> {
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

    let op: Option<opendal::Operator> = if let Some(ref key) = args.global_args.b2args {
        Some(funcs::setup_opendal(key)?)
    } else {
        None
    };

    Ok(())
}
