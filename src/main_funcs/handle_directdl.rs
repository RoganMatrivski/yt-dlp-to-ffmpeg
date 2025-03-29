use color_eyre::eyre::ContextCompat;
use futures_util::StreamExt;

use crate::{
    funcs::{
        ffmpeg::ffmpeg_transcode,
        ffprobe::ffprobe_path,
        opendal::{check_path_exists, copy_path_to_b2},
        progressbar::create_indefinite_spinner,
    },
    init::DownloadOpts,
    statics::MPB,
};

pub async fn handle_direct(
    args: &DownloadOpts,
    i: Option<usize>,
    url: &str,
    op: Option<opendal::Operator>,
) -> Result<(), color_eyre::eyre::Error> {
    let response = reqwest::get(url).await?;
    let content_disposition_str = response
        .headers()
        .get(reqwest::header::CONTENT_DISPOSITION)
        .and_then(|cd| cd.to_str().ok())
        .unwrap();

    let content_disposition =
        &content_disposition::parse_content_disposition(content_disposition_str)
            .filename_full()
            .unwrap();

    let filepath = std::path::Path::new(content_disposition);

    let title = filepath
        .file_stem()
        .and_then(|x| x.to_str())
        .wrap_err("File stem somehow ends with '..'")?;

    let id = title;
    let _thumbnail = ""; // TODO: Find out how to implement thumbnail in ffmpeg
    let ext = filepath
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or("mp4");
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

    let pb = create_indefinite_spinner(MPB.clone(), format!("Fetching {id}"))?;

    let source = if args.download_first {
        let temp_encode_path = output_path.with_file_name(format!(
            "{}_temp{}",
            output_path.file_stem().unwrap().to_string_lossy(),
            output_path
                .extension()
                .map_or(String::new(), |ext| format!(".{}", ext.to_string_lossy()))
        ));

        let file = tokio::fs::File::create(&temp_encode_path).await?;

        let path = tempfile::TempPath::from_path(&temp_encode_path);

        let pb = MPB.add(crate::funcs::progressbar::get_progbar(
            response.content_length().unwrap(), // 'Unwrap' should be fine. If body is None, it'll throw early.
            crate::consts::SUB_BAR_FMT_MSG,
            crate::consts::MAIN_BAR_CHARSET,
        )?);

        let mut res_body = response.bytes_stream();

        let mut wrapped_file = pb.wrap_async_write(file);
        tracing::trace!("Downloading to {}", temp_encode_path.to_string_lossy());

        while let Some(c) = res_body.next().await {
            let c = c?;

            tokio::io::copy(&mut c.as_ref(), &mut wrapped_file).await?;
        }

        pb.finish_and_clear();

        path
    } else {
        tempfile::TempPath::from_path(id)
    };

    let res = ffprobe_path(&source)?;

    pb.finish_and_clear();

    let video_stream = res
        .streams
        .iter()
        .find(|x| x.codec_type == Some("video".to_string()))
        .unwrap();

    let res = video_stream.height.unwrap();

    ffmpeg_transcode(
        &source.to_string_lossy(),
        &output_path,
        format!("{title} ({res})").as_str(),
    )?;

    if let Some(op) = &op {
        copy_path_to_b2(&output_path, op).await?;
        if !args.skip_video_delete {
            std::fs::remove_file(&output_path)?;
        }
    };

    Ok(())
}
