extern crate google_drive3 as drive3;

use drive3::{hyper_rustls, hyper_util, yup_oauth2, DriveHub};
use google_drive3::{
    hyper_rustls::HttpsConnector, hyper_util::client::legacy::connect::HttpConnector,
    yup_oauth2::authenticator::Authenticator,
};
use serde::{Deserialize, Serialize};

use color_eyre::Report;

#[derive(Serialize, Deserialize)]
pub struct GdriveCredentials {
    pub id: String,
    pub secret: String,
}

struct AuthDelegate;

impl yup_oauth2::authenticator_delegate::InstalledFlowDelegate for AuthDelegate {
    fn present_user_url<'a>(
        &'a self,
        url: &'a str,
        _need_code: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send + 'a>>
    {
        Box::pin(present_user_url(url))
    }
}

async fn present_user_url(url: &str) -> Result<String, String> {
    println!();
    println!();
    println!("Gdrive requires permissions to manage your files on Google Drive.");
    println!("Open the url in your browser and follow the instructions:");
    println!("{}", url);
    Ok(String::new())
}

fn get_token_path() -> std::path::PathBuf {
    crate::statics::PROJECT_DIR_PATH.join("gdrive-token.json")
}

fn get_cred_path() -> std::path::PathBuf {
    crate::statics::PROJECT_DIR_PATH.join("gdrive-cred.json")
}

pub fn save_creds(id: &str, secret: &str) -> Result<(), Report> {
    let creds = GdriveCredentials {
        id: id.to_string(),
        secret: secret.to_string(),
    };

    std::fs::write(get_cred_path(), serde_json::to_string(&creds)?)?;

    Ok(())
}

pub fn load_creds() -> Result<GdriveCredentials, Report> {
    let creds_path = get_cred_path();
    if !creds_path.exists() {
        return Err(Report::msg(
            "Gdrive credentials not found Please authenticate first with auth-gdrive subcommand",
        ));
    }

    let creds_str = std::fs::read_to_string(get_cred_path())?;
    Ok(serde_json::from_str(&creds_str).expect("Failed to load Gdrive credentials"))
}

pub async fn get_auth(
    creds: Option<GdriveCredentials>,
) -> Result<Authenticator<HttpsConnector<HttpConnector>>, Report> {
    let GdriveCredentials { id, secret } = creds.unwrap_or_else(|| load_creds().unwrap());

    tracing::info!("Authenticating with Gdrive...");
    let auth_secret = yup_oauth2::ApplicationSecret {
        client_id: id,
        client_secret: secret,
        token_uri: String::from("https://oauth2.googleapis.com/token"),
        auth_uri: String::from("https://accounts.google.com/o/oauth2/auth"),
        redirect_uris: vec![String::from("urn:ietf:wg:oauth:2.0:oob")],
        project_id: None,
        client_email: None,
        auth_provider_x509_cert_url: Some(String::from(
            "https://www.googleapis.com/oauth2/v1/certs",
        )),
        client_x509_cert_url: None,
    };

    yup_oauth2::InstalledFlowAuthenticator::builder(
        auth_secret,
        yup_oauth2::InstalledFlowReturnMethod::HTTPPortRedirect(8420),
    )
    .persist_tokens_to_disk(get_token_path())
    .flow_delegate(Box::new(AuthDelegate))
    .build()
    .await
    .map_err(|e| Report::msg(e.to_string()))
}

pub async fn get_hub(
    auth: Option<Authenticator<HttpsConnector<HttpConnector>>>,
) -> Result<DriveHub<HttpsConnector<HttpConnector>>, Report> {
    let auth = match auth {
        Some(auth) => auth,
        None => get_auth(None).await?,
    };

    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()?
                .https_or_http()
                .enable_http1()
                .enable_http2()
                .build(),
        );

    let hub = DriveHub::new(client, auth);

    Ok(hub)
}

pub async fn authenticate(id: &str, secret: &str) -> Result<(), color_eyre::eyre::Report> {
    save_creds(id, secret)?;

    let creds = load_creds()?;
    let auth = get_auth(Some(creds)).await?;
    let hub = get_hub(Some(auth)).await?;

    let about = hub.about().get().param("fields", "user").doit().await?;
    tracing::info!("Drive Hub Functional: {:?}", about);

    Ok(())
}
