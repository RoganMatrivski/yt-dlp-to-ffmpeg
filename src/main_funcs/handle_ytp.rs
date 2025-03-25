use color_eyre::eyre::Error;

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

    let video = res.into_single_video().unwrap();
    let formats = video.formats.unwrap();
    let bestformat = formats.last().unwrap();

    let res = bestformat.resolution.clone().unwrap();

    let id = video.id;
    let title = video.title.unwrap();
    let _thumbnail = video.thumbnail.unwrap(); // TODO: Find out how to implement thumbnail in ffmpeg
    let url = bestformat.url.clone().unwrap();
    let ext = bestformat.ext.clone().unwrap();
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
