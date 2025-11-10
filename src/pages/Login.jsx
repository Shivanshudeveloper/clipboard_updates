import React, { useState, useEffect } from "react";
import { Mail, Lock, Eye, EyeOff } from "lucide-react";
import { FcGoogle } from "react-icons/fc";
import { Link, useNavigate } from "react-router-dom";
import { 
  handleRedirectResult, 
  getCurrentUser,
  checkFirebaseSetup,
  signInWithEmail,
} from "../libs/firebaseAuth";
import { getAuth, onAuthStateChanged } from "firebase/auth";

import { invoke } from "@tauri-apps/api/core";

// Helper function to get organization ID for a user
async function getOrganizationIdForUser(userId) {
  try {
    const storedUser = localStorage.getItem('user');
    
    if (storedUser) {
      const userData = JSON.parse(storedUser);
      if (userData.organizationId) {
        return userData.organizationId;
      }
    }
    
    return null;
  } catch (error) {
    return null;
  }
}

// Handle post-login processing for both email and Google auth
async function handlePostLogin(user, navigate) {
  try {
    const idToken = await user.getIdToken();
    
    const userResponse = await invoke('login_user', {
      firebaseToken: idToken,
      displayName: user.displayName || "User",
    });
    
  } catch (error) {
    throw new Error(`Login processing failed: ${error.message}`);
  }
}

export default function LoginPage() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [debugInfo, setDebugInfo] = useState("");
  const [error, setError] = useState("");
  const navigate = useNavigate();

  useEffect(() => {
    const auth = getAuth();
    
    const unsubscribe = onAuthStateChanged(auth, async (user) => {
      if (user) {
        try {
          console.log("🔄 Firebase user found, recreating backend session...");
          
          const idToken = await user.getIdToken(true);
          
          const userResponse = await invoke('login_user', {
            firebaseToken: idToken,
            displayName: user.displayName || "User",
          });
          
          console.log("✅ Backend session recreated successfully");
          navigate("/home");
          
        } catch (error) {
          console.error("❌ Failed to recreate backend session:", error);
          await auth.signOut();
          navigate("/login");
        }
      } else {
        console.log("🔴 No Firebase user, showing login page");
        if (window.location.pathname !== '/login') {
          navigate("/login");
        }
      }
    });

    return () => unsubscribe();
  }, [navigate]);

  const handleEmailLogin = async (e) => {
    e.preventDefault();
    if (!email || !password) return;
    
    setIsLoading(true);
    setError(""); // Clear previous errors
    
    try {
      const user = await signInWithEmail(email, password);
      await handlePostLogin(user, navigate);
      
    } catch (error) {
      console.error("❌ Login failed:", error);
      
      let errorMessage = "Login failed. Please try again.";
      
      switch (error.code) {
        case 'auth/invalid-email':
          errorMessage = "Invalid email address.";
          break;
        case 'auth/user-disabled':
          errorMessage = "This account has been disabled.";
          break;
        case 'auth/user-not-found':
          errorMessage = "No account found with this email.";
          break;
        case 'auth/wrong-password':
          errorMessage = "Invalid email or password.";
          break;
        case 'auth/too-many-requests':
          errorMessage = "Too many failed attempts. Please try again later.";
          break;
        default:
          errorMessage = "Invalid email or password.";
      }
      
      setError(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  const runDebugCheck = () => {
    const debugData = checkFirebaseSetup();
    setDebugInfo(JSON.stringify(debugData, null, 2));
  };

  const clearStorage = () => {
    localStorage.removeItem('user');
    localStorage.removeItem('idToken');
    setDebugInfo("Storage cleared at: " + new Date().toISOString());
  };

  const debugLocalStorage = () => {
    const userData = localStorage.getItem('user');
    if (userData) {
    }
    alert(`LocalStorage data: ${userData || 'Empty'}`);
  };

  return (
    <div className="flex flex-col bg-white rounded-t-lg shadow-sm border border-gray-200" style={{ height: '565px' }}>
      {/* Header */}
      <div className="bg-white p-6 pb-0 flex-shrink-0">
        <div className="flex items-center gap-2 mb-2 justify-center">
          <div className="w-8 h-8 rounded-md bg-gradient-to-r from-blue-500 to-blue-400 flex items-center justify-center text-white text-sm font-semibold">
            ⌘
          </div>
          <h1 className="text-xl font-semibold text-gray-800">ClipTray</h1>
        </div>
        <p className="text-center text-sm text-gray-600">
          Sign in to access your clipboard history
        </p>
        
        {/* Debug Buttons - Remove in production */}
       
      </div>

      {/* Login Form */}
      <div className="flex-1 p-6 pt-4">
        <form onSubmit={handleEmailLogin} className="space-y-4">
          {/* Email Input */}
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1.5">
              Email Address
            </label>
            <div className="relative">
              <Mail size={16} className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" />
              <input
                type="email"
                value={email}
                onChange={(e) => {
                  setEmail(e.target.value);
                  setError(""); // Clear error when user starts typing
                }}
                className="w-full h-10 pl-10 pr-3 border border-gray-300 rounded-lg bg-white text-gray-800 text-sm outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder="Enter your email"
                required
              />
            </div>
          </div>

          {/* Password Input */}
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1.5">
              Password
            </label>
            <div className="relative">
              <Lock size={16} className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" />
              <input
                type={showPassword ? "text" : "password"}
                value={password}
                onChange={(e) => {
                  setPassword(e.target.value);
                  setError(""); // Clear error when user starts typing
                }}
                className="w-full h-10 pl-10 pr-10 border border-gray-300 rounded-lg bg-white text-gray-800 text-sm outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder="Enter your password"
                required
              />
              <button
                type="button"
                onClick={() => setShowPassword(!showPassword)}
                className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-gray-600"
              >
                {showPassword ? <EyeOff size={16} /> : <Eye size={16} />}
              </button>
            </div>
            
            {/* Error Message */}
            {error && (
              <div className="mt-2 text-xs text-red-500 flex items-center">
                <svg className="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
                </svg>
                {error}
              </div>
            )}
          </div>

          {/* Sign In Button */}
          <button
            type="submit"
            disabled={!email || !password || isLoading}
            className="w-full h-10 bg-blue-500 text-white rounded-lg font-medium text-sm hover:bg-blue-600 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {isLoading ? (
              <div className="flex items-center justify-center gap-2">
                <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                Signing in...
              </div>
            ) : (
              "Sign in with Email"
            )}
          </button>
        </form>

        {/* Sign Up Link */}
        <div className="text-center mt-4 pt-4 border-t border-gray-200">
          <p className="text-xs text-gray-600">
            Don't have an account?{" "}
            <Link to="/signup">
              <button className="text-blue-500 hover:text-blue-600 font-medium">
                Sign up
              </button>
            </Link>
          </p>
        </div>
      </div>

      {/* Footer */}
      <div className="p-4 text-center text-xs text-gray-400 bg-white flex-shrink-0 rounded-b-lg">
        Create with ❤️ by MakerStudio
      </div>
    </div>
  );
}