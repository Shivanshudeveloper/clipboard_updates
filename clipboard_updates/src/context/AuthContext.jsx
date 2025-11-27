// src/contexts/AuthContext.jsx
import React, { createContext, useContext, useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { 
  getCurrentUser, 
  handleRedirectResult, 
  signOutUser 
} from '../libs/firebaseAuth';

const AuthContext = createContext();

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}

export function AuthProvider({ children }) {
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(true);
  const [isValidating, setIsValidating] = useState(false);

  // Validate session with Tauri backend
  const validateTauriSession = async () => {
    try {
      setIsValidating(true);
      console.log('🔄 Validating Tauri session...');
      
      const result = await invoke('validate_session');
      console.log('📋 Tauri session validation result:', result);
      
      if (result && result.user_id) {
        console.log('✅ Tauri session is valid');
        return true;
      } else {
        console.log('❌ Tauri session is invalid');
        return false;
      }
    } catch (error) {
      console.error('❌ Tauri session validation failed:', error);
      return false;
    } finally {
      setIsValidating(false);
    }
  };

  // Check if we're in Tauri environment
  const isTauri = () => {
    return window.__TAURI__ !== undefined;
  };

  useEffect(() => {
    const initializeAuth = async () => {
      try {
        setLoading(true);

        // First, check if we have a redirect result (user just came back from Google)
        const redirectResult = await handleRedirectResult();
        if (redirectResult) {
          console.log('✅ User authenticated via redirect');
          setUser(redirectResult.user);
          setLoading(false);
          return;
        }

        // If no redirect result, check for existing user in Firebase
        const firebaseUser = getCurrentUser();
        
        if (firebaseUser && isTauri()) {
          // We have a Firebase user, but need to validate Tauri session
          console.log('🔍 Firebase user found, validating Tauri session...');
          const isTauriSessionValid = await validateTauriSession();
          
          if (isTauriSessionValid) {
            console.log('✅ Both Firebase and Tauri sessions are valid');
            setUser(firebaseUser);
          } else {
            console.log('🚫 Tauri session invalid, clearing auth state');
            // Tauri session is invalid, log out user
            await signOutUser();
            setUser(null);
          }
        } else if (firebaseUser && !isTauri()) {
          // Web environment, just use Firebase
          console.log('🌐 Web environment, using Firebase auth only');
          setUser(firebaseUser);
        } else {
          // No user at all
          console.log('👤 No user found, not logged in');
          setUser(null);
        }
      } catch (error) {
        console.error('❌ Auth initialization failed:', error);
        setUser(null);
      } finally {
        setLoading(false);
      }
    };

    initializeAuth();
  }, []);

  const login = (userData) => {
    setUser(userData);
  };

  const logout = async () => {
    try {
      if (isTauri()) {
        // Log out from Tauri backend
        await invoke('logout_user');
      }
      await signOutUser();
      setUser(null);
    } catch (error) {
      console.error("❌ Logout failed:", error);
    }
  };

  const value = {
    user,
    setUser, // ADD THIS LINE - expose setUser to consumers
    login,
    logout,
    loading,
    isValidating,
    validateSession: validateTauriSession
  };

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  );
}