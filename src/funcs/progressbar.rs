use color_eyre::eyre::Error;

pub fn get_spinner(
    spinner_fmt: &str,
    spinner_strset: &[&str],
) -> Result<indicatif::ProgressBar, Error> {
    Ok(indicatif::ProgressBar::new_spinner().with_style(
        indicatif::ProgressStyle::with_template(spinner_fmt)?.tick_strings(spinner_strset),
    ))
}

pub fn get_progbar(
    len: u64,
    bar_fmt: &str,
    bar_char: &str,
) -> Result<indicatif::ProgressBar, Error> {
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
) -> Result<indicatif::ProgressBar, Error> {
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
