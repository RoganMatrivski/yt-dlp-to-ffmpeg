use std::{path::PathBuf, sync::Arc};

use color_eyre::eyre::{ContextCompat, Error};
use futures_util::future::try_join_all;
use google_drive3::{
    hyper_rustls::HttpsConnector, hyper_util::client::legacy::connect::HttpConnector, DriveHub,
};

pub const MIME_TYPE_DRIVE_FOLDER: &str = "application/vnd.google-apps.folder";
pub const MIME_TYPE_DRIVE_SHORTCUT: &str = "application/vnd.google-apps.shortcut";

pub fn is_directory(file: &google_drive3::api::File) -> bool {
    file.mime_type == Some(String::from(MIME_TYPE_DRIVE_FOLDER))
}

#[allow(dead_code)]
pub fn is_binary(file: &google_drive3::api::File) -> bool {
    file.md5_checksum != None
}

pub fn is_shortcut(file: &google_drive3::api::File) -> bool {
    file.mime_type == Some(String::from(MIME_TYPE_DRIVE_SHORTCUT))
}

type Hub = DriveHub<HttpsConnector<HttpConnector>>;

#[derive(Clone)]
pub enum DriveNode {
    Folder {
        name: String,
        parent: Arc<Option<DriveNode>>,
        child: Vec<DriveNode>,
    },
    File {
        id: String,
        name: String,
        parent: Arc<Option<DriveNode>>,
        file_info: google_drive3::api::File,
    },
}

pub struct FileInfo {
    pub id: String,
    pub path: PathBuf,
    pub md5: Option<String>,
}

impl DriveNode {
    // pub fn iterate_nodes(&self) -> Result(String, PathBuf)
    // pub fn join_parent(&self) -> Result<PathBuf, Error> {
    //     if let Some(p) = &self.get_parent()? {
    //         Ok(PathBuf::new())
    //     } else {
    //         Ok(PathBuf::new())
    //     }
    // }

    // pub fn get_path(&self) -> PathBuf {
    //     match self {
    //         DriveNode::Folder { name, parent, .. } => Self::build_path(parent.as_ref(), name),
    //         DriveNode::File { name, parent, .. } => Self::build_path(parent.as_ref(), name),
    //     }
    // }

    pub fn get_parent(&self) -> Option<DriveNode> {
        match self {
            DriveNode::Folder { parent, .. } => parent.as_ref().clone(),
            DriveNode::File { parent, .. } => parent.as_ref().clone(),
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            DriveNode::Folder { name, .. } => name.clone(),
            DriveNode::File { name, .. } => name.clone(),
        }
    }

    pub fn build_path(&self) -> PathBuf {
        if let Some(p) = &self.get_parent() {
            p.build_path().join(self.get_name())
        } else {
            PathBuf::from(self.get_name())
        }
    }

    pub fn get_tuples(&self) -> Vec<FileInfo> {
        match &self {
            DriveNode::Folder { child, .. } => child
                .iter()
                .flat_map(|x| x.get_tuples())
                .collect::<Vec<_>>(),
            DriveNode::File { id, file_info, .. } => vec![FileInfo {
                id: id.to_string(),
                path: self.build_path(),
                md5: file_info.md5_checksum.clone(),
            }],
        }
    }
}

#[async_recursion::async_recursion]
pub async fn fetch_nodes<S: AsRef<str> + std::marker::Send>(
    hub: &Hub,
    id: S,
    parent_node: Arc<Option<DriveNode>>,
) -> Result<DriveNode, Error> {
    let (_, metadata) = hub
        .files()
        .get(id.as_ref())
        .supports_all_drives(true)
        .param("fields", "id, name, mimeType")
        .add_scope(google_drive3::api::Scope::Full)
        .doit()
        .await?;

    if is_shortcut(&metadata) {
        let target_file_id = metadata
            .shortcut_details
            .and_then(|details| details.target_id);

        fetch_nodes(hub, target_file_id.unwrap_or_default(), parent_node).await
    } else if is_directory(&metadata) {
        let child_nodes = super::list_files(
            hub,
            format!("'{}' in parents and trashed = false", id.as_ref()).as_str(),
        )
        .await?;

        let asdf = child_nodes
            .iter()
            .filter_map(|x| x.id.clone())
            .map(|x| (x.clone(), parent_node.clone()))
            .map(|x| fetch_nodes(hub, x.0, x.1))
            .collect::<Vec<_>>();

        let childs = try_join_all(asdf).await?;

        Ok(DriveNode::Folder {
            name: metadata.name.wrap_err("Can't get folder name")?,
            parent: parent_node.clone(),
            child: childs,
        })
        // let newnode = Self::Folder()
    } else {
        Ok(DriveNode::File {
            id: id.as_ref().to_string(),
            name: metadata.name.clone().wrap_err("Can't get file name")?,
            parent: parent_node.clone(),
            file_info: metadata,
        })
    }
}
