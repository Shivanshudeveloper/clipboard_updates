import { useState, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

function isTauri() {
  return "__TAURI__" in window;
}

export function usePayment() {
  const [isPolling, setIsPolling] = useState(false);
  const [pollingError, setPollingError] = useState(null);
  const pollingIntervalRef = useRef(null);
  const consecutiveFailuresRef = useRef(0);
  const maxConsecutiveFailures = 5;

  const stopPolling = useCallback(() => {
    if (pollingIntervalRef.current) {
      clearInterval(pollingIntervalRef.current);
      pollingIntervalRef.current = null;
    }
    setIsPolling(false);
    consecutiveFailuresRef.current = 0;
  }, []);

  const checkPaymentStatus = useCallback(async (firebaseUid) => {
    try {
      const hasActivePlan = await invoke("check_payment_status", {
        firebaseUid: firebaseUid,
      });

      // Reset failure counter on success
      consecutiveFailuresRef.current = 0;

      if (hasActivePlan) {
        console.log("✅ Payment detected!");
        stopPolling();

        // Refresh user plan from backend
        try {
          await invoke("refresh_user_plan_from_backend");
          console.log("✅ User plan refreshed from backend");
        } catch (error) {
          console.error("❌ Failed to refresh user plan:", error);
        }

        return true;
      }

      return false;
    } catch (error) {
      consecutiveFailuresRef.current += 1;
      console.error(
        `❌ Payment check failed (consecutive failures: ${consecutiveFailuresRef.current}):`,
        error
      );

      if (consecutiveFailuresRef.current >= maxConsecutiveFailures) {
        console.log(
          `⚠️ Stopping polling after ${maxConsecutiveFailures} consecutive failures`
        );
        stopPolling();
        setPollingError(
          "Payment check failed. You can upgrade later - payment will be detected on next app launch."
        );
        return false;
      }

      return false;
    }
  }, [stopPolling]);

  const startPaymentPolling = useCallback(
    async (firebaseUid) => {
      if (!isTauri()) {
        console.error("❌ Not in Tauri environment");
        return;
      }

      if (pollingIntervalRef.current) {
        console.log("⚠️ Polling already in progress");
        return;
      }

      console.log("🔄 Starting payment status polling...");
      setIsPolling(true);
      setPollingError(null);
      consecutiveFailuresRef.current = 0;

      // Check immediately first
      const immediateCheck = await checkPaymentStatus(firebaseUid);
      if (immediateCheck) {
        return; // Payment already detected
      }

      // Then poll every 3 seconds
      pollingIntervalRef.current = setInterval(async () => {
        const paymentDetected = await checkPaymentStatus(firebaseUid);
        if (paymentDetected) {
          // Payment detected, polling will be stopped in checkPaymentStatus
          return;
        }
      }, 3000);
    },
    [checkPaymentStatus]
  );

  const openPaymentWebsite = useCallback(async () => {
    if (!isTauri()) {
      console.error("❌ Not in Tauri environment");
      return false;
    }

    try {
      console.log("🌐 Opening payment website...");
      await invoke("open_payment_website");
      console.log("✅ Payment website opened");

      // Get Firebase UID from session for polling
      try {
        const sessionState = await invoke("debug_session_state");
        const firebaseUid = sessionState?.user_id;
        
        if (!firebaseUid) {
          console.error("❌ Could not get Firebase UID for polling");
          return false;
        }

        // Start polling after opening website
        await startPaymentPolling(firebaseUid);
      } catch (error) {
        console.error("❌ Failed to get Firebase UID from session:", error);
        return false;
      }
      return true;
    } catch (error) {
      console.error("❌ Failed to open payment website:", error);
      setPollingError("Failed to open payment website. Please try again.");
      return false;
    }
  }, [startPaymentPolling]);

  return {
    openPaymentWebsite,
    isPolling,
    pollingError,
    stopPolling,
  };
}

