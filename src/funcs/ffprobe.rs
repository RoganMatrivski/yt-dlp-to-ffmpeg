use color_eyre::eyre::Error;

pub fn ffprobe_path_frametotal(path: impl AsRef<std::path::Path>) -> Option<u64> {
    match ffprobe::ffprobe(path) {
        Err(e) => {
            tracing::warn!("ffprobe error: {e}");
            None
        }
        Ok(i) => {
            let mut count = None;
            for (i, s) in i.streams.iter().enumerate() {
                if let Some(fcount) = &s.nb_frames {
                    let parsed_fcount = match fcount.parse::<u64>() {
                        Ok(f) => f,
                        Err(_) => {
                            tracing::warn!("Failed to parse frame count from ffprobe: {fcount}");
                            return None;
                        }
                    };

                    count = Some(parsed_fcount);
                    break;
                }

                tracing::trace!("Stream #{i} can't find nb_frames");
            }

            count
        }
    }
}

pub fn ffprobe_path(path: impl AsRef<std::path::Path>) -> Result<ffprobe::FfProbe, Error> {
    Ok(ffprobe::ffprobe(path)?)
}
