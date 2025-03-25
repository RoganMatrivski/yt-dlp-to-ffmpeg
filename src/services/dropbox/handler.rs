use std::path::PathBuf;

use color_eyre::eyre::ContextCompat;
use indicatif::ProgressIterator;

use crate::{
    consts,
    funcs::{
        ffmpeg::ffmpeg_transcode,
        ffprobe::ffprobe_path,
        opendal::{check_path_exists, copy_path_to_b2},
        progressbar::{create_indefinite_spinner, get_progbar},
    },
    init::DownloadOpts,
    statics::MPB,
};

pub async fn handle_dropbox(
    args: &DownloadOpts,
    i: Option<usize>,
    shared_link: &str,
    op: Option<opendal::Operator>,
) -> Result<(), color_eyre::eyre::Report> {
    let client = super::auth::get_async_client().await?;

    let pb = create_indefinite_spinner(MPB.clone(), format!("Fetching {shared_link}"))?;

    let items = super::walk_shared_link(&client, shared_link, None).await?;

    pb.finish_and_clear();

    let total_dropbox_pb = if items.len() > 1 {
        crate::statics::MPB.add(get_progbar(
            items.len() as u64,
            consts::MAIN_BAR_FMT,
            consts::MAIN_BAR_CHARSET,
        )?)
    } else {
        indicatif::ProgressBar::hidden()
    };

    total_dropbox_pb.set_message("Dropbox Items");

    for (url, path) in items.iter().progress_with(total_dropbox_pb) {
        let url = if url.contains("dl=0") {
            url.replace("dl=0", "dl=1")
        } else if url.contains('?') {
            format!("{}&dl=1", url)
        } else {
            format!("{}?dl=1", url)
        };

        tracing::trace!("Getting {url}");

        let res: Option<i64> = {
            let pb = create_indefinite_spinner(MPB.clone(), format!("Fetching {url}"))?;

            let res = if let Ok(x) = ffprobe_path(&url) {
                let video_stream = x
                    .streams
                    .iter()
                    .find(|x| x.codec_type == Some("video".to_string()))
                    .unwrap();

                Some(video_stream.height.unwrap())
            } else {
                None
            };

            pb.finish_and_clear();

            res
        };

        let title = path
            .file_stem()
            .and_then(|x| x.to_str())
            .wrap_err("File stem somehow ends with '..'")?;

        let _thumbnail = ""; // TODO: Find out how to implement thumbnail in ffmpeg
        let ext = path.extension().and_then(|x| x.to_str()).unwrap_or("mp4");
        let out_name = format!(
            "{idxstr}{title}.{ext}",
            idxstr = if !args.no_index_filename {
                match i {
                    Some(index) => format!("{:05}_", index),
                    None => String::new(),
                }
            } else {
                String::new()
            }
        );

        let out_name = sanitize_filename::sanitize_with_options(
            out_name,
            sanitize_filename::Options {
                windows: true,
                truncate: true,
                replacement: "()",
            },
        );

        let output_path = match args.target_dir {
            Some(ref x) => x.join(out_name),
            None => std::env::current_dir()?.join(out_name),
        };

        if let Some(op) = &op {
            if check_path_exists(&output_path, op).await? {
                tracing::warn!("File already exists on remote");
                continue;
            }
        };

        ffmpeg_transcode(
            &url,
            &output_path,
            format!(
                "{title} ({})",
                res.map(|x| x.to_string()).unwrap_or("".to_string())
            )
            .as_str(),
        )?;

        if let Some(op) = &op {
            copy_path_to_b2(&output_path, op).await?;
            if !args.skip_video_delete {
                std::fs::remove_file(&output_path)?;
            }
        };
    }

    Ok(())
}
