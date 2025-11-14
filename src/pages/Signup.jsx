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
  const [errors, setErrors] = useState({
    fullName: "",
    email: "",
    password: "",
    confirmPassword: "",
    terms: "",
    general: ""
  });
  
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

  const clearErrors = () => {
    setErrors({
      fullName: "",
      email: "",
      password: "",
      confirmPassword: "",
      terms: "",
      general: ""
    });
  };

  const handleEmailSignup = async (e) => {
    e.preventDefault();
    console.log("🖱️ Email signup button clicked");
    
    clearErrors();
    
    // Validation
    let hasErrors = false;
    const newErrors = {
      fullName: "",
      email: "",
      password: "",
      confirmPassword: "",
      terms: "",
      general: ""
    };

    if (!fullName) {
      newErrors.fullName = "Full name is required";
      hasErrors = true;
    }
    if (!email) {
      newErrors.email = "Email is required";
      hasErrors = true;
    }
    if (!password) {
      newErrors.password = "Password is required";
      hasErrors = true;
    }
    if (!confirmPassword) {
      newErrors.confirmPassword = "Please confirm your password";
      hasErrors = true;
    }
    if (password && confirmPassword && password !== confirmPassword) {
      newErrors.confirmPassword = "Passwords don't match";
      hasErrors = true;
    }
    if (!agreeToTerms) {
      newErrors.terms = "Please agree to the Terms of Service and Privacy Policy";
      hasErrors = true;
    }

    if (hasErrors) {
      setErrors(newErrors);
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
      let errorMessage = "Signup failed. Please try again.";
      
      if (error.code === 'auth/email-already-in-use') {
        setErrors({ ...errors, email: "This email is already registered. Please use a different email or sign in." });
      } else if (error.code === 'auth/weak-password') {
        setErrors({ ...errors, password: "Password is too weak. Please use a stronger password." });
      } else if (error.code === 'auth/invalid-email') {
        setErrors({ ...errors, email: "Invalid email address. Please check your email." });
      } else if (error.code === 'auth/operation-not-allowed') {
        setErrors({ ...errors, general: "Email/password sign-up is not enabled. Please contact support." });
      } else if (error.toString().includes("User already exists")) {
        setErrors({ ...errors, email: "User already exists. Please login instead." });
      } else {
        setErrors({ ...errors, general: errorMessage });
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
      setErrors({ ...errors, general: "Google signup failed. Please try again." });
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
        {/* General Error Message */}
        {errors.general && (
          <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg">
            <div className="text-xs text-red-500 flex items-center">
              <svg className="w-3 h-3 mr-2 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
              </svg>
              {errors.general}
            </div>
          </div>
        )}

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
                onChange={(e) => {
                  setFullName(e.target.value);
                  if (errors.fullName) clearErrors();
                }}
                className={`w-full h-10 pl-10 pr-3 border rounded-lg bg-white text-gray-800 text-sm outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent ${
                  errors.fullName ? 'border-red-300' : 'border-gray-300'
                }`}
                placeholder="Enter your full name"
                required
              />
            </div>
            {errors.fullName && (
              <div className="mt-1 text-xs text-red-500 flex items-center">
                <svg className="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
                </svg>
                {errors.fullName}
              </div>
            )}
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
                onChange={(e) => {
                  setEmail(e.target.value);
                  if (errors.email) clearErrors();
                }}
                className={`w-full h-10 pl-10 pr-3 border rounded-lg bg-white text-gray-800 text-sm outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent ${
                  errors.email ? 'border-red-300' : 'border-gray-300'
                }`}
                placeholder="Enter your email"
                required
              />
            </div>
            {errors.email && (
              <div className="mt-1 text-xs text-red-500 flex items-center">
                <svg className="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
                </svg>
                {errors.email}
              </div>
            )}
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
                onChange={(e) => {
                  setPassword(e.target.value);
                  if (errors.password || errors.confirmPassword) clearErrors();
                }}
                className={`w-full h-10 pl-10 pr-10 border rounded-lg bg-white text-gray-800 text-sm outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent ${
                  errors.password ? 'border-red-300' : 'border-gray-300'
                }`}
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
            {errors.password && (
              <div className="mt-1 text-xs text-red-500 flex items-center">
                <svg className="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
                </svg>
                {errors.password}
              </div>
            )}
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
                onChange={(e) => {
                  setConfirmPassword(e.target.value);
                  if (errors.confirmPassword) clearErrors();
                }}
                className={`w-full h-10 pl-10 pr-10 border rounded-lg bg-white text-gray-800 text-sm outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent ${
                  errors.confirmPassword ? 'border-red-300' : 'border-gray-300'
                }`}
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
            {errors.confirmPassword && (
              <div className="mt-1 text-xs text-red-500 flex items-center">
                <svg className="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
                </svg>
                {errors.confirmPassword}
              </div>
            )}
          </div>

          {/* Terms Agreement */}
          <div className="flex items-start gap-2 text-xs">
            <input
              type="checkbox"
              checked={agreeToTerms}
              onChange={(e) => {
                setAgreeToTerms(e.target.checked);
                if (errors.terms) clearErrors();
              }}
              className={`w-3 h-3 mt-0.5 text-blue-500 bg-gray-100 border rounded focus:ring-blue-500 ${
                errors.terms ? 'border-red-300' : 'border-gray-300'
              }`}
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
          {errors.terms && (
            <div className="text-xs text-red-500 flex items-center">
              <svg className="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
              </svg>
              {errors.terms}
            </div>
          )}

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