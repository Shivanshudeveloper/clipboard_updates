// src/components/ProtectedRoute.jsx
import { useAuth } from '../contexts/AuthContext';
import { Navigate } from 'react-router-dom';

export default function ProtectedRoute({ children }) {
  const { user, loading } = useAuth();

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="flex items-center gap-2">
          <div className="w-4 h-4 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
          <span className="text-sm text-gray-600">Loading...</span>
        </div>
      </div>
    );
  }

  if (!user) {
    console.log("🚫 No user, redirecting to login");
    return <Navigate to="/login" replace />;
  }

  console.log("✅ User authenticated, rendering protected content");
  return children;
}