use std::path::PathBuf;

/// Result of finding a TIDAL installation.
pub struct TidalPath {
    /// Path to the TIDAL executable.
    pub executable: PathBuf,
    /// Path to the resources directory (containing app.asar).
    pub resources_dir: PathBuf,
}

#[cfg(windows)]
pub fn get_tidal() -> Option<TidalPath> {
    use crate::windows::get_latest_executable;

    let local_appdata = dirs::data_local_dir()?;
    let tidal_dir = local_appdata.join("TIDAL");

    if !tidal_dir.join("Update.exe").exists() {
        return None;
    }

    let executable = get_latest_executable(&tidal_dir).ok()?;
    let resources_dir = executable.parent()?.join("resources");

    if !resources_dir.exists() {
        return None;
    }

    Some(TidalPath {
        executable,
        resources_dir,
    })
}

#[cfg(target_os = "linux")]
pub fn get_tidal() -> Option<TidalPath> {
    // tidal-hifi on Linux is typically installed to /opt/tidal-hifi/
    let paths_to_try = [
        PathBuf::from("/opt/tidal-hifi"),
        PathBuf::from("/usr/lib/tidal-hifi"),
    ];

    for base in &paths_to_try {
        let executable = base.join("tidal-hifi");
        let resources_dir = base.join("resources");

        if executable.is_file() && resources_dir.is_dir() {
            return Some(TidalPath {
                executable,
                resources_dir,
            });
        }
    }

    // Try checking PATH
    if let Ok(output) = std::process::Command::new("sh")
        .arg("-c")
        .arg("command -v tidal-hifi")
        .output()
    {
        if output.status.success() {
            let path = String::from_utf8(output.stdout).ok()?;
            let executable = PathBuf::from(path.trim());
            // Resolve symlinks to find the actual install directory
            let real_path = std::fs::canonicalize(&executable).ok()?;
            let resources_dir = real_path.parent()?.join("resources");
            if resources_dir.is_dir() {
                return Some(TidalPath {
                    executable,
                    resources_dir,
                });
            }
        }
    }

    None
}

#[cfg(target_os = "macos")]
pub fn get_tidal() -> Option<TidalPath> {
    let app_path = PathBuf::from("/Applications/TIDAL.app");
    let executable = app_path.join("Contents/MacOS/TIDAL");
    let resources_dir = app_path.join("Contents/Resources");

    if executable.is_file() && resources_dir.is_dir() {
        return Some(TidalPath {
            executable,
            resources_dir,
        });
    }

    None
}
