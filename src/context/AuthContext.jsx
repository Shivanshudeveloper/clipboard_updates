// src/contexts/AuthContext.jsx
import React, { createContext, useContext, useState, useEffect } from 'react';
import { 
  getCurrentUser, 
  handleRedirectResult, 
  handleSignOut 
} from '../libs/firebaseAuth';

const AuthContext = createContext();

export function useAuth() {
  return useContext(AuthContext);
}

export function AuthProvider({ children }) {
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    
    const initializeAuth = async () => {
      try {
        // First, check if we have a redirect result (user just came back from Google)
        const redirectResult = await handleRedirectResult();
        if (redirectResult) {
          setUser(redirectResult.user);
          setLoading(false);
          return;
        }

        // If no redirect result, check for existing user
        const currentUser = getCurrentUser();
        if (currentUser) {
          setUser(currentUser);
        } else {
        }
      } catch (error) {
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
      await handleSignOut();
      setUser(null);
    } catch (error) {
      console.error("❌ Logout failed:", error);
    }
  };

  const value = {
    user,
    login,
    logout,
    loading
  };

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  );
}