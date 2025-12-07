// src/hooks/useAutoUpdate.js
import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

function isTauri() {
  return typeof window !== "undefined" && "__TAURI__" in window;
}

export function useAutoUpdate() {
  useEffect(() => {
    if (!isTauri()) {
      console.log("⛔ Not running inside Tauri — auto-update disabled");
      return;
    }

    let unlistenUpdate;
    let unlistenDownload;

    const setupListeners = async () => {
      try {
        // Fired from backend when an update is available
        unlistenUpdate = await listen("update-available", async (event) => {
          console.log("🔔 [AutoUpdate] Update available event:", event.payload);
          try {
            console.log("🚀 [AutoUpdate] Starting automatic update from event...");
            await invoke("auto_update");
          } catch (err) {
            console.error("❌ [AutoUpdate] auto_update from event failed:", err);
          }
        });

        // Optional: just logs download progress for now
        unlistenDownload = await listen("download-progress", (event) => {
          console.log("⬇️ [AutoUpdate] Download progress:", event.payload);
        });
      } catch (err) {
        console.error("❌ [AutoUpdate] Failed to set up listeners:", err);
      }
    };

    const checkForUpdatesOnStart = async () => {
      try {
        console.log("🔍 [AutoUpdate] Checking for updates on startup...");
        const result = await invoke("check_for_updates");
        console.log("🔎 [AutoUpdate] check_for_updates result:", result);

        if (result && result.available) {
          console.log("🚀 [AutoUpdate] Update available, calling auto_update...");
          await invoke("auto_update");
        } else {
          console.log("✅ [AutoUpdate] App is up to date");
        }
      } catch (err) {
        console.error("❌ [AutoUpdate] Startup update check failed:", err);
      }
    };

    setupListeners();
    checkForUpdatesOnStart();

    return () => {
      if (unlistenUpdate) unlistenUpdate();
      if (unlistenDownload) unlistenDownload();
    };
  }, []);
}
