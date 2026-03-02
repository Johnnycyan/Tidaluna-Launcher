#![windows_subsystem = "windows"]

#[tokio::main]
async fn main() {
    tidaluna_launcher::launch().await;
}
