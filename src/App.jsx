// src/App.jsx
import { useEffect, useState } from 'react';
import ClipTray from "./pages/Landing";
import { HashRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import LoginPage from "./pages/Login";
import SignupPage from "./pages/Signup";
import { handleRedirectResult } from './libs/firebaseAuth-redirect';
import { auth } from './libs/firebaseConfig';
import { onAuthStateChanged } from 'firebase/auth';
import ClipTraySettings from './pages/Settings';

function App() {  
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(true);
  const [authChecked, setAuthChecked] = useState(false);

  useEffect(() => {
    
    const unsubscribe = onAuthStateChanged(auth, async (firebaseUser) => {
      
      try {
        // Handle redirect result for Google sign-in
        if (!authChecked) {
          const redirectResult = await handleRedirectResult();
          if (redirectResult) {
            setUser(redirectResult.user);
            setLoading(false);
            setAuthChecked(true);
            return;
          }
          setAuthChecked(true);
        }

        if (firebaseUser) {
          setUser(firebaseUser);
          
          // Update localStorage for consistency
          const userData = {
            uid: firebaseUser.uid,
            email: firebaseUser.email,
            displayName: firebaseUser.displayName,
            photoURL: firebaseUser.photoURL
          };
          localStorage.setItem('user', JSON.stringify(userData));
        } else {
          setUser(null);
          localStorage.removeItem('user');
        }
      } catch (error) {
        console.error("❌ Auth state change error:", error);
      } finally {
        setLoading(false);
      }
    });

    return () => unsubscribe();
  }, [authChecked]);

  // Prevent excessive re-renders
  useEffect(() => {
  }, [user]);

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

  return (
    <Router>
      <Routes>
        <Route path="/home" element={user ? <ClipTray user={user} /> : <Navigate to="/login" replace />} />
        <Route path="/login" element={!user ? <LoginPage /> : <Navigate to="/home" replace />} />
        <Route path="/signup" element={!user ? <SignupPage /> : <Navigate to="/home" replace />} />
        <Route path="/" element={<Navigate to={user ? "/home" : "/login"} replace />} />
        <Route path="/settings" element={<ClipTraySettings/>}/>
      </Routes>
    </Router>
  );
}

export default App;