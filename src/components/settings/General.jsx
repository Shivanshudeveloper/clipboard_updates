import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Lock ,Info, Crown} from "lucide-react";
import { useUserPlan } from "../../hooks/useUserPlan"; // adjust path if needed
import { usePayment } from "../../hooks/usePayment";
import { Link, useNavigate } from 'react-router-dom';

const GeneralSettings = ({
  startAtLogin,
  setStartAtLogin,
  autoPurgeUnpinned,
  setAutoPurgeUnpinned,
  purgeCadence,
  setPurgeCadence,
}) => {
  const [availablePurgeOptions, setAvailablePurgeOptions] = useState([]);
  const [isLoading, setIsLoading] = useState(false);
  const [retainTagged, setRetainTagged] = useState(false);

  const { isFree, loading: planLoading, refetchPlan } = useUserPlan();
  const { openPaymentWebsite, isPolling, pollingError } = usePayment();
  const navigate = useNavigate();

  const MIN_LOADING_MS = 200;

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

  useEffect(() => {
    loadRetainTags();
    loadPurgeSettings();
  }, []);

  const loadRetainTags = async () => {
    try {
      const value = await invoke("get_current_user_retain_tags");
      setRetainTagged(value);
    } catch (err) {
      console.error("❌ Failed to load retain_tags:", err);
    }
  };

  const loadPurgeSettings = async () => {
    try {
      const options = await invoke("get_purge_cadence_options");
      setAvailablePurgeOptions(options);

      const settings = await invoke("get_current_purge_settings");

      // ✅ Free plan locked to Every 24 hours
      if (!planLoading && isFree) {
        setAutoPurgeUnpinned(true);
        setPurgeCadence("Every 24 hours");
        return;
      }

      setAutoPurgeUnpinned(settings.auto_purge_enabled);
      setPurgeCadence(settings.purge_cadence);

      if (typeof settings.retain_tags === "boolean") {
        setRetainTagged(settings.retain_tags);
      }
    } catch (error) {
      console.error("❌ Failed to load purge settings:", error);
    }
  };

  // keep locked even after plan loads
  useEffect(() => {
    if (!planLoading && isFree) {
      setAutoPurgeUnpinned(true);
      setPurgeCadence("Every 24 hours");
    }
  }, [isFree, planLoading, setAutoPurgeUnpinned, setPurgeCadence]);

  const handleRetainTaggedChange = async (checked) => {
    await withMinLoading(async () => {
      setRetainTagged(checked);
      try {
        await invoke("update_retain_tags_setting", { retainTags: checked });
      } catch (error) {
        // alert("Failed to update retain tagged setting: " + error);
        setRetainTagged(!checked);
      }
    });
  };

  const handlePurgeCadenceChange = async (newCadence) => {
    // ✅ block free plan (UI + behavior)
    if (!planLoading && isFree && newCadence !== "Every 24 hours") {
      // alert("Upgrade to Pro to change the auto-delete schedule.");
      setPurgeCadence("Every 24 hours");
      setAutoPurgeUnpinned(true);
      return;
    }

    await withMinLoading(async () => {
      const prevCadence = purgeCadence;

      setPurgeCadence(newCadence);
      setAutoPurgeUnpinned(newCadence !== "Never");

      try {
        await invoke("update_purge_cadence", { purgeCadence: newCadence });
      } catch (error) {
        // alert("Failed to update purge cadence: " + error);
        setPurgeCadence(prevCadence);
      }
    });
  };

  const purgeAllUnpinned = async () => {
    await withMinLoading(async () => {
      try {
        await invoke("purge_unpinned_entries");
      } catch (error) {
        alert("Failed to purge entries: " + error);
      }
    });
  };

  const purgeAllUntagged = async () => {
    await withMinLoading(async () => {
      try {
        await invoke("purge_untagged_entries");
      } catch (error) {
        alert("Failed to purge entries: " + error);
      }
    });
  };

  const isOptionLocked = (option) =>
    !planLoading && isFree && option !== "Every 24 hours";

  return (
    <div className="space-y-6">
{!planLoading && isFree && (
  <div className="rounded-xl border border-gray-200 bg-gray-100 p-3 mb-2">
  <div className="flex items-start gap-2">
    <Info size={16} className="text-blue-500 mt-0.5" />
    <div className="flex-1">
      <div className="text-sm font-semibold text-gray-800">Free Plan Limits</div>

      <div className="mt-2 flex items-center justify-between text-xs text-gray-600">
        <span>Pinned Clips:</span>
        <span className="font-semibold text-orange-500">3/3</span>
      </div>

      <div className="mt-2 h-2 w-full rounded-full bg-gray-200 overflow-hidden">
        <div className="h-full rounded-full bg-orange-400" style={{ width: "100%" }} />
      </div>

      <div className="mt-2 text-[12px] text-gray-600">
        <span className="text-gray-400">•</span> Auto-deletion: Every 24 hours only
      </div>

      <button
  type="button"
  onClick={async () => {
    const opened = await openPaymentWebsite();
    if (opened) {
      // Polling will start automatically
      // Refresh plan when polling detects payment
      const checkInterval = setInterval(async () => {
        if (!isPolling) {
          clearInterval(checkInterval);
          await refetchPlan();
        }
      }, 1000);
    }
  }}
  disabled={isPolling}
  className="mt-3 w-full h-9 rounded-lg text-sm font-semibold text-white flex items-center justify-center gap-2
             bg-gradient-to-r from-blue-500 via-indigo-500 to-purple-500 hover:opacity-95 active:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed"
>
  <Crown size={16} />
  {isPolling ? "Checking payment..." : "Upgrade to Pro"}
</button>
      {pollingError && (
        <p className="mt-2 text-xs text-red-600">{pollingError}</p>
      )}

    </div>
  </div>
</div>

)}
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
          <p className="text-xs text-gray-500 animate-pulse">Saving changes...</p>
        )}
      </div>

      {/* Purge Cadence Section (UI like screenshot) */}
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-medium text-gray-700">
            Schedule deleting unpinned items
          </h3>
          {!planLoading && isFree && <Lock size={14} className="text-gray-400" />}
        </div>

        <div className="space-y-1.5">
          {availablePurgeOptions.map((option) => {
            const locked = isOptionLocked(option);

            return (
              <div key={option} className="flex items-center justify-between">
                <label className={`flex items-start gap-2 ${locked ? "opacity-50" : ""}`}>
                  <input
                    type="radio"
                    id={option}
                    name="purgeCadence"
                    checked={purgeCadence === option}
                    onChange={() => handlePurgeCadenceChange(option)}
                    className="w-4 h-4 text-blue-500 focus:ring-blue-400 mt-0.5"
                    disabled={isLoading}
                  />

                  <div className="leading-tight">
                    <div className="text-sm text-gray-700">{option}</div>
                    {locked && (
                      <div className="text-[11px] text-orange-400">Paid plan only</div>
                    )}
                  </div>
                </label>

                {locked && <Lock size={14} className="text-gray-400" />}
              </div>
            );
          })}
        </div>

        {!planLoading && isFree && (
          <p className="text-xs text-gray-500">
            Free plan is limited to <span className="font-semibold">Every 24 hours</span>.
            Upgrade to unlock more schedules.
          </p>
        )}
      </div>

      {/* Delete now button */}
      <div className="pt-4 space-y-4">
        <button
          onClick={() => {
            if (retainTagged) purgeAllUntagged();
            else purgeAllUnpinned();
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
