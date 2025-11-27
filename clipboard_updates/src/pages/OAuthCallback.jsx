import React, { useEffect } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';

export default function OAuthCallback() {
  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    const handleCallback = async () => {
      try {
        console.log('🔄 Handling Google OAuth callback...');
        
        // Parse URL parameters
        const urlParams = new URLSearchParams(location.search);
        const code = urlParams.get('code');
        const state = urlParams.get('state');
        const error = urlParams.get('error');

        console.log('📋 Callback parameters:', { code, state, error });

        if (error) {
          console.error('❌ OAuth error:', error);
          navigate('/login?error=oauth_failed');
          return;
        }

        if (!code) {
          console.error('❌ Missing authorization code');
          navigate('/login?error=invalid_callback');
          return;
        }

        console.log('✅ Valid callback received, processing with backend...');

        // Call backend to handle the callback
        const result = await invoke('handle_google_oauth_callback', {
          callback: {
            code: code,
            state: state || ''
          }
        });

        console.log('✅ Google OAuth successful:', result);

        if (result.success && result.user) {
          // Store user data in localStorage
          const userData = {
            uid: result.user.uid || result.user.firebase_uid,
            email: result.user.email,
            displayName: result.user.displayName || result.user.display_name,
            organizationId: result.user.organization_id,
            photoURL: result.google_user_info?.picture || null,
            // Include all user data for consistency
            ...result.user
          };
          
          localStorage.setItem('user', JSON.stringify(userData));
          if (result.access_token) {
            localStorage.setItem('google_access_token', result.access_token);
          }
          
          // Clean up OAuth state
          localStorage.removeItem('oauth_state');
          
          console.log('💾 User data stored:', userData);
          console.log('🏢 Organization ID:', userData.organizationId);
          
          // Navigate to home
          navigate('/home', { replace: true });
        } else {
          throw new Error('Invalid response from server');
        }

      } catch (error) {
        console.error('❌ OAuth callback failed:', error);
        navigate('/login?error=oauth_failed');
      }
    };

    handleCallback();
  }, [location, navigate]);

  return (
    <div className="flex items-center justify-center min-h-screen bg-gray-50">
      <div className="text-center p-8 bg-white rounded-lg shadow-sm border border-gray-200">
        <div className="w-12 h-12 border-4 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
        <h3 className="text-lg font-medium text-gray-800 mb-2">Completing Sign In</h3>
        <p className="text-gray-600">Please wait while we finish setting up your account...</p>
      </div>
    </div>
  );
}