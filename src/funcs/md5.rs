use color_eyre::eyre::Context;

use crate::{
    consts::{MAIN_BAR_CHARSET, SUB_BAR_FMT_MSG},
    statics::MPB,
};

use super::progressbar::get_progbar;

pub fn get_md5_from_path(path: &std::path::Path) -> Result<String, color_eyre::eyre::Report> {
    let file = std::fs::File::open(path)?;
    let pb = MPB.add(get_progbar(
        file.metadata()
            .wrap_err("Failed to get existing file size")?
            .len(),
        SUB_BAR_FMT_MSG,
        MAIN_BAR_CHARSET,
    )?);
    let file = pb.wrap_read(file);

    get_md5(file)
}

pub fn get_md5<T: std::io::Read>(reader: T) -> Result<String, color_eyre::eyre::Report> {
    let mut md5writer = crate::structs::Md5Writer::new(std::io::sink());
    let mut reader = reader;
    std::io::copy(&mut reader, &mut md5writer)?;
    Ok(md5writer.md5())
}
