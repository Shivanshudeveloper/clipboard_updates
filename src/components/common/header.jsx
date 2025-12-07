// src/components/home/Header.jsx
import React from "react";
import { Search, X, LogOut, Settings } from "lucide-react";
import { Link } from "react-router-dom";

export default function Header({ q, setQ, onLogout, isLoggingOut }) {
  return (
    <div className="bg-white p-2 flex-shrink-0">
      {/* Top row: logo + title + settings + logout */}
      <div className="flex items-center justify-between mb-1">
        <div className="flex items-center gap-2">
          <div className="w-5 h-5 rounded-md bg-gradient-to-r from-blue-500 to-blue-400 flex items-center justify-center text-white text-xs font-semibold">
            ⌘
          </div>
          <h1 className="text-sm font-semibold text-gray-800">ClipTray 1.0</h1>
        </div>

        <div className="flex gap-2">
          <div className="mt-1">
            <Link to="/settings">
              <Settings size={18} className="text-gray-600" />
            </Link>
          </div>

          <button
            onClick={onLogout}
            disabled={isLoggingOut}
            className="flex items-center gap-1 px-2 py-1 text-xs text-gray-600 bg-white border border-gray-300 rounded-md hover:bg-gray-50 hover:border-gray-400 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
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

      {/* Search bar */}
      <div className="relative">
        <Search
          size={12}
          className="absolute left-2 top-1/2 transform -translate-y-1/2 text-gray-400"
        />
        <input
          className="w-full h-6 pl-7 pr-7 border border-gray-300 rounded-md bg-gray-50 text-gray-800 text-xs outline-none focus:ring-1 focus:ring-blue-500"
          placeholder="Search clips"
          value={q}
          onChange={(e) => setQ(e.target.value)}
        />
        {q && (
          <button
            className="absolute right-2 top-1/2 transform -translate-y-1/2 text-gray-400"
            onClick={() => setQ("")}
          >
            <X size={12} />
          </button>
        )}
      </div>
    </div>
  );
}


