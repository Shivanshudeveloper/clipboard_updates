use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use reqwest;
use semver::Version;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;
use futures_util::StreamExt; // Add this import

// Move these structs to the top level so they can be imported
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateCheckResult {
    pub available: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_notes: String,
    pub download_url: String,
    pub release_url: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub html_url: String,
    pub assets: Vec<ReleaseAsset>,
    pub prerelease: bool,
    pub draft: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadProgress {
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub percentage: f64,
    pub speed: f64,
    pub status: DownloadStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DownloadStatus {
    Starting,
    Downloading,
    Completed,
    Failed(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstallerInfo {
    pub file_path: String,
    pub file_name: String,
    pub file_size: u64,
}

pub struct Updater {
    owner: String,
    repo: String,
    current_version: String,
    temp_dir: Option<TempDir>,
}

impl Updater {
    pub fn new(owner: &str, repo: &str, current_version: &str) -> Self {
        Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
            current_version: current_version.to_string(),
            temp_dir: None,
        }
    }

    pub async fn check_for_updates(&self) -> UpdateCheckResult {
        match self.fetch_latest_release().await {
            Ok(latest_release) => {
                if self.is_newer_version(&latest_release.tag_name) {
                    let asset = self.find_appropriate_asset(&latest_release.assets);
                    
                    UpdateCheckResult {
                        available: true,
                        current_version: self.current_version.clone(),
                        latest_version: latest_release.tag_name.clone(),
                        release_notes: latest_release.body,
                        download_url: asset.map(|a| a.browser_download_url.clone()).unwrap_or_default(),
                        release_url: latest_release.html_url,
                        error: None,
                    }
                } else {
                    UpdateCheckResult {
                        available: false,
                        current_version: self.current_version.clone(),
                        latest_version: latest_release.tag_name,
                        release_notes: String::new(),
                        download_url: String::new(),
                        release_url: String::new(),
                        error: None,
                    }
                }
            }
            Err(e) => UpdateCheckResult {
                available: false,
                current_version: self.current_version.clone(),
                latest_version: String::new(),
                release_notes: String::new(),
                download_url: String::new(),
                release_url: String::new(),
                error: Some(e.to_string()),
            },
        }
    }

    async fn fetch_latest_release(&self) -> Result<ReleaseInfo, Box<dyn std::error::Error>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.owner, self.repo
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", "ClipTray-Updater")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("GitHub API returned status: {}", response.status()).into());
        }

        let release: ReleaseInfo = response.json().await?;
        Ok(release)
    }

    fn is_newer_version(&self, latest_tag: &str) -> bool {
        // Clean version strings (remove 'v' prefix)
        let current_clean = self.current_version.trim_start_matches('v');
        let latest_clean = latest_tag.trim_start_matches('v');

        match (Version::parse(current_clean), Version::parse(latest_clean)) {
            (Ok(current), Ok(latest)) => latest > current,
            _ => {
                // Fallback: string comparison if semver parsing fails
                latest_clean != current_clean
            }
        }
    }

    fn find_appropriate_asset<'a>(&self, assets: &'a [ReleaseAsset]) -> Option<&'a ReleaseAsset> {
        // Look for appropriate installer assets
        let patterns = [
            ".msi", // Windows installer
            ".exe", // Windows executable
            ".dmg", // macOS
            ".AppImage", // Linux
            ".deb", // Debian package
        ];

        assets.iter().find(|asset| {
            patterns.iter().any(|pattern| asset.name.contains(pattern))
        })
    }

    pub async fn download_update(
        &mut self,
        download_url: String,
        app_handle: AppHandle,
    ) -> Result<InstallerInfo, String> {
        // Create temporary directory for download
        let temp_dir = TempDir::new().map_err(|e| format!("Failed to create temp dir: {}", e))?;
        self.temp_dir = Some(temp_dir);
        
        let file_name = Self::extract_filename_from_url(&download_url)
            .unwrap_or_else(|| "update_installer".to_string());
        let file_path = self.temp_dir.as_ref().unwrap().path().join(&file_name);
        
        println!("Downloading update from: {}", download_url);
        println!("Saving to: {:?}", file_path);

        let client = reqwest::Client::new();
        let response = client
            .get(&download_url)
            .header("User-Agent", "ClipTray-Updater")
            .send()
            .await
            .map_err(|e| format!("Failed to start download: {}", e))?;

        let total_size = response
            .content_length()
            .ok_or("Failed to get content length")?;

        // Emit download started event
        let _ = app_handle.emit(
            "download-progress",
            DownloadProgress {
                total_bytes: total_size,
                downloaded_bytes: 0,
                percentage: 0.0,
                speed: 0.0,
                status: DownloadStatus::Starting,
            },
        );

        let mut file = File::create(&file_path)
            .map_err(|e| format!("Failed to create file: {}", e))?;
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();
        let start_time = std::time::Instant::now();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
            file.write_all(&chunk)
                .map_err(|e| format!("Write error: {}", e))?;
            
            downloaded += chunk.len() as u64;
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 {
                downloaded as f64 / elapsed
            } else {
                0.0
            };
            let percentage = (downloaded as f64 / total_size as f64) * 100.0;

            // Emit progress update
            let _ = app_handle.emit(
                "download-progress",
                DownloadProgress {
                    total_bytes: total_size,
                    downloaded_bytes: downloaded,
                    percentage,
                    speed,
                    status: DownloadStatus::Downloading,
                },
            );
        }

        // Verify download completed
        if downloaded != total_size {
            return Err(format!("Download incomplete: {}/{} bytes", downloaded, total_size));
        }

        // Emit download completed event
        let _ = app_handle.emit(
            "download-progress",
            DownloadProgress {
                total_bytes: total_size,
                downloaded_bytes: downloaded,
                percentage: 100.0,
                speed: 0.0,
                status: DownloadStatus::Completed,
            },
        );

        Ok(InstallerInfo {
            file_path: file_path.to_string_lossy().to_string(),
            file_name,
            file_size: total_size,
        })
    }

    pub async fn install_downloaded_update(&self, installer_info: InstallerInfo) -> Result<(), String> {
        // For now, just open the installer - in production you'd want proper installation logic
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new("cmd")
                .args(&["/c", "start", "", &installer_info.file_path])
                .spawn()
                .map_err(|e| format!("Failed to start installer: {}", e))?;
        }
        
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("open")
                .arg(&installer_info.file_path)
                .spawn()
                .map_err(|e| format!("Failed to open installer: {}", e))?;
        }
        
        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            Command::new("xdg-open")
                .arg(&installer_info.file_path)
                .spawn()
                .map_err(|e| format!("Failed to open installer: {}", e))?;
        }
        
        Ok(())
    }

  pub async fn download_and_install(&self, download_url: String, app_handle: AppHandle) -> Result<(), String> {
    // For Tauri 2.0, use the webview to open the URL
    if let Some(window) = app_handle.get_webview_window("main") {
        let js = format!("window.open('{}', '_blank');", download_url);
        window.eval(&js)
            .map_err(|e| format!("Failed to open download URL: {}", e))?;
        Ok(())
    } else {
        // Fallback: use system command to open URL
        #[cfg(target_os = "windows")]
        let command = "cmd";
        #[cfg(target_os = "windows")]
        let args = ["/c", "start", ""];
        
        #[cfg(target_os = "macos")]
        let command = "open";
        #[cfg(target_os = "macos")]
        let args: [&str; 0] = [];
        
        #[cfg(target_os = "linux")]
        let command = "xdg-open";
        #[cfg(target_os = "linux")]
        let args: [&str; 0] = [];

        let status = std::process::Command::new(command)
            .args(&args)
            .arg(&download_url)
            .status()
            .map_err(|e| format!("Failed to open URL: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err("Failed to open download URL".to_string())
        }
    }
}

    fn extract_filename_from_url(url: &str) -> Option<String> {
        url.split('/').last().map(|s| s.to_string())
    }

    pub fn cleanup(&mut self) {
        if let Some(temp_dir) = self.temp_dir.take() {
            let _ = temp_dir.close();
        }
    }
}

impl Drop for Updater {
    fn drop(&mut self) {
        self.cleanup();
    }
}