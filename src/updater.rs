use std::collections::HashMap;
use std::path::Path;

use tinyjson::JsonValue;

use crate::constants;

static USER_AGENT: &str = concat!("TidaLunaLauncher/", env!("CARGO_PKG_VERSION"));

#[allow(dead_code)]
struct GithubRelease {
    pub tag_name: String,
    pub name: String,
    pub updated_at: String,
}

pub async fn download_luna() -> Option<()> {
    let assets_dir = constants::asset_cache_dir().unwrap();
    let release_file = assets_dir.join(constants::RELEASE_INFO_FILE);
    let luna_zip_path = assets_dir.join(constants::LUNA_ZIP_FILENAME);

    // Get the current release.json if it exists.
    let current_version = if release_file.exists() {
        match std::fs::read_to_string(&release_file) {
            Ok(data) => {
                let json: JsonValue = match data.parse() {
                    Ok(j) => j,
                    Err(e) => {
                        eprintln!("[TidaLuna Launcher] Failed to parse release.json: {e:?}");
                        return None;
                    }
                };
                let object: &HashMap<_, _> = json.get()?;

                let tag_name: &String = object.get("tag_name")?.get()?;
                let name: &String = object.get("name")?.get()?;
                let updated_at: String = object
                    .get("updated_at")
                    .and_then(|v| v.get::<String>())
                    .cloned()
                    .unwrap_or_default();

                Some(GithubRelease {
                    tag_name: tag_name.clone(),
                    name: name.clone(),
                    updated_at,
                })
            }
            Err(e) => {
                eprintln!("[TidaLuna Launcher] Failed to read release.json: {e}");
                None
            }
        }
    } else {
        None
    };

    // Get the latest release manifest from GitHub.
    println!("[TidaLuna Launcher] Checking for updates...");
    let response = ureq::get(constants::RELEASE_URL)
        .header("User-Agent", USER_AGENT)
        .call();

    let mut response = match response {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("[TidaLuna Launcher] Failed to fetch release info from GitHub: {e}");
            eprintln!(
                "[TidaLuna Launcher] This may be due to rate limiting (60 requests/hour for unauthenticated requests)."
            );
            return None;
        }
    };

    let status = response.status();
    if status != 200 {
        eprintln!(
            "[TidaLuna Launcher] GitHub API returned non-200 status: {status} - updates may be rate-limited."
        );
        return None;
    }

    let body = match response.body_mut().read_to_string() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[TidaLuna Launcher] Failed to read response body: {e}");
            return None;
        }
    };

    let json: JsonValue = match body.parse() {
        Ok(j) => j,
        Err(e) => {
            eprintln!("[TidaLuna Launcher] Failed to parse GitHub API response: {e:?}");
            return None;
        }
    };
    let object: &HashMap<_, _> = json.get()?;

    let tag_name: &String = object.get("tag_name")?.get()?;
    let name: &String = object.get("name")?.get()?;
    let updated_at: &String = object.get("updated_at")?.get()?;

    // If the latest release has the same updated_at timestamp and the zip exists, skip download.
    if let Some(release) = current_version {
        if release.updated_at == *updated_at && luna_zip_path.exists() {
            println!("[TidaLuna Launcher] Already up to date (updated_at: {updated_at}).");
            return Some(());
        }
        println!(
            "[TidaLuna Launcher] Update detected: cached updated_at='{}' vs remote updated_at='{updated_at}'",
            release.updated_at
        );
    }

    println!("[TidaLuna Launcher] An update is available... Downloading luna.zip...");

    // Find the luna.zip asset in the release
    let assets: &Vec<_> = object.get("assets")?.get()?;
    let asset_url = assets.iter().find_map(|asset| {
        let asset: &HashMap<_, _> = asset.get()?;
        let asset_name: &String = asset.get("name")?.get()?;
        if asset_name == constants::LUNA_ZIP_FILENAME {
            let url: &String = asset.get("browser_download_url")?.get()?;
            Some(url.clone())
        } else {
            None
        }
    });

    let Some(asset_url) = asset_url else {
        eprintln!("[TidaLuna Launcher] Could not find 'luna.zip' asset in the release.");
        return None;
    };

    let response = ureq::get(&asset_url)
        .header("User-Agent", USER_AGENT)
        .call();

    let mut response = match response {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("[TidaLuna Launcher] Failed to download luna.zip: {e}");
            return None;
        }
    };

    let body = match response.body_mut().read_to_vec() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[TidaLuna Launcher] Failed to read luna.zip download: {e}");
            return None;
        }
    };

    println!(
        "[TidaLuna Launcher] Downloaded luna.zip ({} bytes)",
        body.len()
    );

    if let Err(e) = std::fs::write(&luna_zip_path, body) {
        eprintln!("[TidaLuna Launcher] Failed to write luna.zip to disk: {e}");
        return None;
    }

    // Write the new release.json to disk.
    let release_json = format!(
        "{{\n\
        \t\"tag_name\": \"{tag_name}\",\n\
        \t\"name\": \"{name}\",\n\
        \t\"updated_at\": \"{updated_at}\"\n\
		}}"
    );

    if let Err(e) = std::fs::write(&release_file, release_json) {
        eprintln!("[TidaLuna Launcher] Failed to write release.json: {e}");
        return None;
    }

    println!("[TidaLuna Launcher] Download complete.");
    Some(())
}

