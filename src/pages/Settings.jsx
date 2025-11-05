import React, { useState, useEffect } from "react";
import GeneralSettings from "../components/settings/General";
import TagsSettings from "../components/settings/Tags";
import { 
  ArrowLeft, 
  Download, 
  RefreshCw, 
  CheckCircle, 
  AlertCircle,
  X,
  Play
} from "lucide-react";
import { Link } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const ClipTraySettings = () => {
  const [tags, setTags] = useState(["keys", "mcp", "prompts", "todo"]);
  const [newTag, setNewTag] = useState("");
  const [activeTab, setActiveTab] = useState("General");
  const [startAtLogin, setStartAtLogin] = useState(true);
  const [autoPurgeUnpinned, setAutoPurgeUnpinned] = useState(true);
  const [purgeCadence, setPurgeCadence] = useState("Every 24 hours");
  
  // Update states
  const [updateInfo, setUpdateInfo] = useState(null);
  const [isChecking, setIsChecking] = useState(false);
  const [isInstalling, setIsInstalling] = useState(false);
  const [currentVersion, setCurrentVersion] = useState("0.1.6");
  
  // Download states
  const [downloadProgress, setDownloadProgress] = useState(null);
  const [installerInfo, setInstallerInfo] = useState(null);
  const [isDownloading, setIsDownloading] = useState(false);

  useEffect(() => {
    // Check for updates on component mount
    checkForUpdatesOnStart();
    
    // Listen for update available events from backend
    const setupUpdateListener = async () => {
      try {
        const unlisten = await listen('update-available', (event) => {
          console.log('Update available event received:', event.payload);
          setUpdateInfo(event.payload);
        });
        return unlisten;
      } catch (error) {
        console.error('Failed to set up update listener:', error);
      }
    };

    // Listen for download progress events
    const setupDownloadListener = async () => {
      try {
        const unlisten = await listen('download-progress', (event) => {
          console.log('Download progress:', event.payload);
          setDownloadProgress(event.payload);
          
          if (event.payload.status === 'Completed') {
            setIsDownloading(false);
            // Auto-install after download completes
            if (installerInfo) {
              installDownloadedUpdate();
            }
          } else if (event.payload.status === 'Failed') {
            setIsDownloading(false);
            alert(`Download failed: ${event.payload.error}`);
          }
        });
        return unlisten;
      } catch (error) {
        console.error('Failed to set up download listener:', error);
      }
    };

    const unlistenPromise1 = setupUpdateListener();
    const unlistenPromise2 = setupDownloadListener();
    
    return () => {
      unlistenPromise1.then(unlisten => {
        if (unlisten) {
          unlisten();
        }
      });
      unlistenPromise2.then(unlisten => {
        if (unlisten) {
          unlisten();
        }
      });
    };
  }, [installerInfo]);

  const checkForUpdatesOnStart = async () => {
    try {
      // Check for updates silently on startup
      const result = await invoke('check_for_updates');
      console.log('Update check result:', result);
      if (result.available) {
        setUpdateInfo(result);
      }
    } catch (error) {
      console.error('Silent update check failed:', error);
    }
  };

  const checkForUpdates = async () => {
    setIsChecking(true);
    try {
      const result = await invoke('check_for_updates');
      setUpdateInfo(result);
      
      if (!result.available) {
        alert('🎉 You are running the latest version!');
      }
    } catch (error) {
      console.error('Update check failed:', error);
      alert(`Update check failed: ${error.message || error}`);
    } finally {
      setIsChecking(false);
    }
  };

  const downloadAndInstallUpdate = async () => {
    if (!updateInfo?.download_url && !updateInfo?.release_url) {
      alert('No download URL available. Please visit the GitHub releases page manually.');
      return;
    }

    setIsDownloading(true);
    setDownloadProgress(null);
    setInstallerInfo(null);
    
    try {
      const downloadUrl = updateInfo.download_url || updateInfo.release_url;
      console.log('Starting download:', downloadUrl);
      
      const result = await invoke('download_update', { downloadUrl });
      console.log('Download completed:', result);
      setInstallerInfo(result);
      
      // The installation will be triggered automatically by the download progress listener
      // when status becomes 'Completed'
      
    } catch (error) {
      console.error('Download failed:', error);
      alert(`Download failed: ${error.message || error}`);
      setIsDownloading(false);
      
      // Fallback to simple download
      const downloadUrl = updateInfo.download_url || updateInfo.release_url;
      if (downloadUrl) {
        window.open(downloadUrl, '_blank');
        alert('Opening download page in your browser as fallback...');
      }
    }
  };

  const installDownloadedUpdate = async () => {
    if (!installerInfo) {
      console.error('No installer info available');
      return;
    }

    setIsInstalling(true);
    try {
      console.log('Installing update:', installerInfo);
      await invoke('install_downloaded_update', { installerInfo });
      alert('Installation started! The application will restart automatically.');
    } catch (error) {
      console.error('Installation failed:', error);
      alert(`Installation failed: ${error.message || error}. Please run the installer manually.`);
    } finally {
      setIsInstalling(false);
    }
  };

  const cancelDownload = async () => {
    try {
      await invoke('cancel_update');
      setIsDownloading(false);
      setDownloadProgress(null);
      setInstallerInfo(null);
    } catch (error) {
      console.error('Cancel failed:', error);
    }
  };

  // Simple download (fallback)
  const simpleDownloadUpdate = async () => {
    if (!updateInfo?.download_url && !updateInfo?.release_url) {
      alert('No download URL available.');
      return;
    }

    setIsInstalling(true);
    try {
      const downloadUrl = updateInfo.download_url || updateInfo.release_url;
      await invoke('install_update', { downloadUrl });
      window.open(downloadUrl, '_blank');
    } catch (error) {
      console.error('Simple download failed:', error);
      const downloadUrl = updateInfo.download_url || updateInfo.release_url;
      if (downloadUrl) {
        window.open(downloadUrl, '_blank');
        alert('Opening download page in your browser...');
      }
    } finally {
      setIsInstalling(false);
    }
  };

  const purgeAllUnpinned = () => {
    console.log("Purging all unpinned items");
    alert("All unpinned items have been purged.");
  };

  // Format release notes for display
  const formatReleaseNotes = (notes) => {
    if (!notes) return '';
    return notes.substring(0, 200).replace(/[#*`]/g, '') + (notes.length > 200 ? '...' : '');
  };

  // Format file size for display
  const formatFileSize = (bytes) => {
    if (!bytes) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  // Format download speed for display
  const formatSpeed = (bytesPerSecond) => {
    if (!bytesPerSecond) return '0 B/s';
    const k = 1024;
    const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
    const i = Math.floor(Math.log(bytesPerSecond) / Math.log(k));
    return parseFloat((bytesPerSecond / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  return (
    <div className="w-auto h-[565px] bg-gray-50 border border-gray-300 rounded-xl shadow-sm p-5 font-sans flex flex-col overflow-hidden">
      <Link to="/home">
        <button className="flex items-center gap-2 text-gray-700 hover:text-black transition mt-3">
          <ArrowLeft size={20} />
          <span>Back</span>
        </button>
      </Link>
      
      <div className="flex justify-between items-center mb-3">
        <h2 className="text-base font-semibold text-gray-800">ClipTray Settings</h2>
        <span className="text-xs text-gray-500 bg-gray-200 px-2 py-1 rounded">
          v{currentVersion}
        </span>
      </div>

      {/* Download Progress Banner */}
      {isDownloading && downloadProgress && (
        <div className="mb-4 p-3 bg-blue-50 border border-blue-200 rounded-lg">
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <Download size={16} className="text-blue-600" />
              <span className="text-sm font-medium text-blue-800">
                Downloading Update...
              </span>
            </div>
            <button
              onClick={cancelDownload}
              className="text-blue-600 hover:text-blue-800"
              title="Cancel Download"
            >
              <X size={16} />
            </button>
          </div>
          
          {/* Progress Bar */}
          <div className="w-full bg-gray-200 rounded-full h-2 mb-2">
            <div 
              className="bg-blue-600 h-2 rounded-full transition-all duration-300"
              style={{ width: `${downloadProgress.percentage}%` }}
            />
          </div>
          
          <div className="flex justify-between text-xs text-blue-700">
            <span>
              {formatFileSize(downloadProgress.downloaded_bytes)} / {formatFileSize(downloadProgress.total_bytes)}
            </span>
            <span>{downloadProgress.percentage.toFixed(1)}%</span>
            <span>{formatSpeed(downloadProgress.speed)}</span>
          </div>
        </div>
      )}

      {/* Downloaded Ready to Install */}
      {installerInfo && !isDownloading && (
        <div className="mb-4 p-3 bg-green-50 border border-green-200 rounded-lg">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <CheckCircle size={16} className="text-green-600" />
              <div>
                <span className="text-sm font-medium text-green-800">
                  Update Downloaded!
                </span>
                <p className="text-xs text-green-700">
                  {installerInfo.file_name} ({formatFileSize(installerInfo.file_size)})
                </p>
              </div>
            </div>
            <button
              onClick={installDownloadedUpdate}
              disabled={isInstalling}
              className="flex items-center gap-1 px-3 py-1 bg-green-600 text-white text-sm rounded hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {isInstalling ? (
                <RefreshCw size={14} className="animate-spin" />
              ) : (
                <Play size={14} />
              )}
              {isInstalling ? 'Installing...' : 'Install Now'}
            </button>
          </div>
        </div>
      )}

      {/* Update Available Banner (when not downloading) */}
      {updateInfo?.available && !isDownloading && !installerInfo && (
        <div className="mb-4 p-3 bg-blue-50 border border-blue-200 rounded-lg">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Download size={16} className="text-blue-600" />
              <div>
                <span className="text-sm font-medium text-blue-800">
                  Update Available: v{updateInfo.latest_version}
                </span>
                {updateInfo.release_notes && (
                  <p className="text-xs text-blue-700 mt-1">
                    {formatReleaseNotes(updateInfo.release_notes)}
                  </p>
                )}
              </div>
            </div>
            <div className="flex gap-2">
              <button
                onClick={downloadAndInstallUpdate}
                className="flex items-center gap-1 px-3 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 transition-colors"
                title="Download and install automatically"
              >
                <Download size={14} />
                Auto Install
              </button>
              <button
                onClick={simpleDownloadUpdate}
                className="flex items-center gap-1 px-3 py-1 bg-gray-600 text-white text-sm rounded hover:bg-gray-700 transition-colors"
                title="Download manually in browser"
              >
                <Download size={14} />
                Manual
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Tabs */}
      <div className="flex gap-2 mb-1 flex-shrink-0">
        {["General", "Tags", "Updates"].map((tab) => (
          <button
            key={tab}
            className={`px-3.5 py-1.5 text-sm border border-gray-300 rounded-lg cursor-pointer transition-all duration-200 ${
              activeTab === tab
                ? "bg-blue-500 text-white border-blue-500 shadow-sm shadow-blue-500/30"
                : "bg-gray-200 hover:bg-gray-300"
            }`}
            onClick={() => setActiveTab(tab)}
          >
            {tab}
          </button>
        ))}
      </div>

      <hr className="border-none border-t border-gray-300 my-2.5 mb-3.5" />

      {activeTab === "General" && (
        <GeneralSettings
          startAtLogin={startAtLogin}
          setStartAtLogin={setStartAtLogin}
          autoPurgeUnpinned={autoPurgeUnpinned}
          setAutoPurgeUnpinned={setAutoPurgeUnpinned}
          purgeCadence={purgeCadence}
          setPurgeCadence={setPurgeCadence}
          purgeAllUnpinned={purgeAllUnpinned}
        />
      )}

      {activeTab === "Tags" && (
        <TagsSettings
          tags={tags}
          setTags={setTags}
          newTag={newTag}
          setNewTag={setNewTag}
        />
      )}

      {activeTab === "Updates" && (
        <div className="flex-1 overflow-y-auto">
          <div className="space-y-4">
            {/* Current Version */}
            <div className="bg-white p-4 rounded-lg border border-gray-200">
              <h3 className="text-sm font-medium text-gray-800 mb-2">Current Version</h3>
              <div className="flex items-center gap-2">
                <CheckCircle size={16} className="text-green-500" />
                <span className="text-sm text-gray-700">v{currentVersion}</span>
              </div>
            </div>

            {/* Update Status */}
            <div className="bg-white p-4 rounded-lg border border-gray-200">
              <h3 className="text-sm font-medium text-gray-800 mb-3">Update Status</h3>
              
              {updateInfo ? (
                <div className="space-y-2">
                  {updateInfo.available ? (
                    <div className="space-y-2">
                      <div className="flex items-center gap-2 text-green-600">
                        <Download size={16} />
                        <span className="text-sm font-medium">
                          Update available: v{updateInfo.latest_version}
                        </span>
                      </div>
                      {updateInfo.release_notes && (
                        <div className="bg-gray-50 p-3 rounded border">
                          <p className="text-xs text-gray-700">
                            {formatReleaseNotes(updateInfo.release_notes)}
                          </p>
                        </div>
                      )}
                      {updateInfo.error && (
                        <div className="flex items-center gap-2 text-red-600">
                          <AlertCircle size={14} />
                          <span className="text-xs">{updateInfo.error}</span>
                        </div>
                      )}
                    </div>
                  ) : (
                    <div className="flex items-center gap-2 text-gray-600">
                      <CheckCircle size={16} />
                      <span className="text-sm">You're running the latest version!</span>
                    </div>
                  )}
                </div>
              ) : (
                <div className="flex items-center gap-2 text-gray-500">
                  <AlertCircle size={16} />
                  <span className="text-sm">Check for updates to see current status</span>
                </div>
              )}
            </div>

            {/* Download Progress Section */}
            {(isDownloading || installerInfo) && (
              <div className="bg-white p-4 rounded-lg border border-gray-200">
                <h3 className="text-sm font-medium text-gray-800 mb-3">
                  {isDownloading ? 'Download Progress' : 'Ready to Install'}
                </h3>
                
                {isDownloading && downloadProgress && (
                  <div className="space-y-3">
                    <div>
                      <div className="flex justify-between text-sm text-gray-600 mb-1">
                        <span>Progress</span>
                        <span>{downloadProgress.percentage.toFixed(1)}%</span>
                      </div>
                      <div className="w-full bg-gray-200 rounded-full h-3">
                        <div 
                          className="bg-blue-600 h-3 rounded-full transition-all duration-300"
                          style={{ width: `${downloadProgress.percentage}%` }}
                        />
                      </div>
                    </div>
                    
                    <div className="grid grid-cols-3 gap-4 text-xs text-gray-600">
                      <div>
                        <div className="font-medium">Downloaded</div>
                        <div>{formatFileSize(downloadProgress.downloaded_bytes)}</div>
                      </div>
                      <div>
                        <div className="font-medium">Total</div>
                        <div>{formatFileSize(downloadProgress.total_bytes)}</div>
                      </div>
                      <div>
                        <div className="font-medium">Speed</div>
                        <div>{formatSpeed(downloadProgress.speed)}</div>
                      </div>
                    </div>
                    
                    <button
                      onClick={cancelDownload}
                      className="w-full px-4 py-2 bg-red-600 text-white text-sm rounded-lg hover:bg-red-700 transition-colors"
                    >
                      Cancel Download
                    </button>
                  </div>
                )}
                
                {installerInfo && !isDownloading && (
                  <div className="space-y-3">
                    <div className="flex items-center gap-2 text-green-600">
                      <CheckCircle size={16} />
                      <span className="text-sm">Update successfully downloaded</span>
                    </div>
                    
                    <div className="bg-gray-50 p-3 rounded border">
                      <div className="text-sm text-gray-700">
                        <div><strong>File:</strong> {installerInfo.file_name}</div>
                        <div><strong>Size:</strong> {formatFileSize(installerInfo.file_size)}</div>
                        <div><strong>Location:</strong> {installerInfo.file_path}</div>
                      </div>
                    </div>
                    
                    <div className="flex gap-2">
                      <button
                        onClick={installDownloadedUpdate}
                        disabled={isInstalling}
                        className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-green-600 text-white text-sm rounded-lg hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                      >
                        {isInstalling ? (
                          <>
                            <RefreshCw size={16} className="animate-spin" />
                            Installing...
                          </>
                        ) : (
                          <>
                            <Play size={16} />
                            Install Now
                          </>
                        )}
                      </button>
                    </div>
                  </div>
                )}
              </div>
            )}

            {/* Update Actions */}
            <div className="bg-white p-4 rounded-lg border border-gray-200">
              <h3 className="text-sm font-medium text-gray-800 mb-3">Update Actions</h3>
              <div className="space-y-2">
                <button
                  onClick={checkForUpdates}
                  disabled={isChecking}
                  className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-blue-600 text-white text-sm rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  {isChecking ? (
                    <>
                      <RefreshCw size={16} className="animate-spin" />
                      Checking for Updates...
                    </>
                  ) : (
                    <>
                      <RefreshCw size={16} />
                      Check for Updates
                    </>
                  )}
                </button>

                {updateInfo?.available && !isDownloading && !installerInfo && (
                  <>
                    <button
                      onClick={downloadAndInstallUpdate}
                      className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-green-600 text-white text-sm rounded-lg hover:bg-green-700 transition-colors"
                    >
                      <Download size={16} />
                      Auto Download & Install v{updateInfo.latest_version}
                    </button>
                    
                    <button
                      onClick={simpleDownloadUpdate}
                      className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-gray-600 text-white text-sm rounded-lg hover:bg-gray-700 transition-colors"
                    >
                      <Download size={16} />
                      Manual Download in Browser
                    </button>
                  </>
                )}
              </div>
              
              {/* Manual Download Fallback */}
              {updateInfo?.available && (
                <div className="mt-3 p-3 bg-yellow-50 border border-yellow-200 rounded">
                  <p className="text-xs text-yellow-800">
                    <strong>Manual Download:</strong> If automatic installation fails, 
                    you can download manually from GitHub.
                  </p>
                  <button
                    onClick={() => window.open(updateInfo.release_url, '_blank')}
                    className="mt-2 text-xs text-yellow-800 underline hover:text-yellow-900"
                  >
                    Open GitHub Releases
                  </button>
                </div>
              )}
            </div>

            {/* Update Information */}
            <div className="bg-white p-4 rounded-lg border border-gray-200">
              <h3 className="text-sm font-medium text-gray-800 mb-2">How Updates Work</h3>
              <ul className="text-xs text-gray-600 space-y-1">
                <li>• <strong>Auto Install:</strong> Downloads and installs automatically with progress</li>
                <li>• <strong>Manual Download:</strong> Opens GitHub releases in your browser</li>
                <li>• Updates are fetched from GitHub releases</li>
                <li>• Your data and settings are preserved during updates</li>
              </ul>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default ClipTraySettings;
