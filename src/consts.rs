#![allow(dead_code)]

pub const MAIN_BAR_FMT: &str =
    "[{elapsed_precise}] {wide_bar:.blue} {pos:>}/{len} ({percent}%) eta {eta_precise:.blue}";
pub const MAIN_BAR_FMT_MSG: &str =
    "{msg} {wide_bar:.blue} {pos:>}/{len} ({percent}%) eta {eta_precise:.blue}";
pub const SUB_BAR_FMT: &str = "{wide_bar:.blue} {bytes:>11.green}/{total_bytes:<11.green} {bytes_per_sec:>13.red} eta {eta:.blue}";
pub const SUB_BAR_FMT_MSG: &str = "{msg} {wide_bar:.blue} {bytes:>11.green}/{total_bytes:<11.green} {bytes_per_sec:>13.red} eta {eta:.blue}";
pub const MAIN_BAR_CHARSET: &str = "==>-";
pub const SUB_BAR_CHARSET: &str = "█▉▊▋▌▍▎▏  ";

pub const SPINNER_FMT: &str = "{spinner} [{elapsed_precise}] {wide_msg}";
pub const SPINNER_STRSET_DOTS12: &[&str; 56] = &[
    "⢀⠀", "⡀⠀", "⠄⠀", "⢂⠀", "⡂⠀", "⠅⠀", "⢃⠀", "⡃⠀", "⠍⠀", "⢋⠀", "⡋⠀", "⠍⠁", "⢋⠁", "⡋⠁", "⠍⠉", "⠋⠉",
    "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⢈⠩", "⡀⢙", "⠄⡙", "⢂⠩", "⡂⢘", "⠅⡘", "⢃⠨", "⡃⢐", "⠍⡐", "⢋⠠",
    "⡋⢀", "⠍⡁", "⢋⠁", "⡋⠁", "⠍⠉", "⠋⠉", "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⠈⠩", "⠀⢙", "⠀⡙", "⠀⠩",
    "⠀⢘", "⠀⡘", "⠀⠨", "⠀⢐", "⠀⡐", "⠀⠠", "⠀⢀", "⠀⡀",
];
pub const SPINNER_STRSET_MATERIAL: &[&str; 92] = &[
    "█▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "██▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "███▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "████▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "██████▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "██████▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "███████▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "████████▁▁▁▁▁▁▁▁▁▁▁▁",
    "█████████▁▁▁▁▁▁▁▁▁▁▁",
    "█████████▁▁▁▁▁▁▁▁▁▁▁",
    "██████████▁▁▁▁▁▁▁▁▁▁",
    "███████████▁▁▁▁▁▁▁▁▁",
    "█████████████▁▁▁▁▁▁▁",
    "██████████████▁▁▁▁▁▁",
    "██████████████▁▁▁▁▁▁",
    "▁██████████████▁▁▁▁▁",
    "▁██████████████▁▁▁▁▁",
    "▁██████████████▁▁▁▁▁",
    "▁▁██████████████▁▁▁▁",
    "▁▁▁██████████████▁▁▁",
    "▁▁▁▁█████████████▁▁▁",
    "▁▁▁▁██████████████▁▁",
    "▁▁▁▁██████████████▁▁",
    "▁▁▁▁▁██████████████▁",
    "▁▁▁▁▁██████████████▁",
    "▁▁▁▁▁██████████████▁",
    "▁▁▁▁▁▁██████████████",
    "▁▁▁▁▁▁██████████████",
    "▁▁▁▁▁▁▁█████████████",
    "▁▁▁▁▁▁▁█████████████",
    "▁▁▁▁▁▁▁▁████████████",
    "▁▁▁▁▁▁▁▁████████████",
    "▁▁▁▁▁▁▁▁▁███████████",
    "▁▁▁▁▁▁▁▁▁███████████",
    "▁▁▁▁▁▁▁▁▁▁██████████",
    "▁▁▁▁▁▁▁▁▁▁██████████",
    "▁▁▁▁▁▁▁▁▁▁▁▁████████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁███████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁██████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁█████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁█████",
    "█▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁████",
    "██▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁███",
    "██▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁███",
    "███▁▁▁▁▁▁▁▁▁▁▁▁▁▁███",
    "████▁▁▁▁▁▁▁▁▁▁▁▁▁▁██",
    "█████▁▁▁▁▁▁▁▁▁▁▁▁▁▁█",
    "█████▁▁▁▁▁▁▁▁▁▁▁▁▁▁█",
    "██████▁▁▁▁▁▁▁▁▁▁▁▁▁█",
    "████████▁▁▁▁▁▁▁▁▁▁▁▁",
    "█████████▁▁▁▁▁▁▁▁▁▁▁",
    "█████████▁▁▁▁▁▁▁▁▁▁▁",
    "█████████▁▁▁▁▁▁▁▁▁▁▁",
    "█████████▁▁▁▁▁▁▁▁▁▁▁",
    "███████████▁▁▁▁▁▁▁▁▁",
    "████████████▁▁▁▁▁▁▁▁",
    "████████████▁▁▁▁▁▁▁▁",
    "██████████████▁▁▁▁▁▁",
    "██████████████▁▁▁▁▁▁",
    "▁██████████████▁▁▁▁▁",
    "▁██████████████▁▁▁▁▁",
    "▁▁▁█████████████▁▁▁▁",
    "▁▁▁▁▁████████████▁▁▁",
    "▁▁▁▁▁████████████▁▁▁",
    "▁▁▁▁▁▁███████████▁▁▁",
    "▁▁▁▁▁▁▁▁█████████▁▁▁",
    "▁▁▁▁▁▁▁▁█████████▁▁▁",
    "▁▁▁▁▁▁▁▁▁█████████▁▁",
    "▁▁▁▁▁▁▁▁▁█████████▁▁",
    "▁▁▁▁▁▁▁▁▁▁█████████▁",
    "▁▁▁▁▁▁▁▁▁▁▁████████▁",
    "▁▁▁▁▁▁▁▁▁▁▁████████▁",
    "▁▁▁▁▁▁▁▁▁▁▁▁███████▁",
    "▁▁▁▁▁▁▁▁▁▁▁▁███████▁",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁███████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁███████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁█████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁████",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁███",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁███",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁██",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁██",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁██",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁█",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁█",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁█",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
    "▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁",
];

pub const PB_PATH_CUTOFF_LEN: usize = 32;

// Shamelessly stolen from `is-video` package from sindresorhus
// https://github.com/sindresorhus/is-video/blob/3ba58fa79b52949a0915e25f3fd2765b7a8a9809/index.js#L5
pub const VIDEO_EXTENSIONS: &[&str] = &[
    "3g2", "3gp", "aaf", "asf", "avchd", "avi", "drc", "flv", "m2v", "m3u8", "m4p", "m4v", "mkv",
    "mng", "mov", "mp2", "mp4", "mpe", "mpeg", "mpg", "mpv", "mxf", "nsv", "ogg", "ogv", "qt",
    "rm", "rmvb", "roq", "svi", "vob", "webm", "wmv", "yuv",
];
