import React, { useState, useEffect } from "react";
import GeneralSettings from "../components/settings/General";
import TagsSettings from "../components/settings/Tags";
import { ArrowLeft, Download, RefreshCw, CheckCircle, AlertCircle } from "lucide-react";
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
  const [currentVersion, setCurrentVersion] = useState("");

  useEffect(() => {
    // Get current app version on component mount
    getCurrentVersion();
    
    // Listen for update available events
    const setupListener = async () => {
      try {
        const unlisten = await listen('update-available', (event) => {
          setUpdateInfo(event.payload);
        });
        
        // Store unlisten function for cleanup
        return unlisten;
      } catch (error) {
        console.error('Failed to set up update listener:', error);
      }
    };
    
    const unlistenPromise = setupListener();
    
    return () => {
      unlistenPromise.then(unlisten => {
        if (unlisten) {
          unlisten();
        }
      });
    };
  }, []);

  const getCurrentVersion = async () => {
    try {
      const version = await invoke('get_app_version');
      setCurrentVersion(version);
    } catch (error) {
      console.error('Failed to get current version:', error);
    }
  };

  const checkForUpdates = async () => {
    setIsChecking(true);
    try {
      const result = await invoke('check_for_updates');
      setUpdateInfo(result);
    } catch (error) {
      console.error('Update check failed:', error);
      alert(`Update check failed: ${error}`);
    } finally {
      setIsChecking(false);
    }
  };

  const installUpdate = async () => {
    setIsInstalling(true);
    try {
      const result = await invoke('install_update');
      if (result.success) {
        alert('Update installed successfully! The application will restart momentarily.');
      }
    } catch (error) {
      console.error('Update installation failed:', error);
      alert(`Update installation failed: ${error}`);
      setIsInstalling(false);
    }
  };

  const purgeAllUnpinned = () => {
    console.log("Purging all unpinned items");
    alert("All unpinned items have been purged.");
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
        {currentVersion && (
          <span className="text-xs text-gray-500 bg-gray-200 px-2 py-1 rounded">
            v{currentVersion}
          </span>
        )}
      </div>

      {/* Update Available Banner */}
      {updateInfo?.available && (
        <div className="mb-4 p-3 bg-blue-50 border border-blue-200 rounded-lg">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Download size={16} className="text-blue-600" />
              <span className="text-sm font-medium text-blue-800">
                Update Available: v{updateInfo.latest_version}
              </span>
            </div>
            <button
              onClick={installUpdate}
              disabled={isInstalling}
              className="flex items-center gap-1 px-3 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isInstalling ? (
                <RefreshCw size={14} className="animate-spin" />
              ) : (
                <Download size={14} />
              )}
              {isInstalling ? 'Installing...' : 'Install Now'}
            </button>
          </div>
          {updateInfo.body && (
            <p className="text-xs text-blue-700 mt-1">{updateInfo.body}</p>
          )}
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
                <span className="text-sm text-gray-700">v{currentVersion || 'Loading...'}</span>
              </div>
            </div>

            {/* Update Status */}
            <div className="bg-white p-4 rounded-lg border border-gray-200">
              <h3 className="text-sm font-medium text-gray-800 mb-3">Update Status</h3>
              
              {updateInfo ? (
                <div className="space-y-2">
                  {updateInfo.available ? (
                    <div className="flex items-center gap-2 text-green-600">
                      <Download size={16} />
                      <span className="text-sm">
                        Update available: v{updateInfo.latest_version}
                      </span>
                    </div>
                  ) : (
                    <div className="flex items-center gap-2 text-gray-600">
                      <CheckCircle size={16} />
                      <span className="text-sm">{updateInfo.message}</span>
                    </div>
                  )}
                  
                  {updateInfo.body && (
                    <p className="text-xs text-gray-600 mt-2">{updateInfo.body}</p>
                  )}
                </div>
              ) : (
                <div className="flex items-center gap-2 text-gray-500">
                  <AlertCircle size={16} />
                  <span className="text-sm">No update check performed yet</span>
                </div>
              )}
            </div>

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

                {updateInfo?.available && (
                  <button
                    onClick={installUpdate}
                    disabled={isInstalling}
                    className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-green-600 text-white text-sm rounded-lg hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                  >
                    {isInstalling ? (
                      <>
                        <RefreshCw size={16} className="animate-spin" />
                        Installing Update...
                      </>
                    ) : (
                      <>
                        <Download size={16} />
                        Install v{updateInfo.latest_version}
                      </>
                    )}
                  </button>
                )}
              </div>
            </div>

           
          </div>
        </div>
      )}
    </div>
  );
};

export default ClipTraySettings;