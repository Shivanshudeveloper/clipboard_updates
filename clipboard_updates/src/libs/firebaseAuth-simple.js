// TEMPORARY: firebaseAuth-simple.js (use this for testing)
import { 
  signInWithPopup, 
  GoogleAuthProvider, 
  signOut
} from "firebase/auth";
import { auth, googleProvider } from "./firebaseConfig";

/**
 * Simple Google Sign-In without Rust backend
 */
export async function handleGoogleSignIn() {
  try {
    console.log("🔐 Starting Google Sign-In (Simple version)...");

    // 🔹 1. Firebase popup sign-in only
    const result = await signInWithPopup(auth, googleProvider);
    const user = result.user;
    
    console.log("✅ Firebase auth successful:", user.email);

    // Store user data
    const userData = {
      uid: user.uid,
      email: user.email,
      displayName: user.displayName,
      photoURL: user.photoURL
    };
    localStorage.setItem('user', JSON.stringify(userData));

    console.log("✅ User data stored, returning success");
    
    return {
      user: userData,
      backendAvailable: false,
      message: "Frontend auth only - Rust backend skipped"
    };

  } catch (error) {
    console.error("❌ Google Sign-In failed:", {
      code: error.code,
      message: error.message
    });
    
    throw new Error(`Sign-in failed: ${error.message}`);
  }
}

export const getCurrentUser = () => {
  const storedUser = localStorage.getItem('user');
  return storedUser ? JSON.parse(storedUser) : null;
};

// Update your import in Signup.jsx to use the simple version temporarily
// import { handleGoogleSignIn } from "../libs/firebaseAuth-simple";