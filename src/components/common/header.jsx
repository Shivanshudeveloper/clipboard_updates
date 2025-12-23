// src/components/home/Header.jsx
import React from "react";
import { Search, X, LogOut, Settings, Crown, Sparkles } from "lucide-react";
import { Link } from "react-router-dom";
import { usePayment } from "../../hooks/usePayment";
import { useUserPlan } from "../../hooks/useUserPlan";

export default function Header({
  q,
  setQ,
  onLogout,
  isLoggingOut,
  showUpgradeBanner = false,   // ✅ NEW
  onUpgradeClick,              // ✅ optional
  onDismissUpgrade,            // ✅ optional
}) {
  const { isFree, refetchPlan } = useUserPlan();
  const { openPaymentWebsite, isPolling } = usePayment();

  const handleUpgradeClick = async () => {
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
  };
  return (
    <div className="bg-white p-2 flex-shrink-0">
      {/* Top row: logo + title + settings + logout */}
      <div className="flex items-center justify-between mb-1.5">
        <div className="flex items-center gap-2">
          <div className="w-5 h-5 rounded-md bg-gradient-to-r from-blue-500 to-blue-400 flex items-center justify-center text-white text-xs font-semibold shadow-sm">
            ⌘
          </div>
          <h1 className="text-sm font-semibold text-gray-800">ClipTray 1.0</h1>
        </div>

        <div className="flex gap-2">
          <div className="mt-1">
            <Link to="/settings">
              <Settings size={18} className="text-gray-600 hover:text-gray-800 transition-colors" title="Settings" />
            </Link>
          </div>

          <button
            onClick={onLogout}
            disabled={isLoggingOut}
            className="flex items-center gap-1 px-2 py-1 text-xs text-gray-600 bg-white border border-gray-300 rounded-md hover:bg-gray-50 hover:border-gray-400 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
            title="Logout"
          >
            {isLoggingOut ? (
              <div className="w-3 h-3 border-2 border-gray-400 border-t-transparent rounded-full animate-spin" />
            ) : (
              <LogOut size={12} />
            )}
            Logout
          </button>
        </div>
      </div>

      {/* ✅ Upgrade banner (ABOVE search bar) */}
      {showUpgradeBanner && (
        <div className="mb-1.5 px-2.5 py-1.5 rounded-md border border-yellow-200 bg-gradient-to-r from-yellow-50 to-amber-50 text-[11px] text-yellow-800 flex items-center justify-between shadow-sm">
          <div className="flex items-center gap-1.5">
            <Sparkles size={12} className="text-yellow-600" />
            <span>Limit reached — upgrade plan</span>
          </div>

          <div className="flex items-center gap-2">
            <button
              onClick={handleUpgradeClick}
              className="font-semibold underline hover:text-yellow-900 transition-colors"
            >
              Upgrade
            </button>
            <button
              onClick={onDismissUpgrade}
              className="text-yellow-900/70 hover:text-yellow-900 transition-colors"
              title="Dismiss"
            >
              <X size={12} />
            </button>
          </div>
        </div>
      )}

      {/* Search bar */}
      <div className="relative">
        {/* Upgrade button - right side (only for Free users) */}
        {isFree && (
          <button
            onClick={handleUpgradeClick}
            disabled={isPolling}
            className="absolute right-2 top-1/2 transform -translate-y-1/2 z-10 flex items-center gap-1 px-2 py-0.5 rounded-md bg-gradient-to-r from-blue-500 via-indigo-500 to-purple-500 text-white text-[10px] font-semibold hover:opacity-90 active:opacity-80 disabled:opacity-50 disabled:cursor-not-allowed transition-all shadow-sm"
            title="Upgrade to Pro"
          >
            <Crown size={10} className="fill-white" />
            <span>{isPolling ? "..." : "Upgrade Pro"}</span>
          </button>
        )}
        <Search
          size={12}
          className={`absolute top-1/2 transform -translate-y-1/2 text-gray-400 transition-all ${
            isFree ? "left-14" : "left-2"
          }`}
        />
        <input
          className={`w-full h-6 border border-gray-300 rounded-md bg-gray-50 text-gray-800 text-xs outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-400 transition-all ${
            isFree ? "pl-20 pr-7" : "pl-7 pr-7"
          }`}
          placeholder="Search clips"
          value={q}
          onChange={(e) => setQ(e.target.value)}
        />
        {q && (
          <button
            className="absolute right-2 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-gray-600 transition-colors"
            onClick={() => setQ("")}
            title="Clear search"
          >
            <X size={12} />
          </button>
        )}
      </div>
    </div>
  );
}
