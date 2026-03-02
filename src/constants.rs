pub static RELEASE_URL: &str = "https://api.github.com/repos/Inrixia/TidaLuna/releases/latest";
pub static LUNA_ZIP_FILENAME: &str = "luna.zip";
pub static RELEASE_INFO_FILE: &str = "release.json";

pub fn asset_cache_dir() -> Option<std::path::PathBuf> {
    let local_appdata = dirs::data_local_dir()?;

    let dir = local_appdata.join("TidaLunaLauncher").join("cache");

    if !dir.exists() {
        std::fs::create_dir_all(&dir).ok()?;
    }

    Some(dir)
}
