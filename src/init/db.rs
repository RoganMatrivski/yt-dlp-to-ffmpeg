use clap::{crate_name, Subcommand};
use color_eyre::eyre::Error;

use libsql::Builder;

use crate::statics::PROJECT_DIR_PATH;
use std::fs;

#[derive(Subcommand, Clone)]
pub enum DbCommands {
    Init {
        #[clap(flatten)]
        remote_args: Option<RemoteDbCredentials>,
    },
    SyncRemote,
    // SyncLocal,
}

#[derive(Debug, Clone, clap::Args, serde::Deserialize, serde::Serialize)]
pub struct RemoteDbCredentials {
    token: String,
    url: String,
}

impl RemoteDbCredentials {
    pub fn new<T: ToString>(token: T, url: T) -> Self {
        Self {
            token: token.to_string(),
            url: url.to_string(),
        }
    }

    pub fn load_default() -> Result<Self, Error> {
        let cred_path = PROJECT_DIR_PATH.join("db_creds.json");
        let cred_data = fs::read_to_string(&cred_path)?;
        Ok(serde_json::from_str(&cred_data)?)
    }

    pub fn save(&self) -> Result<(), Error> {
        let cred_path = PROJECT_DIR_PATH.join("db_creds.json");
        let filestr = serde_json::to_string(self)?;
        fs::write(&cred_path, &filestr)?;
        Ok(())
    }
}

pub async fn handle_db_init(cmd: &Option<RemoteDbCredentials>) -> Result<(), Error> {
    let db_path = PROJECT_DIR_PATH.join("database.db");
    let db = match cmd {
        Some(RemoteDbCredentials { token, url }) => {
            Builder::new_remote_replica(&db_path, url.to_string(), token.to_string())
                .build()
                .await?
        }
        None => Builder::new_local(&db_path).build().await?,
    };

    let conn = db.connect()?;
    conn.query("select 1; select 1;", ()).await.unwrap();
    db.sync().await?;

    if let Some(creds) = cmd {
        creds.save()?;
    }

    Ok(())
}

pub async fn handle_db_commands(cmd: &DbCommands) -> Result<(), Error> {
    match cmd {
        DbCommands::Init { remote_args } => {
            handle_db_init(remote_args).await?;
        }
        DbCommands::SyncRemote => todo!(),
    }
    Ok(())
}
