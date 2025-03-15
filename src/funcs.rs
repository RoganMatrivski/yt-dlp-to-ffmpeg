use color_eyre::eyre::{eyre, Context, Report};
use ffmpeg_sidecar::{
    command::FfmpegCommand,
    event::{FfmpegEvent, LogLevel},
};

pub fn ffprobe_path_frametotal(path: impl AsRef<std::path::Path>) -> Result<Option<u64>, Report> {
    match ffprobe::ffprobe(path) {
        Err(e) => {
            tracing::warn!("ffprobe error: {e}");
            Ok(None)
        }
        Ok(i) => {
            let mut count = None;
            for (i, s) in i.streams.iter().enumerate() {
                if let Some(fcount) = &s.nb_frames {
                    count = Some(fcount.parse::<u64>()?);
                    break;
                }

                tracing::trace!("Stream #{i} can't find nb_frames");
            }

            Ok(count)
        }
    }
}

pub fn ffprobe_path_checkerrors(path: impl AsRef<std::path::Path>) -> bool {
    match ffprobe::ffprobe(path) {
        Err(e) => {
            tracing::warn!("ffprobe error: {e}");
            true
        }
        Ok(_) => false,
    }
}

pub fn ffprobe_path(path: impl AsRef<std::path::Path>) -> Result<ffprobe::FfProbe, Report> {
    Ok(ffprobe::ffprobe(path)?)
}

pub fn get_spinner(
    spinner_fmt: &str,
    spinner_strset: &[&str],
) -> Result<indicatif::ProgressBar, Report> {
    Ok(indicatif::ProgressBar::new_spinner().with_style(
        indicatif::ProgressStyle::with_template(spinner_fmt)?.tick_strings(spinner_strset),
    ))
}

pub fn get_progbar(
    len: u64,
    bar_fmt: &str,
    bar_char: &str,
) -> Result<indicatif::ProgressBar, Report> {
    Ok(indicatif::ProgressBar::new(len)
        .with_style(indicatif::ProgressStyle::with_template(bar_fmt)?.progress_chars(bar_char)))
}

pub fn update_pb_by_ffmpegprogress(
    pb: &indicatif::ProgressBar,
    event: ffmpeg_sidecar::event::FfmpegProgress,
    filename: &str,
) {
    pb.set_position(event.frame as u64);

    pb.set_message(format!(
        "[+] {filename} | {fps}/s s:{size} b:{bitrate}kbps",
        fps = event.fps,
        size = event.size_kb,
        bitrate = event.bitrate_kbps
    ))
}

pub fn create_indefinite_spinner(
    mpb: indicatif::MultiProgress,
    msg: impl Into<std::borrow::Cow<'static, str>>,
) -> Result<indicatif::ProgressBar, Report> {
    let pb = mpb.add(
        get_spinner(
            crate::consts::SPINNER_FMT,
            crate::consts::SPINNER_STRSET_DOTS12,
        )?
        .with_message(msg),
    );

    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    Ok(pb)
}

pub fn setup_opendal(service_string: &str) -> Result<opendal::BlockingOperator, Report> {
    let [service, id, key, bucket, bucket_id, rootpath] =
        service_string.split(';').collect::<Vec<_>>()[0..6]
    else {
        color_eyre::eyre::bail!("Error parsing service string")
    };

    let builder = match service {
        "B2" | "b2" => {
            opendal::services::B2::default()
                // set the storage bucket for OpenDAL
                .root(rootpath)
                // set the application_key_id for OpenDAL
                .application_key_id(id)
                // set the application_key for OpenDAL
                .application_key(key)
                // set the bucket name for OpenDAL
                .bucket(bucket)
                .bucket_id(bucket_id)
        }
        others => unimplemented!("{others} service is not implemented"),
    };

    let _guard = crate::RUNTIME.enter();
    Ok(opendal::Operator::new(builder)?
        .layer(opendal::layers::BlockingLayer::create()?)
        .layer(
            opendal::layers::RetryLayer::new()
                .with_factor(2.0)
                .with_max_times(128),
        )
        .finish()
        .blocking())
}

