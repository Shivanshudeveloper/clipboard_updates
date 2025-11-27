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
    await signOut(auth);
    
    // Clear any stored data
    localStorage.removeItem('user');
    localStorage.removeItem('idToken');
    
    console.log("User signed out successfully");
    return true;
  } catch (error) {
    console.error("Sign out failed:", error);
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