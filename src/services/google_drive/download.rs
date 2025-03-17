use std::{path::PathBuf, sync::Arc};

use color_eyre::eyre::{ContextCompat, Error};
use google_drive3::{
    hyper::body::Bytes, hyper_rustls::HttpsConnector,
    hyper_util::client::legacy::connect::HttpConnector, DriveHub,
};

use http_body_util::combinators::BoxBody;
use indicatif::ProgressIterator;

use crate::{
    consts,
    funcs::{ffmpeg::ffmpeg_transcode, opendal::copy_path_to_b2, progressbar::get_progbar},
    init::DownloadOpts,
};

type Hub = DriveHub<HttpsConnector<HttpConnector>>;

pub async fn get_body_from_id(
    hub: &Hub,
    file_id: &str,
) -> Result<BoxBody<Bytes, google_drive3::hyper::Error>, Error> {
    let (response, _) = hub
        .files()
        .get(file_id)
        .supports_all_drives(true)
        .param("alt", "media")
        .add_scope(google_drive3::api::Scope::Full)
        .doit()
        .await?;

    Ok(response.into_body())
}

pub async fn handle_google_drive(
    args: &DownloadOpts,
    i: Option<usize>,
    file_id: &str,
    op: Option<opendal::Operator>,
) -> Result<(), color_eyre::eyre::Report> {
    let hub = super::auth::get_hub(None).await?;

    let nodes = super::node::fetch_nodes(&hub, file_id, Arc::new(None)).await?;
    let items = nodes.get_tuples();

    let output_dir = match args.target_dir {
        Some(ref x) => x,
        None => &std::env::current_dir()?,
    };

    let total_gdrive_pb = if items.len() > 1 {
        crate::statics::MPB.add(get_progbar(
            items.len() as u64,
            consts::MAIN_BAR_FMT,
            consts::MAIN_BAR_CHARSET,
        )?)
    } else {
        indicatif::ProgressBar::hidden()
    };

    total_gdrive_pb.set_message("Google Drive Items");

    for super::node::FileInfo { id, path, md5 } in items.iter().progress_with(total_gdrive_pb) {
        let file_path: PathBuf = output_dir.join(path);

        let body = get_body_from_id(&hub, id).await?;
        super::save_body_to_file(body, &file_path, md5.clone()).await?;

        let output_path_stem = file_path
            .file_stem()
            .wrap_err("Cannot get file stem")?
            .to_string_lossy();
        let output_path_ext = file_path
            .extension()
            .unwrap_or(std::ffi::OsStr::new("mp4"))
            .to_string_lossy();

        let out_name = format!(
            "{idxstr}{output_path_stem}.{output_path_ext}",
            idxstr = if let Some(i) = i {
                format!("{i:05}_")
            } else {
                "".to_string()
            }
        );

        let final_output_path = file_path.with_file_name(out_name);

        let encode_output_path = file_path
            .with_file_name((output_path_stem.clone() + "_temp." + output_path_ext).as_ref());

        ffmpeg_transcode(
            &file_path.to_string_lossy(),
            &encode_output_path,
            format!("{output_path_stem}").as_str(),
        )?;

        tracing::trace!("Verifying {output_path_stem}...");
        crate::funcs::ffmpeg::ffmpeg_check(&encode_output_path)?;
        tracing::trace!("Verified {output_path_stem}...");

        // OpenDAL doesn't support checksumming yet
        tracing::info!(
            "MD5: {}",
            crate::funcs::md5::get_md5_from_path(&encode_output_path)?
        );

        std::fs::remove_file(&file_path)?;
        std::fs::rename(&encode_output_path, &final_output_path)?;

        tracing::info!(
            "MD5: {}",
            crate::funcs::md5::get_md5_from_path(&final_output_path)?
        );

        if let Some(op) = &op {
            copy_path_to_b2(&final_output_path, op).await?;
            std::fs::remove_file(&final_output_path)?;
        };
    }

    Ok(())
}
