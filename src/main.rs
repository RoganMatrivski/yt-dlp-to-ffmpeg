use std::rc::Rc;

use color_eyre::Report;
use indicatif::ProgressIterator;

mod consts;
mod funcs;
mod init;
mod main_funcs;
mod services;
mod statics;
mod structs;

mod parser;

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<(), Report> {
    let args = Rc::new(init::initialize()?);

    let args = match &args.command {
        init::Subcommands::Authenticate { service } => {
            match &service {
                init::AuthorizeCommands::GoogleDrive {
                    client_id,
                    client_secret,
                } => services::google_drive::auth::authenticate(&client_id, &client_secret).await?,
                init::AuthorizeCommands::Dropbox { client_id } => {
                    services::dropbox::auth::authenticate(client_id).await?;
                }
            };

            return Ok(());
        }
        init::Subcommands::Database { command } => {
            return init::db::handle_db_commands(command).await;

            // return Ok(());
        }
        init::Subcommands::Download(download_opts) => download_opts,
        _ => unimplemented!(),
    };

    if let Some(path) = &args.target_dir {
        if path.exists() && !path.is_dir() {
            panic!("Target path exists and is not a directory");
        }

        if !path.exists() {
            tracing::info!("Target directory not found, creating...");
            std::fs::create_dir_all(path)?;
        }
    }

    let op: Option<opendal::Operator> = if let Some(ref key) = args.b2args {
        Some(funcs::opendal::setup_opendal(key)?)
    } else {
        None
    };

    let playlist_str = args.clone().contents()?;

    let vids = playlist_str
        .lines()
        .filter(parser::line_filter)
        .map(parser::parse_line)
        .collect::<Result<Vec<_>, Report>>()?;

    let total_pb = if vids.len() > 1 {
        statics::MPB.add(funcs::progressbar::get_progbar(
            vids.len() as u64,
            consts::MAIN_BAR_FMT,
            consts::MAIN_BAR_CHARSET,
        )?)
    } else {
        indicatif::ProgressBar::hidden()
    };

    for (i, (ty, x)) in vids.iter().progress_with(total_pb).enumerate() {
        let i: Option<usize> = if vids.len() > 1 { Some(i) } else { None };

        for retry_num in 0..args.retry {
            let run_result = match ty {
                parser::DlTypes::YtDlp => main_funcs::handle_ytdlp(args, i, x, op.clone()).await,
                parser::DlTypes::DirectLink => {
                    main_funcs::handle_directdl::handle_direct(args, i, x, op.clone()).await
                }
                parser::DlTypes::GoogleDrive => {
                    services::google_drive::handle_google_drive(args, i, x, op.clone()).await
                }
                parser::DlTypes::Dropbox => {
                    services::dropbox::handler::handle_dropbox(args, i, x, op.clone()).await
                }
            };

            let is_inner_retry = {
                match ty {
                    parser::DlTypes::GoogleDrive => true,
                    parser::DlTypes::Dropbox => true,
                    _ => false,
                }
            };

            if run_result.is_ok() {
                break;
            }

            let line_pos_str = i.map_or("".to_string(), |x| format!(" at line {}", x + 1));

            tracing::warn!(
                "Attempt #{retry_num}{line_pos_str} failed. Reason: {}",
                run_result.unwrap_err()
            );
        }
    }

    Ok(())
}
