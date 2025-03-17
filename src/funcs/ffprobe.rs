use color_eyre::eyre::Error;

pub fn ffprobe_path_frametotal(path: impl AsRef<std::path::Path>) -> Result<Option<u64>, Error> {
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

pub fn ffprobe_path(path: impl AsRef<std::path::Path>) -> Result<ffprobe::FfProbe, Error> {
    Ok(ffprobe::ffprobe(path)?)
}
