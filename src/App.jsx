// src/App.jsx
import { useEffect, useState } from "react";
import { HashRouter as Router, Routes, Route, Navigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";

import ClipTray from "./pages/Landing";
import LoginPage from "./pages/Login";
import SignupPage from "./pages/Signup";
import ClipTraySettings from "./pages/Settings";

function App() {
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;

    const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

    const init = async () => {
      try {
        // 1️⃣ Wait for DB to be ready (poll check_database_status)
        let dbReady = false;
        for (let i = 0; i < 60 && !dbReady && !cancelled; i++) {
          try {
            dbReady = await invoke("check_database_status");
          } catch (e) {
            console.log("check_database_status failed:", e);
          }
          if (!dbReady) {
            await sleep(250); // 15s max
          }
        }

        if (cancelled) return;

        if (!dbReady) {
          console.log("⚠️ DB not ready after timeout, treating as no session");
          setUser(null);
          return;
        }

        console.log("✅ DB is ready, checking session...");

        // 2️⃣ Try restore from localStorage
        const stored = localStorage.getItem("cliptray_user");
        if (stored) {
          try {
            const parsed = JSON.parse(stored);
            if (parsed.organization_id) {
              console.log("🧩 Found stored user, calling restore_session...");
              const restoredUser = await invoke("restore_session", {
                organizationId: parsed.organization_id,
              });

              if (restoredUser) {
                console.log("✅ Session restored from stored user");
                setUser(restoredUser);
                return;
              } else {
                console.log("⚠ Stored user invalid, clearing localStorage");
                localStorage.removeItem("cliptray_user");
              }
            }
          } catch (e) {
            console.log("⚠ Failed to parse stored user, clearing:", e);
            localStorage.removeItem("cliptray_user");
          }
        }

        // 3️⃣ Fallback: ask backend if there is an in-memory session
        try {
          const sessionUser = await invoke("validate_session");
          if (sessionUser) {
            console.log("✅ Backend in-memory session found");
            setUser(sessionUser);

            // sync to localStorage for next restart
            localStorage.setItem(
              "cliptray_user",
              JSON.stringify({
                user_id: sessionUser.user_id,
                organization_id: sessionUser.organization_id,
                email: sessionUser.email,
              })
            );

            return;
          }
        } catch (e) {
          console.log("validate_session failed:", e);
        }

        // 4️⃣ Nothing found → not logged in
        setUser(null);
      } finally {
        if (!cancelled) setLoading(false);
      }
    };

    init();

    return () => {
      cancelled = true;
    };
  }, []);

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-50">
        <div className="flex items-center gap-3">
          <div className="w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
          <span className="text-sm text-gray-600">Loading...</span>
        </div>
      </div>
    );
  }

  const isLoggedIn = !!user;

  return (
    <Router>
      <Routes>
        <Route
          path="/home"
          element={isLoggedIn ? <ClipTray user={user} /> : <Navigate to="/login" replace />}
        />
        <Route
          path="/login"
          element={!isLoggedIn ? <LoginPage /> : <Navigate to="/home" replace />}
        />
        <Route
          path="/signup"
          element={!isLoggedIn ? <SignupPage /> : <Navigate to="/home" replace />}
        />
        <Route
          path="/"
          element={<Navigate to={isLoggedIn ? "/home" : "/login"} replace />}
        />
        <Route
          path="/settings"
          element={isLoggedIn ? <ClipTraySettings /> : <Navigate to="/login" replace />}
        />
      </Routes>
    </Router>
  );
}

export default App;