pub fn copy_to_b2(
    filepath: &std::path::Path,
    op: &opendal::BlockingOperator,
    mpb: &indicatif::MultiProgress,
) -> Result<(), Report> {
    let file = std::fs::File::open(filepath)?;
    let filename = filepath.file_name().unwrap().to_string_lossy();

    let output_filesize = file.metadata()?.len();

    let pb = mpb.add(crate::funcs::get_progbar(
        output_filesize,
        crate::consts::SUB_BAR_FMT_MSG,
        crate::consts::MAIN_BAR_CHARSET,
    )?);

    let mut wrapped_file = pb.wrap_read(file);

    let mut writer = op
        .writer_with(&filename)
        .append(false)
        .call()?
        .into_std_write();

    std::io::copy(&mut wrapped_file, &mut writer)?;

    Ok(())
}

pub fn ffmpeg_transcode(
    frame_total: Option<u64>,
    src: &std::path::Path,
    dst: &std::path::Path,
    progbar_msg: &str,
) -> Result<(), Report> {
    let pb = crate::MPB.add(match frame_total {
        Some(len) => get_progbar(
            len,
            crate::consts::MAIN_BAR_FMT_MSG,
            crate::consts::SUB_BAR_CHARSET,
        )?,
        None => get_spinner(
            crate::consts::SPINNER_FMT,
            crate::consts::SPINNER_STRSET_MATERIAL,
        )?,
    });
    pb.tick();
    pb.set_message("0 0/s s:0 b:0kbps");

    let mut ffmpeg = FfmpegCommand::new()
        .input(src.to_string_lossy())
        .codec_video("libx265")
        .codec_audio("copy")
        .pix_fmt("yuva420p10le")
        .args(["-vf", crate::FFMPEG_SCALE])
        .output(dst.to_string_lossy())
        .overwrite()
        .spawn()?;

    ffmpeg
        .iter()
        .map_err(|m| eyre!(m))?
        .map(|e| {
            match e {
                FfmpegEvent::Log(LogLevel::Error, e) => color_eyre::eyre::bail!(e),
                FfmpegEvent::Progress(p) => update_pb_by_ffmpegprogress(&pb, p, progbar_msg),
                _e => {}
            };

            color_eyre::eyre::Ok(())
        })
        .collect::<Result<Vec<_>, Report>>()?;

    Ok(())
}

pub fn ffmpeg_check(src: &std::path::Path) -> Result<(), Report> {
    let frame_total = ffprobe_path_frametotal(src)?;
    let pb = crate::MPB.add(match frame_total {
        Some(len) => get_progbar(
            len,
            crate::consts::MAIN_BAR_FMT_MSG,
            crate::consts::SUB_BAR_CHARSET,
        )?,
        None => get_spinner(
            crate::consts::SPINNER_FMT,
            crate::consts::SPINNER_STRSET_MATERIAL,
        )?,
    });
    pb.tick();
    pb.set_message("0 0/s s:0 b:0kbps");

    let mut ffmpeg = FfmpegCommand::new()
        .input(src.to_string_lossy())
        .format("null")
        .output("-")
        .spawn()?;

    ffmpeg
        .iter()
        .map_err(|m| eyre!(m))?
        .map(|e| {
            match e {
                FfmpegEvent::Log(LogLevel::Error, e) => color_eyre::eyre::bail!(e),
                FfmpegEvent::Progress(p) => update_pb_by_ffmpegprogress(&pb, p, "Checking file"),
                _e => {}
            };

            color_eyre::eyre::Ok(())
        })
        .collect::<Result<Vec<_>, Report>>()?;

    Ok(())
}

pub fn get_md5_from_path(path: &std::path::Path) -> Result<String, color_eyre::eyre::Report> {
    let file = std::fs::File::open(path)?;
    let pb = crate::MPB.add(crate::funcs::get_progbar(
        file.metadata()
            .wrap_err("Failed to get existing file size")?
            .len(),
        crate::consts::SUB_BAR_FMT_MSG,
        crate::consts::MAIN_BAR_CHARSET,
    )?);
    let file = pb.wrap_read(file);

    get_md5(file)
}

pub fn get_md5<T: std::io::Read>(reader: T) -> Result<String, color_eyre::eyre::Report> {
    let mut md5writer = crate::ext_functions::md5writer::Md5Writer::new(std::io::sink());
    let mut reader = reader;
    std::io::copy(&mut reader, &mut md5writer)?;
    Ok(md5writer.md5())
}
