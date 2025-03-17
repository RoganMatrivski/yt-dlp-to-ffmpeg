use color_eyre::eyre::Error;
use google_drive3::{
    hyper_rustls::HttpsConnector, hyper_util::client::legacy::connect::HttpConnector, DriveHub,
};

type Hub = DriveHub<HttpsConnector<HttpConnector>>;

pub async fn list_files(hub: &Hub, q: &str) -> Result<Vec<google_drive3::api::File>, Error> {
    let mut collected_files: Vec<google_drive3::api::File> = vec![];
    let mut next_page_token: Option<String> = None;

    loop {
        let mut req = hub.files().list();

        if let Some(token) = next_page_token {
            req = req.page_token(&token);
        }

        let (_, file_list) = req
            .page_size(1000i32)
            .q(q)
            .add_scope(google_drive3::api::Scope::Full)
            .supports_all_drives(true)
            .include_items_from_all_drives(true)
            .param(
                "fields",
                "files(id,name,md5Checksum,mimeType,size,createdTime,parents),nextPageToken",
            )
            .doit()
            .await?;

        if let Some(mut files) = file_list.files {
            collected_files.append(&mut files);
        }

        next_page_token = file_list.next_page_token;

        if next_page_token.is_none() {
            break;
        }
    }

    Ok(collected_files)
}
