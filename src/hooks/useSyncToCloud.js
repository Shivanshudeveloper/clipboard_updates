// src/hooks/useSyncToCloud.js
import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
function isTauri() {
  return "__TAURI__" in window;
}

export function useSyncToCloud() {
  const isSyncingRef = useRef(false);

  useEffect(() => {
    if (!isTauri()) {
      console.log("⛔ Not running inside Tauri — cloud sync disabled");
      return;
    }

    const syncNow = async (reason) => {
      if (!navigator.onLine) {
        console.log(`⚠️ Skipping sync (${reason}) — offline`);
        return;
      }

      if (isSyncingRef.current) {
        console.log("⏳ Sync already running — skip");
        return;
      }

      try {
        isSyncingRef.current = true;
        console.log(`☁️ Running cloud sync (${reason})...`);
        const synced = await invoke("sync_clipboard_to_cloud");
        console.log(`✅ Cloud sync completed → ${synced} updated (${reason})`);
      } catch (err) {
        console.error("❌ Cloud sync failed:", err);
      } finally {
        isSyncingRef.current = false;
      }
    };

    // 1️⃣ Run once on mount if online
    if (navigator.onLine) {
      syncNow("initial-mount");
    } else {
      console.log("⚠️ Started offline — will sync when online");
    }

    // 2️⃣ Sync whenever user comes back online
    const handleOnline = () => syncNow("online-event");
    window.addEventListener("online", handleOnline);

    const intervalId = setInterval(() => {
      syncNow("interval");
    }, 60000);

    return () => {
      window.removeEventListener("online", handleOnline);
      clearInterval(intervalId);
    };
  }, []);
}
