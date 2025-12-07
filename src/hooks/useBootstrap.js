import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useBootstrap() {
  useEffect(() => {
    const runBootstrap = async () => {
      if (!navigator.onLine) {
        console.log("⚠️ Offline — skipping cloud bootstrap");
        return;
      }

      try {
        console.log("🌐 Online — running cloud bootstrap...");
        const synced = await invoke("bootstrap_cloud_now");
        console.log(`✅ Cloud bootstrap completed → ${synced} entries updated`);
      } catch (err) {
        console.error("❌ Cloud bootstrap failed:", err);
      }
    };

    // Run immediately on mount
    runBootstrap();

    // Re-run when user comes online
    const handleOnline = () => runBootstrap();
    window.addEventListener("online", handleOnline);

    return () => window.removeEventListener("online", handleOnline);
  }, []);
}
