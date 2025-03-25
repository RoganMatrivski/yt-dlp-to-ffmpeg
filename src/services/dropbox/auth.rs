use std::path::PathBuf;

use color_eyre::eyre::{bail, ContextCompat, Error};
use dropbox_sdk::{
    async_routes::{
        files::{list_folder, list_folder_continue},
        sharing::get_shared_link_metadata,
        users::get_current_account,
    },
    default_async_client::{NoauthDefaultClient, UserAuthDefaultClient},
    files::{ListFolderArg, ListFolderContinueArg, SharedLink},
    oauth2::{Authorization, AuthorizeUrlBuilder, Oauth2Type, PkceCode},
    sharing::GetSharedLinkMetadataArg,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DropboxCredentials {
    id: String,
}

fn get_token_path() -> std::path::PathBuf {
    crate::statics::PROJECT_DIR_PATH.join("dropbox-token.txt")
}

fn get_cred_path() -> std::path::PathBuf {
    crate::statics::PROJECT_DIR_PATH.join("dropbox-cred.json")
}

pub fn save_creds(id: &str) -> Result<(), Error> {
    let creds = DropboxCredentials { id: id.to_string() };

    std::fs::write(get_cred_path(), serde_json::to_string(&creds)?)?;

    Ok(())
}

pub fn load_creds() -> Result<DropboxCredentials, Error> {
    let creds_path = get_cred_path();
    if !creds_path.exists() {
        bail!(
            "Dropbox credentials not found Please authenticate first with auth-dropbox subcommand"
        );
    }

    let creds_str = std::fs::read_to_string(get_cred_path())?;
    Ok(serde_json::from_str(&creds_str).expect("Failed to load Dropbox credentials"))
}

pub fn save_token(state: &str) -> Result<(), Error> {
    std::fs::write(get_token_path(), state)?;

    Ok(())
}

pub fn load_token() -> Result<Option<String>, Error> {
    let token_path = get_token_path();
    if !token_path.exists() {
        return Ok(None);
    }

    let token_str = std::fs::read_to_string(token_path)?;
    Ok(Some(token_str))
}

fn present_user_url(url: &str) {
    println!();
    println!("Dropbox requires permissions to use Dropbox API.");
    println!("Open the url in your browser and follow the instructions:");
    println!("{}", url);
}

fn present_user_prompt(url: &str) -> String {
    r###"
Dropbox requires permissions to use Dropbox API.
Open the url in your browser and follow the instructions.
"###
    .to_string()
        + url
        + "\nPaste the code here"
}

pub async fn authenticate(client_id: &str) -> Result<Authorization, Error> {
    save_creds(client_id)?;

    let auth = if let Some(token) = load_token()? {
        Authorization::load(client_id.to_string(), &token)
    } else {
        None
    };

    let mut auth = if let Some(auth) = auth {
        auth
    } else {
        let oauth2_flow = Oauth2Type::PKCE(PkceCode::new());
        let url = AuthorizeUrlBuilder::new(client_id, &oauth2_flow).build();

        let authcode = dialoguer::Password::new()
            .with_prompt(present_user_prompt(url.as_str()))
            .interact()?;

        dbg!(&authcode);

        Authorization::from_auth_code(
            client_id.to_string(),
            oauth2_flow,
            authcode,
            Some(url.as_str().to_owned()),
        )
    };

    auth.obtain_access_token_async(NoauthDefaultClient::default())
        .await?;

    let auth_state = auth.save().wrap_err("Cannot authenticate")?;
    save_token(&auth_state)?;

    Ok(auth)
}

pub async fn get_async_client() -> Result<UserAuthDefaultClient, Error> {
    let cred = load_creds()?;
    Ok(UserAuthDefaultClient::new(authenticate(&cred.id).await?))
}
