use async_compat::CompatExt;
use color_eyre::eyre::{ContextCompat, Error};
use futures_util::AsyncWriteExt;

pub fn setup_opendal(service_string: &str) -> Result<opendal::Operator, Error> {
    let [service, id, key, bucket, bucket_id, rootpath] =
        service_string.split(';').collect::<Vec<_>>()[0..6]
    else {
        color_eyre::eyre::bail!("Error parsing service string")
    };

    let builder = match service {
        "B2" | "b2" => {
            opendal::services::B2::default()
                // set the storage bucket for OpenDAL
                .root(rootpath)
                // set the application_key_id for OpenDAL
                .application_key_id(id)
                // set the application_key for OpenDAL
                .application_key(key)
                // set the bucket name for OpenDAL
                .bucket(bucket)
                .bucket_id(bucket_id)
        }
        others => unimplemented!("{others} service is not implemented"),
    };

    Ok(opendal::Operator::new(builder)?
        .layer(opendal::layers::BlockingLayer::create()?)
        .layer(
            opendal::layers::RetryLayer::new()
                .with_factor(2.0)
                .with_max_times(128),
        )
        .finish())
}

pub async fn copy_path_to_b2(path: &std::path::Path, op: &opendal::Operator) -> Result<(), Error> {
    let file = tokio::fs::File::open(path).await?;
    let filename = path
        .file_name()
        .wrap_err("Invalid filename")?
        .to_string_lossy();

    let output_filesize = file.metadata().await?.len();

    let pb = crate::statics::MPB.add(crate::funcs::progressbar::get_progbar(
        output_filesize,
        crate::consts::SUB_BAR_FMT_MSG,
        crate::consts::MAIN_BAR_CHARSET,
    )?);

    let mut wrapped_file = pb.wrap_async_read(file);

    let mut writer = op
        .writer_with(&filename)
        .append(false)
        .await?
        .into_futures_async_write();

    tokio::io::copy(&mut wrapped_file, &mut writer.compat_mut()).await?;

    writer.close().await?;

    Ok(())
}

pub async fn check_path_exists(
    path: &std::path::Path,
    op: &opendal::Operator,
) -> Result<bool, Error> {
    let filename = path
        .file_name()
        .wrap_err("Invalid filename")?
        .to_string_lossy();

    let urlencoded_filename = urlencoding::encode(&filename)
        .replace("%27", "'")
        .replace("%28", "(")
        .replace("%29", ")");

    tracing::trace!("Checking if {urlencoded_filename} exists...");

    Ok(op.exists(&urlencoded_filename).await?)
}
