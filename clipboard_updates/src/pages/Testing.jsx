// src/components/SimpleClipboardViewer.jsx
import React, { useState, useEffect } from 'react';
import { useClipboardDB } from '../hooks/useClipboardDB';

const Testing = () => {
  const [entries, setEntries] = useState([]);
  const { getClipboardEntries, loading, error } = useClipboardDB();

  const loadEntries = async () => {
    const data = await getClipboardEntries(50);
    setEntries(data || []);
  };

  useEffect(() => {
    loadEntries();
  }, []);

  const formatDate = (timestamp) => {
    return new Date(timestamp).toLocaleString();
  };

  const truncateText = (text, maxLength = 100) => {
    if (!text) return 'Empty content';
    if (text.length <= maxLength) return text;
    return text.substring(0, maxLength) + '...';
  };

  if (error) {
    return (
      <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
        <div className="text-red-600 font-semibold">Error loading entries</div>
        <div className="text-red-500 text-sm mt-1">{error}</div>
        <button
          onClick={loadEntries}
          className="mt-2 bg-red-600 text-white px-3 py-1 rounded text-sm hover:bg-red-700"
        >
          Try Again
        </button>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow-md p-6">
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-2xl font-bold text-gray-800">Clipboard Entries</h2>
        <button
          onClick={loadEntries}
          disabled={loading}
          className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 disabled:bg-gray-400 transition-colors"
        >
          {loading ? 'Loading...' : 'Refresh'}
        </button>
      </div>

      {loading && entries.length === 0 ? (
        <div className="space-y-4">
          {[...Array(5)].map((_, i) => (
            <div key={i} className="animate-pulse">
              <div className="h-4 bg-gray-200 rounded mb-2"></div>
              <div className="h-3 bg-gray-200 rounded w-3/4"></div>
            </div>
          ))}
        </div>
      ) : (
        <div className="space-y-4 max-h-96 overflow-y-auto">
          {entries.map((entry) => (
            <div
              key={entry.id}
              className="border border-gray-200 rounded-lg p-4 hover:bg-gray-50 transition-colors"
            >
              <div className="flex justify-between items-start mb-2">
                <span className="text-xs font-medium text-gray-500 bg-gray-100 px-2 py-1 rounded">
                  #{entry.id}
                </span>
                <span className="text-xs text-gray-400">
                  {formatDate(entry.timestamp)}
                </span>
              </div>
              
              <div className="text-gray-800 mb-2">
                {truncateText(entry.content)}
              </div>
              
              <div className="flex gap-2">
                {entry.is_pinned && (
                  <span className="text-xs bg-yellow-100 text-yellow-800 px-2 py-1 rounded">
                    📌 Pinned
                  </span>
                )}
                <span className="text-xs text-gray-500 capitalize">
                  {entry.content_type || 'text'}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}

      {!loading && entries.length === 0 && (
        <div className="text-center py-8 text-gray-500">
          No clipboard entries found
        </div>
      )}

      {!loading && entries.length > 0 && (
        <div className="mt-4 text-sm text-gray-500 text-center">
          Showing {entries.length} entries
        </div>
      )}
    </div>
  );
};

export default Testing;
