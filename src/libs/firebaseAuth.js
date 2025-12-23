import { 
  getAuth, 
  signInWithEmailAndPassword, 
  createUserWithEmailAndPassword,
  signInWithRedirect,
  getRedirectResult,
  GoogleAuthProvider,
  signOut 
} from "firebase/auth";
import { app } from "./firebaseConfig";

const auth = getAuth(app);
const googleProvider = new GoogleAuthProvider();

// Email/Password Sign In
export const signInWithEmail = async (email, password) => {
  try {
    const userCredential = await signInWithEmailAndPassword(auth, email, password);
    return userCredential.user;
  } catch (error) {
    throw error;
  }
};

// Email/Password Sign Up
export const signUpWithEmail = async (email, password) => {
  try {
    const userCredential = await createUserWithEmailAndPassword(auth, email, password);
    return userCredential.user;
  } catch (error) {
    throw error;
  }
};

// Existing Google auth functions
export const handleGoogleSignIn = async () => {
  try {
    await signInWithRedirect(auth, googleProvider);
  } catch (error) {
    throw error;
  }
};

export const handleRedirectResult = async () => {
  try {
    const result = await getRedirectResult(auth);
    return result;
  } catch (error) {
    throw error;
  }
};

export const getCurrentUser = () => {
  return auth.currentUser;
};

export const signOutUser = async () => {
  try {
    // Sign out from Firebase first
    await signOut(auth);
    
    // Clear ALL localStorage items
    localStorage.clear();
    
    // Clear ALL sessionStorage items
    sessionStorage.clear();
    
    // Clear any cookies (if any exist)
    document.cookie.split(";").forEach((c) => {
      const eqPos = c.indexOf("=");
      const name = eqPos > -1 ? c.substr(0, eqPos).trim() : c.trim();
      document.cookie = `${name}=;expires=Thu, 01 Jan 1970 00:00:00 GMT;path=/`;
      document.cookie = `${name}=;expires=Thu, 01 Jan 1970 00:00:00 GMT;path=/;domain=${window.location.hostname}`;
    });
    
    console.log("✅ User signed out successfully - all storage cleared");
    return true;
  } catch (error) {
    console.error("❌ Sign out failed:", error);
    throw error;
  }
};

export const checkFirebaseSetup = () => {
  return {
    auth: auth ? "✅ Configured" : "❌ Not configured",
    app: app ? "✅ Configured" : "❌ Not configured",
    currentUser: auth?.currentUser ? `✅ ${auth.currentUser.email}` : "❌ No user",
    providers: {
      google: GoogleAuthProvider ? "✅ Available" : "❌ Not available",
      email: "✅ Available (if enabled in console)"
    }
  };
};