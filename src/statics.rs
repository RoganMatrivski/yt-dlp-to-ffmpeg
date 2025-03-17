use std::sync::LazyLock;

pub static MPB: LazyLock<indicatif::MultiProgress> = LazyLock::new(indicatif::MultiProgress::new);
pub static PROJECT_DIR_PATH: LazyLock<std::path::PathBuf> = LazyLock::new(|| {
    let dirpath = match directories::ProjectDirs::from(
        crate::consts::APP_ID[0],
        crate::consts::APP_ID[1],
        std::env!("CARGO_PKG_NAME"),
    ) {
        Some(x) => x.data_dir().to_path_buf(),
        None => std::env::current_dir().expect("Failed to get current directory"),
    };

    std::fs::create_dir_all(&dirpath).unwrap();

    dirpath
});
