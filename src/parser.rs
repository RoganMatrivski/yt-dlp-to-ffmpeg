use std::str::FromStr;

use nom::character::complete::multispace0;
use nom::combinator::opt;
use nom::{
    bytes::complete::is_not,
    character::complete::char,
    sequence::{delimited, pair, preceded},
    IResult,
};
use strum::EnumString;

// List of websites that accepts a URL string
#[derive(Debug, PartialEq, EnumString)]
pub enum DlTypes {
    #[strum(ascii_case_insensitive, serialize = "yt-dlp", serialize = "yt-dl")]
    YtDlp,
    #[strum(ascii_case_insensitive, serialize = "direct", serialize = "dl")]
    DirectLink,
    #[strum(
        ascii_case_insensitive,
        serialize = "google-drive",
        serialize = "gdrive"
    )]
    GoogleDrive,
    #[strum(ascii_case_insensitive, serialize = "dropbox")]
    Dropbox,
}

fn nom_parse_line(line: &str) -> IResult<&str, (&str, &str)> {
    // Parse lines like this:
    // #[(TYPE)]: (url)
    // where TYPE is a alphanumeric string
    preceded(
        char('#'),
        pair(
            delimited(char('['), is_not("]"), char(']')),
            preceded(
                multispace0,
                preceded(opt(char(':')), preceded(multispace0, is_not("\n"))),
            ),
        ),
    )(line)
}

pub fn line_filter(line: &&str) -> bool {
    !line.is_empty() && !line.starts_with(r#"//"#)
}

pub fn parse_line(line: &str) -> Result<(DlTypes, &str), color_eyre::eyre::Report> {
    if !line.starts_with("#[") {
        return Ok((DlTypes::YtDlp, line));
    }

    let (ty, url) = nom_parse_line(line)
        .map_err(|e| color_eyre::eyre::eyre!(e.to_owned()))?
        .1;

    Ok((DlTypes::from_str(ty)?, url))
}
