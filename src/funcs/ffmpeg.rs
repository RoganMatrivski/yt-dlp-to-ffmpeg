use color_eyre::eyre::{eyre, Error};
use ffmpeg_sidecar::{
    command::FfmpegCommand,
    event::{FfmpegEvent, LogLevel},
};

use crate::{consts::FFMPEG_SCALE, statics::MPB};

use super::{
    ffprobe::ffprobe_path_frametotal,
    progressbar::{get_progbar, get_spinner, update_pb_by_ffmpegprogress},
};

pub fn ffmpeg_transcode<S: AsRef<str>>(
    src: S,
    dst: &std::path::Path,
    progbar_msg: &str,
) -> Result<(), Error> {
    let frame_total = ffprobe_path_frametotal(&src.as_ref());

    let pb = MPB.add(match frame_total {
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
        .input(src)
        .codec_video("libx265")
        .codec_audio("copy")
        .pix_fmt("yuva420p10le")
        .args(["-vf", FFMPEG_SCALE])
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
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(())
}

pub fn ffmpeg_check(src: &std::path::Path) -> Result<(), Error> {
    let frame_total = ffprobe_path_frametotal(src);
    let pb = MPB.add(match frame_total {
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
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(())
}
