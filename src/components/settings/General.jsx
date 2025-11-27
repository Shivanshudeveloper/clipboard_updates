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
  const [retainTagged, setRetainTagged] = useState(false);

  const MIN_LOADING_MS = 1000; // 1s minimum loading effect

  useEffect(() => {
    loadPurgeSettings();
  }, []);

  const withMinLoading = async (fn) => {
    setIsLoading(true);
    const start = Date.now();

    try {
      await fn();
    } finally {
      const elapsed = Date.now() - start;
      const remaining = Math.max(0, MIN_LOADING_MS - elapsed);
      setTimeout(() => setIsLoading(false), remaining);
    }
  };

  const loadPurgeSettings = async () => {
    try {
      console.log("📋 Loading purge settings...");

      const options = await invoke("get_purge_cadence_options");
      setAvailablePurgeOptions(options);
      console.log("✅ Available purge options:", options);

      const settings = await invoke("get_current_purge_settings");
      console.log("✅ Current purge settings:", settings);

      setAutoPurgeUnpinned(settings.auto_purge_enabled);
      setPurgeCadence(settings.purge_cadence);

      if (typeof settings.retain_tags === "boolean") {
        setRetainTagged(settings.retain_tags);
      }
    } catch (error) {
      console.error("❌ Failed to load purge settings:", error);
    }
  };

  const handleRetainTaggedChange = async (checked) => {
    await withMinLoading(async () => {
      // optimistic update
      setRetainTagged(checked);
      console.log("🔄 Updating retain_tags to:", checked);

      try {
        await invoke("update_retain_tags_setting", {
          retainTags: checked,
        });
        console.log("✅ retain_tags updated successfully");
      } catch (error) {
        console.error("❌ Failed to update retain_tags:", error);
        alert("Failed to update retain tagged setting: " + error);
        // revert on error
        setRetainTagged(!checked);
      }
    });
  };

  const handlePurgeCadenceChange = async (newCadence) => {
    await withMinLoading(async () => {
      const prevCadence = purgeCadence;

      // optimistic update
      setPurgeCadence(newCadence);
      if (newCadence === "Never") {
        setAutoPurgeUnpinned(false);
      } else {
        setAutoPurgeUnpinned(true);
      }

      console.log("🔄 Changing purge cadence to:", newCadence);

      try {
        await invoke("update_purge_cadence", {
          purgeCadence: newCadence,
        });
        console.log("✅ Purge cadence updated successfully");
      } catch (error) {
        console.error("❌ Failed to update purge cadence:", error);
        alert("Failed to update purge cadence: " + error);
        // revert on error
        setPurgeCadence(prevCadence);
      }
    });
  };

  const purgeAllUnpinned = async () => {
    await withMinLoading(async () => {
      try {
        console.log("🗑️ Purging all unpinned entries...");
        const result = await invoke("purge_unpinned_entries");
        console.log(`✅ Purged ${result} unpinned entries`);
      } catch (error) {
        console.error("❌ Failed to purge entries:", error);
        alert("Failed to purge entries: " + error);
      }
    });
  };

  const purgeAllUntagged = async () => {
    await withMinLoading(async () => {
      try {
        console.log("🏷️ Purging all untagged entries...");
        const result = await invoke("purge_untagged_entries");
        console.log(`✅ Purged ${result} untagged entries`);
      } catch (error) {
        console.error("❌ Failed to purge entries:", error);
        alert("Failed to purge entries: " + error);
      }
    });
  };

  return (
    <div className="space-y-6">
      {/* Delete Section */}
      <div className="space-y-3">
        <h3 className="text-sm font-medium text-gray-700">Delete Unpinned Items</h3>

        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="retainTagged"
            checked={retainTagged}
            onChange={(e) => handleRetainTaggedChange(e.target.checked)}
            className="w-4 h-4 text-blue-500 rounded focus:ring-blue-400"
            disabled={isLoading}
          />
          <label htmlFor="retainTagged" className="text-sm text-gray-700">
            Retain tagged clips
          </label>
        </div>

        {isLoading && (
          <p className="text-xs text-gray-500 animate-pulse">
            Saving changes...
          </p>
        )}
      </div>

      {/* Purge Cadence Section */}
      <div className="space-y-2">
        <h3 className="text-sm font-medium text-gray-700">
          Schedule deleting unpinned items
        </h3>
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

      {/* Delete now button */}
      <div className="pt-4 space-y-4">
        <button
          onClick={() => {
            if (retainTagged) {
              purgeAllUntagged();
            } else {
              purgeAllUnpinned();
            }
          }}
          className="px-4 py-2 text-sm border border-red-300 rounded-lg bg-white text-red-700 cursor-pointer transition-colors duration-200 hover:bg-red-50 disabled:opacity-50 disabled:cursor-not-allowed"
          disabled={isLoading}
        >
          {isLoading ? "Processing..." : "🗑️ Delete now"}
        </button>
      </div>
    </div>
  );
};

export default GeneralSettings;
