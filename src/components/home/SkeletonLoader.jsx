// src/components/SkeletonLoader.jsx
import React from "react";

export function SkeletonClipItem() {
  return (
    <div className="bg-white p-1.5 rounded-md shadow-sm border border-gray-100 animate-pulse">
      {/* Content skeleton */}
      <div className="space-y-1.5 mb-2">
        <div className="h-3 bg-gray-200 rounded w-3/4"></div>
        <div className="h-3 bg-gray-200 rounded w-1/2"></div>
      </div>
      
      {/* Tags skeleton */}
      <div className="flex gap-1 mb-2">
        <div className="w-12 h-5 bg-gray-200 rounded-full"></div>
        <div className="w-16 h-5 bg-gray-200 rounded-full"></div>
      </div>
      
      {/* Footer skeleton */}
      <div className="flex justify-between items-center">
        <div className="w-16 h-3 bg-gray-200 rounded"></div>
        <div className="flex gap-1">
          <div className="w-10 h-6 bg-gray-200 rounded-md"></div>
          <div className="w-6 h-6 bg-gray-200 rounded-full"></div>
        </div>
      </div>
    </div>
  );
}

export function SkeletonHeader() {
  return (
    <div className="bg-white p-2 flex-shrink-0 animate-pulse">
      <div className="flex items-center justify-between mb-1">
        <div className="flex items-center gap-2">
          <div className="w-5 h-5 bg-gray-300 rounded-md"></div>
          <div className="w-20 h-4 bg-gray-300 rounded"></div>
        </div>
        <div className="w-16 h-6 bg-gray-300 rounded-md"></div>
      </div>
      <div className="w-full h-6 bg-gray-200 rounded-md"></div>
    </div>
  );
}

export function SkeletonTags() {
  return (
    <div className="bg-white px-2 pt-0.5 flex-shrink-0 animate-pulse">
      <div className="flex items-center gap-1 overflow-x-auto pb-0.5">
        {[...Array(5)].map((_, i) => (
          <div key={i} className="w-12 h-6 bg-gray-200 rounded-full"></div>
        ))}
      </div>
    </div>
  );
}