use std::path::Path;

use async_compat::CompatExt;
use color_eyre::eyre::{ContextCompat, Error};
use dropbox_sdk::{
    async_routes::sharing::get_shared_link_file, default_async_client::UserAuthDefaultClient,
    sharing::GetSharedLinkFileArg,
};
use tokio::fs::{self, File};

use crate::{consts, funcs::progressbar::get_progbar, statics::MPB};

pub async fn download_shared_file<U: ToString, T: AsRef<Path>>(
    client: &UserAuthDefaultClient,
    src: U,
    dst: T,
) -> Result<(), Error> {
    let dl_arg = GetSharedLinkFileArg::new(src.to_string());

    if let Some(parent) = dst.as_ref().parent() {
        fs::create_dir_all(parent).await?;
    }
    let mut file = File::create(&dst).await?;

    let res = get_shared_link_file(client, &dl_arg, None, None).await?;
    let res_body = res.body.wrap_err("Failed to get response body")?;

    let pb = MPB.add(get_progbar(
        res.content_length.unwrap(), // 'Unwrap' should be fine. If body is None, it'll throw early.
        consts::SUB_BAR_FMT_MSG,
        consts::MAIN_BAR_CHARSET,
    )?);

    let mut wrapped_body = pb.wrap_async_read(res_body.compat());

    tracing::trace!("Downloading to {}", dst.as_ref().to_string_lossy());

    tokio::io::copy(&mut wrapped_body, &mut file).await?;
    pb.finish_and_clear();

    Ok(())
}
