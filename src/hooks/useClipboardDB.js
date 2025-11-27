// src/hooks/useClipboardDB.js
import { invoke } from '@tauri-apps/api/core';
import { useState, useCallback, useEffect, useRef } from 'react';

export const useClipboardDB = () => {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [initialLoad, setInitialLoad] = useState(true);

  // Get all clipboard entries
  const getClipboardEntries = useCallback(async (limit = 100) => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke('get_my_entries', { 
        limit: limit || undefined
      });
      setInitialLoad(false);
      return data;
    } catch (err) {
      setError(err.message);
      console.error('Failed to get clipboard entries:', err);
      return [];
    } finally {
      setLoading(false);
    }
  }, []);
  const resetInitialLoad = useCallback(() => {
    setInitialLoad(true);
  }, []);

  // Get recent entries (last N hours)
  const getRecentEntries = useCallback(async (hours = 24) => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke('get_recent_entries', { 
        hours: hours || undefined 
      });
      return data;
    } catch (err) {
      setError(err.message);
      console.error('Failed to get recent entries:', err);
      return [];
    } finally {
      setLoading(false);
    }
  }, []);

  // Get entry by ID
  const getEntryById = useCallback(async (id) => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke('get_entry_by_id', { id });
      return data;
    } catch (err) {
      setError(err.message);
      console.error('Failed to get entry:', err);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  // Search entries
  const searchEntries = useCallback(async (query) => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke('search_entries', { query });
      return data;
    } catch (err) {
      setError(err.message);
      console.error('Failed to search entries:', err);
      return [];
    } finally {
      setLoading(false);
    }
  }, []);

  // Delete entry
  const deleteEntry = useCallback(async (id) => {
    setLoading(true);
    setError(null);
    try {
      console.log("[useClipboardDB] deleteEntry called with id:", id);
      const coercedId = (typeof id === "string" && /^\d+$/.test(id)) ? Number(id) : id;
      const result = await invoke('delete_entry', { id: coercedId });
      console.log("[useClipboardDB] delete_entry result:", result);
      return result;
    } catch (err) {
      setError(err.message);
      console.error('Failed to delete entry:', err);
      return false;
    } finally {
      setLoading(false);
    }
  }, []);

  // Update entry
  const updateEntry = useCallback(async (id, updates) => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke('update_entry', {
        id,
        updates,
      });
      return data;
    } catch (err) {
      setError(err.message);
      console.error('Failed to update entry:', err);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  // Update entry content
  const updateEntryContent = useCallback(async (id, newContent) => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke('update_entry_content', {
        id,
        newContent,
      });
      return data;
    } catch (err) {
      setError(err.message);
      console.error('Failed to update entry content:', err);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  // Polling function for real-time updates
  const startPolling = useCallback((callback, interval = 1000) => {
    let isMounted = true;
    let timeoutId;

    const poll = async () => {
      if (!isMounted) return;

      try {
        const data = await getClipboardEntries();
        if (isMounted) {
          callback(data);
        }
      } catch (err) {
        console.error('Polling error:', err);
      } finally {
        if (isMounted) {
          timeoutId = setTimeout(poll, interval);
        }
      }
    };

    // Start polling
    poll();

    // Cleanup function
    return () => {
      isMounted = false;
      if (timeoutId) {
        clearTimeout(timeoutId);
      }
    };
  }, [getClipboardEntries]);

  return {
    loading,
    error,
    initialLoad,
    getClipboardEntries,
    getRecentEntries,
    searchEntries,
    getEntryById,
    updateEntry,
    updateEntryContent,
    deleteEntry,
    startPolling, // Add polling function
  };
};