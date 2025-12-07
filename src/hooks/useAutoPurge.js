import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useAutoPurge() {
  useEffect(() => {
    const interval = setInterval(() => {
      console.log("⏱️ Running auto purge…");

      invoke("run_auto_purge_now")
        .then((count) => console.log("🧹 Purged entries:", count))
        .catch((err) => console.error("Purge failed", err));
    }, 10 * 1000); // every 1 hour

    return () => clearInterval(interval);
  }, []);
}