/// Extract the cached luna.zip into the TIDAL resources directory.
///
/// This performs the TidaLuna injection:
/// 1. Renames `app.asar` → `original.asar` (backup, if not already done)
/// 2. Removes any existing `app/` folder
/// 3. Extracts `luna.zip` contents into `resources/app/`
pub fn extract_luna(resources_dir: &Path) -> Result<(), String> {
    let assets_dir = constants::asset_cache_dir()
        .ok_or_else(|| "Failed to get asset cache directory".to_string())?;

    let luna_zip_path = assets_dir.join(constants::LUNA_ZIP_FILENAME);

    if !luna_zip_path.exists() {
        return Err("luna.zip not found in cache. Download may have failed.".to_string());
    }

    let app_asar = resources_dir.join("app.asar");
    let original_asar = resources_dir.join("original.asar");
    let app_dir = resources_dir.join("app");

    // Step 1: Backup app.asar → original.asar (if not already done)
    if app_asar.exists() && !original_asar.exists() {
        println!("[TidaLuna Launcher] Backing up app.asar → original.asar...");
        std::fs::rename(&app_asar, &original_asar)
            .map_err(|e| format!("Failed to rename app.asar to original.asar: {e}"))?;
    }

    // Step 2: Remove existing app/ folder if present
    if app_dir.exists() {
        println!("[TidaLuna Launcher] Removing existing app/ folder...");
        std::fs::remove_dir_all(&app_dir)
            .map_err(|e| format!("Failed to remove existing app/ folder: {e}"))?;
    }

    // Step 3: Extract luna.zip into resources/app/
    println!(
        "[TidaLuna Launcher] Extracting luna.zip to {}/app/ ...",
        resources_dir.display()
    );
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app/ directory: {e}"))?;

    let zip_file =
        std::fs::File::open(&luna_zip_path).map_err(|e| format!("Failed to open luna.zip: {e}"))?;

    let mut archive = zip::ZipArchive::new(zip_file)
        .map_err(|e| format!("Failed to read luna.zip archive: {e}"))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry {i}: {e}"))?;

        let outpath = app_dir.join(file.mangled_name());

        if file.name().ends_with('/') {
            // Directory entry
            std::fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory {}: {e}", outpath.display()))?;
        } else {
            // File entry
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        format!(
                            "Failed to create parent directory {}: {e}",
                            parent.display()
                        )
                    })?;
                }
            }

            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create file {}: {e}", outpath.display()))?;

            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file {}: {e}", outpath.display()))?;
        }
    }

    println!(
        "[TidaLuna Launcher] Successfully extracted {} files to app/",
        archive.len()
    );
    Ok(())
}
