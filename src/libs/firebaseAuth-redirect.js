// src/libs/firebaseAuth-redirect.js
import { 
  signInWithRedirect, 
  getRedirectResult,
  GoogleAuthProvider, 
  signOut
} from "firebase/auth";
import { auth, googleProvider } from "./firebaseConfig";

/**
 * Sign in with Google using REDIRECT method (works better with Tauri)
 */
export async function handleGoogleSignIn() {
  try {
    console.log("🔐 Starting Google Sign-In with Redirect...");
    console.log("📍 Current URL:", window.location.href);

    // Store that we're starting auth
    localStorage.setItem('google_auth_started', 'true');
    localStorage.setItem('google_auth_timestamp', Date.now().toString());
    
    
    // Use redirect instead of popup
    console.log("🔄 Redirecting to Google...");
    await signInWithRedirect(auth, googleProvider);
    
    // The app will redirect away, so we don't return anything here
    return { status: "redirecting" };

  } catch (error) {
    console.error("❌ Google Sign-In failed:", error);
    localStorage.removeItem('google_auth_started');
    localStorage.removeItem('google_auth_timestamp');
    throw new Error(`Sign-in failed: ${error.message}`);
  }
}

/**
 * Handle the redirect result when user returns
 */
export async function handleRedirectResult() {
  try {
    console.log("🔄 Checking for redirect result...");
    
    const result = await getRedirectResult(auth);
    
    if (result) {
      const user = result.user;
      console.log("✅ Redirect sign-in successful:", user.email);

      const userData = {
        uid: user.uid,
        email: user.email,
        displayName: user.displayName,
        photoURL: user.photoURL
      };
      
      localStorage.setItem('user', JSON.stringify(userData));
      localStorage.removeItem('google_auth_started');
      localStorage.removeItem('google_auth_timestamp');
      
      return {
        user: userData,
        backendAvailable: false
      };
    }
    
    // Check if we were in the middle of auth but no result
    const authStarted = localStorage.getItem('google_auth_started');
    if (authStarted) {
      console.log("⚠️ Auth was started but no result found");
      localStorage.removeItem('google_auth_started');
      localStorage.removeItem('google_auth_timestamp');
    }
    
    return null;
  } catch (error) {
    console.error("❌ Redirect result error:", error);
    localStorage.removeItem('google_auth_started');
    localStorage.removeItem('google_auth_timestamp');
    throw error;
  }
}

export const getCurrentUser = () => {
  const storedUser = localStorage.getItem('user');
  return storedUser ? JSON.parse(storedUser) : null;
};

export const handleSignOut = async () => {
  try {
    await signOut(auth);
    localStorage.removeItem('user');
    console.log('✅ Signed out successfully');
  } catch (error) {
    console.error('❌ Sign-out failed:', error);
    throw error;
  }
};