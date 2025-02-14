use ffmpeg_sidecar::{
    command::FfmpegCommand,
    event::{FfmpegEvent, LogLevel},
};

use crate::{
    consts,
    funcs::{self, copy_to_b2, ffprobe_path, ffprobe_path_frametotal},
    init::Args,
    FFMPEG_SCALE, MPB,
};

pub fn handle_ytdlp<T: AsRef<Args>>(
    args: T,
    i: Option<usize>,
    x: &str,
    op: Option<opendal::BlockingOperator>,
) -> Result<(), color_eyre::eyre::Report> {
    let pb = funcs::create_indefinite_spinner(MPB.clone(), format!("Fetching {x}"))?;
    let args = args.as_ref();

    let res = match youtube_dl::YoutubeDl::new(x)
        .youtube_dl_path(args.yt_dlp.clone().unwrap_or("yt-dlp".into()))
        .cookies(args.cookies.canonicalize()?.to_string_lossy())
        .run()
    {
        Ok(x) => x,
        Err(e) => {
            println!("# Error: {e}");
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
        idxstr = if let Some(i) = i {
            format!("{i:05}_")
        } else {
            "".to_string()
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

    let frame_total = ffprobe_path_frametotal(&url)?;

    tracing::trace!("Frame total probe result: {frame_total:?}");

    let pb = MPB.add(match frame_total {
        Some(len) => funcs::get_progbar(len, consts::MAIN_BAR_FMT_MSG, consts::SUB_BAR_CHARSET)?,
        None => funcs::get_spinner(consts::SPINNER_FMT, consts::SPINNER_STRSET_MATERIAL)?,
    });
    pb.tick();
    pb.set_message("0 0/s s:0 b:0kbps");

    let mut ffmpeg = FfmpegCommand::new()
        .input(&url)
        .codec_video("libx265")
        .codec_audio("copy")
        .pix_fmt("yuva420p10le")
        .args(["-vf", FFMPEG_SCALE])
        .output(output_path.to_string_lossy())
        .overwrite()
        .spawn()?;

    ffmpeg.iter().unwrap().for_each(|e| match e {
        FfmpegEvent::Log(LogLevel::Error, e) => println!("Error: {}", e),
        FfmpegEvent::Progress(p) => {
            funcs::update_pb_by_ffmpegprogress(&pb, p, format!("{title} ({res})").as_str())
        }
        _ => {}
    });

    pb.finish_and_clear();

    if let Some(op) = &op {
        copy_to_b2(&output_path, op, &MPB)?;
        std::fs::remove_file(&output_path)?;
    };

    Ok(())
}

pub fn handle_direct<T: AsRef<Args>>(
    args: T,
    i: Option<usize>,
    x: &str,
    op: Option<opendal::BlockingOperator>,
) -> Result<(), color_eyre::eyre::Report> {
    let pb = funcs::create_indefinite_spinner(MPB.clone(), format!("Fetching {x}"))?;
    let args = args.as_ref();

    let res = ffprobe_path(x)?;

    pb.finish_and_clear();

    let video_stream = res
        .streams
        .iter()
        .find(|x| x.codec_type == Some("video".to_string()))
        .unwrap();

    let res = video_stream.height.unwrap();

    let response = reqwest::blocking::get(x)?;
    // let content_disposition = response
    //     .headers()
    //     .get(reqwest::header::CONTENT_DISPOSITION)
    //     .and_then(|cd| cd.to_str().ok())
    //     .and_then(|cd| cd.split("filename=").nth(1))
    //     .map(|filename| filename.trim_matches('"'))
    //     .unwrap();
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
    let url = x;
    let ext = filepath.extension().and_then(|x| x.to_str()).unwrap();
    let out_name = format!(
        "{idxstr}{title}_[{id}].{ext}",
        idxstr = if let Some(i) = i {
            format!("{i:05}_")
        } else {
            "".to_string()
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

    let frame_total = ffprobe_path_frametotal(url)?;

    tracing::trace!("Frame total probe result: {frame_total:?}");

    let pb = MPB.add(match frame_total {
        Some(len) => funcs::get_progbar(len, consts::MAIN_BAR_FMT_MSG, consts::SUB_BAR_CHARSET)?,
        None => funcs::get_spinner(consts::SPINNER_FMT, consts::SPINNER_STRSET_MATERIAL)?,
    });
    pb.tick();
    pb.set_message("0 0/s s:0 b:0kbps");

    let mut ffmpeg = FfmpegCommand::new()
        .input(url)
        .codec_video("libx265")
        .codec_audio("copy")
        .pix_fmt("yuva420p10le")
        .args(["-vf", FFMPEG_SCALE])
        .output(output_path.to_string_lossy())
        .overwrite()
        .spawn()?;

    ffmpeg.iter().unwrap().for_each(|e| match e {
        FfmpegEvent::Log(LogLevel::Error, e) => println!("Error: {}", e),
        FfmpegEvent::Progress(p) => {
            funcs::update_pb_by_ffmpegprogress(&pb, p, format!("{title} ({res})").as_str())
        }
        _ => {}
    });

    pb.finish_and_clear();

    if let Some(op) = &op {
        copy_to_b2(&output_path, op, &MPB)?;
        std::fs::remove_file(&output_path)?;
    };

    Ok(())
}
