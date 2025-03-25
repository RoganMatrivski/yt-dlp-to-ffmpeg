use color_eyre::eyre::{ContextCompat, Error};

use crate::{
    funcs::{
        ffmpeg::ffmpeg_transcode,
        opendal::{check_path_exists, copy_path_to_b2},
        progressbar::create_indefinite_spinner,
    },
    init::DownloadOpts,
    statics::MPB,
};

pub async fn handle_ytdlp(
    args: &DownloadOpts,
    i: Option<usize>,
    x: &str,
    op: Option<opendal::Operator>,
) -> Result<(), Error> {
    let pb = create_indefinite_spinner(MPB.clone(), format!("Fetching {x}"))?;

    let res = match youtube_dl::YoutubeDl::new(x)
        .youtube_dl_path(args.yt_dlp.clone().unwrap_or("yt-dlp".into()))
        .cookies(args.get_cookie_path().canonicalize()?.to_string_lossy())
        .run()
    {
        Ok(x) => x,
        Err(e) => {
            tracing::error!("# Error: {e}");
            return Ok(());
        }
    };

    pb.finish_and_clear();

    let video = res.into_single_video().wrap_err("Failed to get video")?;
    let formats = video.formats.wrap_err("Failed to get formats")?;
    let bestformat = formats.last().wrap_err("Failed to get bestformat")?;

    let res = bestformat.resolution.clone().unwrap_or("".to_string());

    let id = video.id;
    let title = video.title.wrap_err("Failed to get title")?;
    let _thumbnail = video.thumbnail.wrap_err("Failed to get thumbnail")?; // TODO: Find out how to implement thumbnail in _thumbnailmpeg
    let url = bestformat.url.clone().wrap_err("Failed to get url")?;
    let ext = bestformat.ext.clone().wrap_err("Failed to get ext")?;
    let out_name = format!(
        "{idxstr}{title}_[{id}].{ext}",
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
            return Ok(());
        }
    };

    ffmpeg_transcode(&url, &output_path, format!("{title} ({res})").as_str())?;

    if let Some(op) = &op {
        copy_path_to_b2(&output_path, op).await?;
        if !args.skip_video_delete {
            std::fs::remove_file(&output_path)?;
        }
    };

    Ok(())
}
