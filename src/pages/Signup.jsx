import React, { useState, useEffect } from "react";
import { Mail, Lock, Eye, EyeOff, User } from "lucide-react";
import { FcGoogle } from "react-icons/fc";
import { Link, useNavigate } from "react-router-dom";
import { createUserWithEmailAndPassword, updateProfile, GoogleAuthProvider, signInWithRedirect } from "firebase/auth";
import { auth } from "../libs/firebaseConfig";
import { invoke } from "@tauri-apps/api/core";
import { v4 as uuidv4 } from 'uuid';

export default function SignupPage() {
  const [fullName, setFullName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [agreeToTerms, setAgreeToTerms] = useState(false);
  
  const navigate = useNavigate();

  // Check if user is already logged in when component mounts
  useEffect(() => {
    console.log("🔄 SignupPage mounted - checking auth state");
    const user = auth.currentUser;
    if (user) {
      console.log("✅ User already logged in, redirecting to home");
      navigate("/home", { replace: true });
    }
  }, [navigate]);

  const handleEmailSignup = async (e) => {
    e.preventDefault();
    console.log("🖱️ Email signup button clicked");
    
    if (!fullName || !email || !password || !confirmPassword) {
      console.log("❌ Form validation failed - missing fields");
      alert("Please fill in all fields");
      return;
    }
    if (password !== confirmPassword) {
      console.log("❌ Passwords don't match");
      alert("Passwords don't match!");
      return;
    }
    if (!agreeToTerms) {
      console.log("❌ Terms not agreed");
      alert("Please agree to the Terms of Service and Privacy Policy");
      return;
    }
    
    console.log("✅ Form validation passed, starting Firebase signup...");
    setIsLoading(true);
    
    try {
      // Generate organization ID
      const organizationId = uuidv4();
      console.log("🏢 Generated organization ID:", organizationId);

      // Step 1: Create user in Firebase Auth
      console.log("🔥 Creating Firebase user...");
      const userCredential = await createUserWithEmailAndPassword(auth, email, password);
      const user = userCredential.user;
      
      // Step 2: Update profile with display name
      console.log("👤 Updating user profile...");
      await updateProfile(user, {
        displayName: fullName
      });
      
      // Step 3: Get ID token for backend authentication
      console.log("🔑 Getting ID token...");
      const idToken = await user.getIdToken();
      console.log("✅ Firebase user created:", user.uid);
      
      // Step 4: Send user data to Tauri backend using signup_user command
      console.log("🚀 Sending user data to backend...");
      await invoke('signup_user', { 
        firebaseToken: idToken,
        displayName: fullName,
        organizationId: organizationId
      });
      
      console.log("✅ Backend user creation successful");
      
      
      // Step 6: Store user data locally (include organizationId)
      const userData = {
        uid: user.uid,
        email: user.email,
        displayName: fullName,
        photoURL: user.photoURL,
        organizationId: organizationId
      };
      
      localStorage.setItem('user', JSON.stringify(userData));
      console.log("💾 User data stored locally");
      console.log("🏢 Organization ID stored:", organizationId);
      
      // Step 7: Navigate to home
      console.log("🔄 Navigating to /home...");
      navigate("/home", { replace: true });
      
    } catch (error) {
      console.error("❌ Email signup error:", error);
      console.error("❌ Error code:", error.code);
      console.error("❌ Error message:", error.message);
      
      // Handle specific Firebase errors
      if (error.code === 'auth/email-already-in-use') {
        alert("This email is already registered. Please use a different email or sign in.");
      } else if (error.code === 'auth/weak-password') {
        alert("Password is too weak. Please use a stronger password.");
      } else if (error.code === 'auth/invalid-email') {
        alert("Invalid email address. Please check your email.");
      } else if (error.code === 'auth/operation-not-allowed') {
        alert("Email/password sign-up is not enabled. Please contact support.");
      } else {
        // Handle backend errors (like user already exists in our database)
        if (error.toString().includes("User already exists")) {
          alert("User already exists. Please login instead.");
        } else {
          alert("Signup failed. Please try again.");
        }
      }
    } finally {
      setIsLoading(false);
    }
  };

  // For Google signup
  const handleGoogleSignup = async () => {
    try {
      console.log("🖱️ Google signup clicked");
      const organizationId = uuidv4();
      console.log("🏢 Generated organization ID for Google signup:", organizationId);
      
      // Store organization ID temporarily for when user returns from redirect
      localStorage.setItem('pending_organization_id', organizationId);
      
      const provider = new GoogleAuthProvider();
      await signInWithRedirect(auth, provider);
      console.log("✅ Redirecting to Google signup...");
    } catch (error) {
      console.error("❌ Google Signup Failed:", error);
      alert("Google signup failed. Please try again.");
    }
  };

  return (
    <div className="flex flex-col bg-white rounded-t-lg shadow-sm border border-gray-200" style={{ height: '680px' }}>
      {/* Header */}
      <div className="bg-white p-6 pb-0 flex-shrink-0">
        <div className="flex items-center gap-2 mb-2 justify-center">
          <div className="w-8 h-8 rounded-md bg-gradient-to-r from-blue-500 to-blue-400 flex items-center justify-center text-white text-sm font-semibold">
            ⌘
          </div>
          <h1 className="text-xl font-semibold text-gray-800">ClipTray</h1>
        </div>
        <p className="text-center text-sm text-gray-600">
          Create your account to get started
        </p>
      </div>

      {/* Form */}
      <div className="flex-1 p-6 pt-4">
        <form onSubmit={handleEmailSignup} className="space-y-4">
          {/* Full Name Field */}
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1.5">
              Full Name
            </label>
            <div className="relative">
              <User size={16} className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" />
              <input
                type="text"
                value={fullName}
                onChange={(e) => setFullName(e.target.value)}
                className="w-full h-10 pl-10 pr-3 border border-gray-300 rounded-lg bg-white text-gray-800 text-sm outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder="Enter your full name"
                required
              />
            </div>
          </div>

          {/* Email Field */}
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1.5">
              Email Address
            </label>
            <div className="relative">
              <Mail size={16} className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" />
              <input
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="w-full h-10 pl-10 pr-3 border border-gray-300 rounded-lg bg-white text-gray-800 text-sm outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder="Enter your email"
                required
              />
            </div>
          </div>

          {/* Password Fields */}
          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1.5">
              Password
            </label>
            <div className="relative">
              <Lock size={16} className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" />
              <input
                type={showPassword ? "text" : "password"}
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full h-10 pl-10 pr-10 border border-gray-300 rounded-lg bg-white text-gray-800 text-sm outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder="Create a password"
                required
                minLength={6}
              />
              <button
                type="button"
                onClick={() => setShowPassword(!showPassword)}
                className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-gray-600"
              >
                {showPassword ? <EyeOff size={16} /> : <Eye size={16} />}
              </button>
            </div>
          </div>

          <div>
            <label className="block text-xs font-medium text-gray-700 mb-1.5">
              Confirm Password
            </label>
            <div className="relative">
              <Lock size={16} className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" />
              <input
                type={showConfirmPassword ? "text" : "password"}
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                className="w-full h-10 pl-10 pr-10 border border-gray-300 rounded-lg bg-white text-gray-800 text-sm outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder="Confirm your password"
                required
                minLength={6}
              />
              <button
                type="button"
                onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-gray-600"
              >
                {showConfirmPassword ? <EyeOff size={16} /> : <Eye size={16} />}
              </button>
            </div>
          </div>

          {/* Terms Agreement */}
          <div className="flex items-start gap-2 text-xs">
            <input
              type="checkbox"
              checked={agreeToTerms}
              onChange={(e) => setAgreeToTerms(e.target.checked)}
              className="w-3 h-3 mt-0.5 text-blue-500 bg-gray-100 border-gray-300 rounded focus:ring-blue-500"
            />
            <label className="text-gray-700">
              I agree to the{" "}
              <button type="button" className="text-blue-500 hover:text-blue-600 font-medium">
                Terms of Service
              </button>{" "}
              and{" "}
              <button type="button" className="text-blue-500 hover:text-blue-600 font-medium">
                Privacy Policy
              </button>
            </label>
          </div>

          {/* Submit Button */}
          <button
            type="submit"
            disabled={!fullName || !email || !password || !confirmPassword || !agreeToTerms || isLoading}
            className="w-full h-10 bg-blue-500 text-white rounded-lg font-medium text-sm hover:bg-blue-600 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {isLoading ? (
              <div className="flex items-center justify-center gap-2">
                <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                Creating account...
              </div>
            ) : (
              "Create Account"
            )}
          </button>
        </form>

        {/* <div className="flex items-center my-6">
          <div className="flex-1 border-t border-gray-300"></div>
          <span className="px-3 text-xs text-gray-500 font-medium">OR</span>
          <div className="flex-1 border-t border-gray-300"></div>
        </div> */}

        {/* Google Signup Button */}
        {/* <div className="space-y-3">
          <button
            onClick={handleGoogleSignup}
            disabled={isLoading}
            className="w-full h-10 flex items-center justify-center gap-3 border border-gray-300 rounded-lg bg-white text-gray-700 text-sm font-medium hover:bg-gray-50 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            <FcGoogle size={18} />
            {isLoading ? "Redirecting..." : "Sign up with Google"}
          </button>
        </div> */}

        <div className="text-center mt-6 pt-4 border-t border-gray-200">
          <p className="text-xs text-gray-600">
            Already have an account?{" "}
            <Link to="/login" className="text-blue-500 hover:text-blue-600 font-medium">
              Sign in
            </Link>
          </p>
        </div>
      </div>

      <div className="p-4 text-center text-xs text-gray-400 bg-white flex-shrink-0 rounded-b-lg">
        Create with ❤️ by MakerStudio
      </div>
    </div>
  );
}