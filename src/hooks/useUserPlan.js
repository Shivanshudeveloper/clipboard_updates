import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

function isTauri() {
  return "__TAURI__" in window;
}

export function useUserPlan({ autoFetch = true } = {}) {
  const [plan, setPlan] = useState(null);        // "Free" | "Pro"
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const fetchUserPlan = useCallback(async () => {
    if (!isTauri()) return;

    setLoading(true);
    setError(null);

    try {
      const result = await invoke("get_user_plan");
      setPlan(result); // backend already returns display string
    } catch (err) {
      console.error("❌ Failed to fetch user plan:", err);
      setError(err?.toString() || "Failed to fetch user plan");
      setPlan(null);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (autoFetch) {
      fetchUserPlan();
    }
  }, [autoFetch, fetchUserPlan]);

  return {
    plan,                 // "Free" | "Pro"
    isFree: plan === "Free",
    isPro: plan === "Pro",
    loading,
    error,
    refetchPlan: fetchUserPlan,
  };
}
