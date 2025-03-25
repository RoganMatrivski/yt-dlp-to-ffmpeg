use color_eyre::eyre::Error;
use dropbox_sdk::{
    async_client_trait::HttpClient,
    async_routes::files::list_folder,
    default_async_client::NoauthDefaultClient,
    files::{ListFolderArg, SharedLink},
    oauth2::Authorization,
};

pub fn get_client() -> Result<(), Error> {
    let asdf = Authorization::load("asdf".to_string(), "asdf");

    let client = NoauthDefaultClient::default();

    Ok(())
}
