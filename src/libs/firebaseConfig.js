import { initializeApp } from "firebase/app";
import { getAuth, GoogleAuthProvider } from "firebase/auth";

const firebaseConfig = {
  apiKey: "AIzaSyAUZDQHF2PhuAdqPYNUhLZPy7b1WuAcMro",
  authDomain: "mealpro-development.firebaseapp.com",
  projectId: "mealpro-development",
  storageBucket: "mealpro-development.firebasestorage.app",
  messagingSenderId: "378316186332",
  appId: "1:378316186332:web:32260175dd5caa238aca19",
  measurementId: "G-CTZKMQB87V"
};



// Initialize Firebase
export const app = initializeApp(firebaseConfig);
export const auth = getAuth(app);

// Configure Google Provider for redirect
export const googleProvider = new GoogleAuthProvider();

// Add scopes
googleProvider.addScope('https://www.googleapis.com/auth/userinfo.email');
googleProvider.addScope('https://www.googleapis.com/auth/userinfo.profile');

// IMPORTANT: Set the redirect URL explicitly for Tauri
// In Tauri, we need to handle the redirect manually
googleProvider.setCustomParameters({
  prompt: 'select_account',
  redirect_uri: window.location.origin + window.location.pathname
});

export default app;