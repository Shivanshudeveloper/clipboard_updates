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
  Zap
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
  
  // Auto-update states
  const [isAutoUpdating, setIsAutoUpdating] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState(null);
  const [currentVersion, setCurrentVersion] = useState("1.0.0");
  const [showUpdateNotification, setShowUpdateNotification] = useState(false);

  useEffect(() => {
    checkForUpdatesOnStart();
    setupEventListeners();
  }, []);

  const setupEventListeners = async () => {
    try {
      // Listen for update available events
      const unlistenUpdate = await listen('update-available', (event) => {
        console.log('Update available event received:', event.payload);
        
        // Auto-start update if available
        startAutoUpdate(event.payload);
      });

      // Listen for download progress
      const unlistenDownload = await listen('download-progress', (event) => {
        console.log('Download progress:', event.payload);
        setDownloadProgress(event.payload);
        
        if (event.payload.status === 'Completed') {
          // Show notification when download completes
          setShowUpdateNotification(true);
        } else if (event.payload.status === 'Failed') {
          setIsAutoUpdating(false);
          setShowUpdateNotification(false);
        }
      });

      return () => {
        unlistenUpdate();
        unlistenDownload();
      };
    } catch (error) {
      console.error('Failed to set up event listeners:', error);
    }
  };

  const checkForUpdatesOnStart = async () => {
    try {
      console.log('🔍 Performing automatic update check on startup...');
      const result = await invoke('check_for_updates');
      console.log('Update check result:', result);
      
      if (result.available) {
        // Auto-start update if available
        console.log('🚀 Auto-update enabled, starting update process...');
        startAutoUpdate(result);
      } else {
        console.log('✅ App is up to date');
      }
    } catch (error) {
      console.error('Silent update check failed:', error);
    }
  };

  const startAutoUpdate = async (updateInfo) => {
    if (!updateInfo) {
      console.log('No update info available for auto-update');
      return;
    }
    
    console.log('🚀 Starting automatic update process...');
    setIsAutoUpdating(true);
    setDownloadProgress(null);
    
    try {
      const updated = await invoke('auto_update');
      
      if (updated) {
        console.log('✅ Auto-update completed successfully');
        // Show notification
        setShowUpdateNotification(true);
      } else {
        console.log('ℹ️ No update was performed (already up to date)');
        setIsAutoUpdating(false);
      }
    } catch (error) {
      console.error('❌ Auto-update failed:', error);
      setIsAutoUpdating(false);
    }
  };

  const purgeAllUnpinned = () => {
    console.log("Purging all unpinned items");
    alert("All unpinned items have been purged.");
  };

  const formatFileSize = (bytes) => {
    if (!bytes) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

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

      {/* Auto-Update Notification Snackbar */}
      {showUpdateNotification && (
        <div className="mb-4 p-3 bg-green-50 border border-green-200 rounded-lg animate-in slide-in-from-top duration-300">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <CheckCircle size={16} className="text-green-600" />
              <div>
                <span className="text-sm font-medium text-green-800">
                  Update Ready to Install!
                </span>
                <p className="text-xs text-green-700">
                  The app will restart automatically to complete the update.
                </p>
              </div>
            </div>
            <button
              onClick={() => setShowUpdateNotification(false)}
              className="text-green-600 hover:text-green-800 transition-colors"
            >
              <X size={16} />
            </button>
          </div>
        </div>
      )}

      {/* Auto-Update in Progress Banner */}
      {isAutoUpdating && (
        <div className="mb-4 p-3 bg-purple-50 border border-purple-200 rounded-lg">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Zap size={16} className="text-purple-600" />
              <span className="text-sm font-medium text-purple-800">
                Auto-updating...
              </span>
            </div>
            <div className="text-xs text-purple-600">
              {downloadProgress ? `${downloadProgress.percentage.toFixed(1)}%` : 'Starting...'}
            </div>
          </div>
          {downloadProgress && (
            <>
              <div className="w-full bg-gray-200 rounded-full h-2 mt-2">
                <div 
                  className="bg-purple-600 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${downloadProgress.percentage}%` }}
                />
              </div>
              <div className="flex justify-between text-xs text-purple-700 mt-1">
                <span>
                  {formatFileSize(downloadProgress.downloaded_bytes)} / {formatFileSize(downloadProgress.total_bytes)}
                </span>
                <span>{formatSpeed(downloadProgress.speed)}</span>
              </div>
            </>
          )}
        </div>
      )}

      {/* Tabs */}
      <div className="flex gap-2 mb-1 flex-shrink-0">
        {["General", "Tags"].map((tab) => (
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
    </div>
  );
};

export default ClipTraySettings;