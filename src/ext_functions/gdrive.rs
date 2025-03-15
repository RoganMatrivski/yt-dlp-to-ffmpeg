use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use color_eyre::eyre::{Context, ContextCompat, Error};
use google_drive3::{
    hyper::body::{Body, Bytes},
    hyper_rustls::HttpsConnector,
    hyper_util::client::legacy::connect::HttpConnector,
    DriveHub,
};

use futures_util::StreamExt;
use http_body_util::{combinators::BoxBody, BodyExt};

type Hub = DriveHub<HttpsConnector<HttpConnector>>;

pub async fn download_regular(
    hub: &Hub,
    file_id: &str,
    root_path: &Path,
    file: &google_drive3::api::File,
) -> Result<PathBuf, Error> {
    let body = download_file(&hub, file_id).await?;

    let file_name = file.name.clone().wrap_err("Cannot get file name")?;
    let root_path = root_path;
    let abs_file_path = root_path.join(&file_name);

    tracing::info!("Downloading {}", file_name);
    save_body_to_file(body, &abs_file_path, file.md5_checksum.clone()).await?;
    tracing::info!("Successfully downloaded {}", file_name);

    Ok(abs_file_path)
}

pub async fn download_file(
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

pub async fn save_body_to_file(
    body: BoxBody<Bytes, google_drive3::hyper::Error>,
    file_path: &PathBuf,
    expected_md5: Option<String>,
) -> Result<(), Error> {
    if file_path.exists() {
        tracing::info!("Existing file detected. Checking MD5...");

        let existing_file = File::open(&file_path)?;

        if let Some(expected_md5) = &expected_md5 {
            let pb = crate::MPB.add(crate::funcs::get_progbar(
                existing_file
                    .metadata()
                    .wrap_err("Failed to get existing file size")?
                    .len(),
                crate::consts::SUB_BAR_FMT_MSG,
                crate::consts::MAIN_BAR_CHARSET,
            )?);

            let mut existing_file = pb.wrap_read(existing_file);
            let mut md5writer = crate::ext_functions::md5writer::Md5Writer::new(std::io::sink());

            std::io::copy(&mut existing_file, &mut md5writer)?;
            pb.finish_and_clear();

            let actual_md5 = md5writer.md5();
            if actual_md5 == *expected_md5 {
                tracing::info!(
                    "File already downloaded and verified: {}",
                    file_path.display()
                );
                return Ok(());
            } else {
                tracing::info!(
                    "MD5 mismatch for existing file, re-downloading: {}",
                    file_path.display()
                );
            }
        } else {
            tracing::info!(
                "File already exists but no MD5 to verify: {}",
                file_path.display()
            );
        }
    }

    let pb = crate::MPB.add(crate::funcs::get_progbar(
        body.size_hint().upper().unwrap_or(0),
        crate::consts::SUB_BAR_FMT_MSG,
        crate::consts::MAIN_BAR_CHARSET,
    )?);

    // Create temporary file
    let tmp_file_path = file_path.with_extension("incomplete");
    let file = File::create(&tmp_file_path)?;

    let file = pb.wrap_write(file);
    let mut file = crate::ext_functions::md5writer::Md5Writer::new(file);

    let mut bodystream = body.into_data_stream();

    // Read chunks from stream and write to file
    while let Some(chunk) = bodystream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
    }

    if let Some(expected_md5) = expected_md5 {
        let actual_md5 = file.md5();
        if actual_md5 != expected_md5 {
            color_eyre::eyre::bail!(
                "MD5 mismatch: expected {}, got {}",
                expected_md5,
                actual_md5
            );
        }
    };

    pb.finish_and_clear();

    // Rename temporary file to final file
    std::fs::rename(&tmp_file_path, &file_path).wrap_err("Cannot rename temporary file to final")
}
