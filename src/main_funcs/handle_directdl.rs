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
    id: &str,
    op: Option<opendal::Operator>,
) -> Result<(), color_eyre::eyre::Error> {
    let pb = create_indefinite_spinner(MPB.clone(), format!("Fetching {id}"))?;

    let res = ffprobe_path(id)?;

    pb.finish_and_clear();

    let video_stream = res
        .streams
        .iter()
        .find(|x| x.codec_type == Some("video".to_string()))
        .unwrap();

    let res = video_stream.height.unwrap();

    let response = reqwest::blocking::get(id)?;
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

    let title = filepath.file_stem().and_then(|x| x.to_str()).unwrap();

    let id = title;
    let _thumbnail = ""; // TODO: Find out how to implement thumbnail in ffmpeg
    let url = id;
    let ext = filepath.extension().and_then(|x| x.to_str()).unwrap();
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
