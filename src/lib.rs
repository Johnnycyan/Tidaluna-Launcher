pub mod constants;
pub mod tidal;
pub mod updater;

#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub mod progress;

#[cfg(windows)]
pub use windows::*;

pub async fn launch() {
    let Some(tidal) = tidal::get_tidal() else {
        let title = "No TIDAL installation found!";
        let message = "TidaLuna couldn't find your TIDAL installation.\n\
            Please install TIDAL from https://offer.tidal.com/download and try again.\n\n\
            Note: The Windows Store version of TIDAL is not supported.\n\
            Please install the desktop version.";

        #[cfg(not(windows))]
        {
            use dialog::DialogBox as _;
            let _ = dialog::Message::new(message).title(title).show();
        }

        #[cfg(windows)]
        messagebox(title, message, MessageBoxIcon::Error);

        return;
    };

    println!(
        "[TidaLuna Launcher] Found TIDAL at: {}",
        tidal.executable.display()
    );
    println!(
        "[TidaLuna Launcher] Resources directory: {}",
        tidal.resources_dir.display()
    );

    // Download/update luna.zip from GitHub releases
    let _ = updater::download_luna().await;

    // Extract luna.zip into TIDAL's resources directory
    match updater::extract_luna(&tidal.resources_dir) {
        Ok(()) => {
            println!("[TidaLuna Launcher] Luna injection complete.");
        }
        Err(e) => {
            eprintln!("[TidaLuna Launcher] Failed to extract Luna: {e}");

            let title = "TidaLuna Extraction Failed";
            let message = format!(
                "Failed to extract Luna into TIDAL:\n{e}\n\n\
                Make sure TIDAL is not running and try again."
            );

            #[cfg(not(windows))]
            {
                use dialog::DialogBox as _;
                let _ = dialog::Message::new(&message).title(title).show();
            }

            #[cfg(windows)]
            messagebox(title, &message, MessageBoxIcon::Error);

            return;
        }
    }

    // Launch TIDAL
    println!(
        "[TidaLuna Launcher] Launching TIDAL from: {}",
        tidal.executable.display()
    );

    let result = std::process::Command::new(&tidal.executable).spawn();

    match result {
        Ok(_) => {
            println!("[TidaLuna Launcher] TIDAL launched successfully.");
        }
        Err(e) => {
            eprintln!("[TidaLuna Launcher] Failed to launch TIDAL: {e}");

            let title = "Failed to launch TIDAL";
            let message = format!("Failed to launch TIDAL:\n{e}");

            #[cfg(not(windows))]
            {
                use dialog::DialogBox as _;
                let _ = dialog::Message::new(&message).title(title).show();
            }

            #[cfg(windows)]
            messagebox(title, &message, MessageBoxIcon::Error);
        }
    }
}
