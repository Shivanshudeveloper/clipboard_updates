import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

const GeneralSettings = ({ 
  startAtLogin, 
  setStartAtLogin, 
  autoPurgeUnpinned, 
  setAutoPurgeUnpinned,
  purgeCadence,
  setPurgeCadence
}) => {
  const [availablePurgeOptions, setAvailablePurgeOptions] = useState([]);
  const [isLoading, setIsLoading] = useState(false);

  // Load purge settings when component mounts
  useEffect(() => {
    loadPurgeSettings();
  }, []);

  // Load available purge options and current settings
  const loadPurgeSettings = async () => {
    try {
      console.log("📋 Loading purge settings...");
      
      // Get available purge cadence options
      const options = await invoke('get_purge_cadence_options');
      setAvailablePurgeOptions(options);
      console.log("✅ Available purge options:", options);
      
      // Get current user settings
      const settings = await invoke('get_current_purge_settings');
      console.log("✅ Current purge settings:", settings);
      
      // Update local state with actual settings from backend
      setAutoPurgeUnpinned(settings.auto_purge_enabled);
      setPurgeCadence(settings.purge_cadence);
      
    } catch (error) {
      console.error('❌ Failed to load purge settings:', error);
    }
  };

  // Handle auto purge toggle
  const handleAutoPurgeToggle = async (enabled) => {
    try {
      setIsLoading(true);
      console.log("🔄 Toggling auto purge:", enabled);
      
      // Determine the new cadence - if disabling, set to "Never", otherwise keep current
      const newCadence = enabled ? purgeCadence : 'Never';
      
      // Update backend
      await invoke('update_auto_purge_settings', {
        autoPurgeUnpinned: enabled,
        purgeCadence: newCadence
      });
      
      // Update local state
      setAutoPurgeUnpinned(enabled);
      if (!enabled) {
        setPurgeCadence('Never');
      }
      
      console.log("✅ Auto purge settings updated successfully");
      
    } catch (error) {
      console.error('❌ Failed to update auto purge settings:', error);
      alert('Failed to update settings: ' + error);
      // Revert local state on error
      setAutoPurgeUnpinned(!enabled);
    } finally {
      setIsLoading(false);
    }
  };

  // Handle purge cadence change
  const handlePurgeCadenceChange = async (newCadence) => {
    try {
      setIsLoading(true);
      console.log("🔄 Changing purge cadence to:", newCadence);
      
      // Update backend
      await invoke('update_purge_cadence', {
        purgeCadence: newCadence
      });
      
      // Update local state
      setPurgeCadence(newCadence);
      
      // If selecting "Never", also update auto purge toggle
      if (newCadence === 'Never') {
        setAutoPurgeUnpinned(false);
      } else {
        setAutoPurgeUnpinned(true);
      }
      
      console.log("✅ Purge cadence updated successfully");
      
    } catch (error) {
      console.error('❌ Failed to update purge cadence:', error);
      alert('Failed to update purge cadence: ' + error);
      // Revert local state on error
      setPurgeCadence(purgeCadence);
    } finally {
      setIsLoading(false);
    }
  };

  // Purge all unpinned entries (keep this same as before)
  const purgeAllUnpinned = async () => {
    try {
      console.log("🗑️ Purging all unpinned entries...");
      const result = await invoke('purge_unpinned_entries');
      console.log(`✅ Purged ${result} unpinned entries`);
      
      // Show success message
      // alert(`Successfully purged ${result} unpinned entries`);
      
      // Refresh the entries list if you have that function
      // if (window.refreshEntries) {
      //   window.refreshEntries();
      // }
      
    } catch (error) {
      console.error('❌ Failed to purge entries:', error);
      alert('Failed to purge entries: ' + error);
    }
  };

  return (
    <div className="space-y-6">
      {/* Startup Section */}
      <div className="space-y-3">
        <h3 className="text-sm font-medium text-gray-700">Startup</h3>
        
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="startAtLogin"
            checked={startAtLogin}
            onChange={(e) => setStartAtLogin(e.target.checked)}
            className="w-4 h-4 text-blue-500 rounded focus:ring-blue-400"
            disabled={isLoading}
          />
          <label htmlFor="startAtLogin" className="text-sm text-gray-700">
            Start ClipTray at login
          </label>
        </div>
        
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="autoPurgeUnpinned"
            checked={autoPurgeUnpinned}
            onChange={(e) => handleAutoPurgeToggle(e.target.checked)}
            className="w-4 h-4 text-blue-500 rounded focus:ring-blue-400"
            disabled={isLoading}
          />
          <label htmlFor="autoPurgeUnpinned" className="text-sm text-gray-700">
            Auto-purge unpinned items
          </label>
          {isLoading && <span className="text-xs text-gray-500">Updating...</span>}
        </div>
      </div>

      {/* Purge Cadence Section */}
      {autoPurgeUnpinned && (
        <div className="space-y-2">
          <h3 className="text-sm font-medium text-gray-700">Purge cadence</h3>
          <div className="space-y-1.5">
            {availablePurgeOptions.map((option) => (
              <div key={option} className="flex items-center gap-2">
                <input
                  type="radio"
                  id={option}
                  name="purgeCadence"
                  checked={purgeCadence === option}
                  onChange={() => handlePurgeCadenceChange(option)}
                  className="w-4 h-4 text-blue-500 focus:ring-blue-400"
                  disabled={isLoading}
                />
                <label htmlFor={option} className="text-sm text-gray-700">
                  {option}
                </label>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Purge Action Buttons */}
      <div className="pt-4 space-y-3">
        <button
          onClick={purgeAllUnpinned}
          className="px-4 py-2 text-sm border border-red-300 rounded-lg bg-white text-red-700 cursor-pointer transition-colors duration-200 hover:bg-red-50 disabled:opacity-50 disabled:cursor-not-allowed"
          disabled={isLoading}
        >
          🗑️ Purge all unpinned now
        </button>
        
        {/* Debug info - remove in production */}
        {/* <div className="p-2 text-xs bg-gray-100 rounded">
          <div>Current Cadence: {purgeCadence}</div>
          <div>Auto Purge: {autoPurgeUnpinned ? 'Enabled' : 'Disabled'}</div>
          <div>Options: {availablePurgeOptions.join(', ')}</div>
        </div> */}
      </div>
    </div>
  );
};

export default GeneralSettings;