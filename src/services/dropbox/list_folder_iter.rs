use dropbox_sdk::{
    async_routes::files::list_folder_continue,
    default_async_client::UserAuthDefaultClient,
    files::{ListFolderContinueArg, ListFolderResult, Metadata},
};

pub struct ListFolderIter<'a> {
    pub entries: Vec<Metadata>,
    pub cursor: String,
    pub has_more: bool,

    pub client: &'a UserAuthDefaultClient,

    pub entry_iter: std::vec::IntoIter<Metadata>,
}

impl<'a> ListFolderIter<'a> {
    pub fn new(client: &'a UserAuthDefaultClient, ls_res: ListFolderResult) -> Self {
        Self {
            entries: ls_res.entries.clone(),
            cursor: ls_res.cursor,
            has_more: ls_res.has_more,

            client: client.clone(),

            entry_iter: ls_res.entries.into_iter(),
        }
    }
}

// impl<'a> Iterator for ListFolderIter<'a> {
//     type Item = Metadata;

//     fn next(&mut self) -> Option<Self::Item> {
//         if let Some(x) = self.entry_iter.next() {
//             return Some(x);
//         }

//         if self.has_more {
//             let ls_cont_arg = ListFolderContinueArg::new(self.cursor);
//             list_folder_continue(self.client, &ls_cont_arg)
//         }
//     }
// }

// impl IntoIterator for ListFolderIter {
//     type Item = Metadata;

//     type IntoIter = std::vec::IntoIter<Self::Item>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.entry_iter
//     }
// }
