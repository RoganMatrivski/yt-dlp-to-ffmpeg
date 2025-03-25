use std::path::PathBuf;

use color_eyre::eyre::Error;
use dropbox_sdk::{
    async_routes::{
        files::{list_folder, list_folder_continue},
        sharing::get_shared_link_metadata,
    },
    default_async_client::UserAuthDefaultClient,
    files::{ListFolderArg, ListFolderContinueArg, SharedLink},
    sharing::GetSharedLinkMetadataArg,
};

#[async_recursion::async_recursion]
pub async fn walk_shared_link(
    client: &UserAuthDefaultClient,
    shared_link: &str,
    path: Option<PathBuf>,
) -> Result<Vec<(String, PathBuf)>, Error> {
    use dropbox_sdk::files::Metadata;
    use dropbox_sdk::sharing::SharedLinkMetadata;

    let is_empty_path = path.is_none();
    let root_path = path.unwrap_or(PathBuf::new());
    let m_args = if !is_empty_path {
        GetSharedLinkMetadataArg::new(shared_link.to_string())
            .with_path(root_path.to_string_lossy().into_owned())
    } else {
        GetSharedLinkMetadataArg::new(shared_link.to_string())
    };

    let m = get_shared_link_metadata(client, &m_args).await?;

    let results = match m {
        SharedLinkMetadata::File(f) => vec![(f.url, root_path)],
        SharedLinkMetadata::Folder(f) => {
            let slink = SharedLink::new(f.url);
            let ls_arg = ListFolderArg::new("".to_string()).with_shared_link(slink);
            let ls = list_folder(client, &ls_arg).await?;

            fn process_ls(ls: Vec<Metadata>) -> Vec<String> {
                ls.into_iter()
                    .map(|x| match x {
                        Metadata::File(f) => f.name,
                        Metadata::Folder(f) => f.name,
                        _ => unimplemented!(),
                    })
                    .collect::<Vec<_>>()
            }

            let mut entries = process_ls(ls.entries);
            let mut has_more = ls.has_more;
            let mut cursor = ls.cursor;

            while has_more {
                let ls_cont_arg = ListFolderContinueArg::new(cursor);
                let ls = list_folder_continue(client, &ls_cont_arg).await?;

                let mut new_entries = process_ls(ls.entries);
                entries.append(&mut new_entries);
                has_more = ls.has_more;
                cursor = ls.cursor;
            }

            let mut urls = vec![];
            for name in entries {
                let new_path = if root_path.has_root() {
                    root_path.join(name)
                } else {
                    root_path.join(format!("/{name}"))
                };

                let mut new_urls = walk_shared_link(client, shared_link, Some(new_path)).await?;
                urls.append(&mut new_urls);
            }

            urls
        }
        _ => unimplemented!(),
    };

    Ok(results)
}
