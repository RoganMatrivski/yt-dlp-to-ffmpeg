use color_eyre::eyre::Report;

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
