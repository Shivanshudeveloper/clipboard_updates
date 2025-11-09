// hooks/useTagsDB.js
import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export function useTagsDB() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [initialLoad, setInitialLoad] = useState(true);

  const getTags = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const tags = await invoke('get_organization_tags');
      setInitialLoad(false);
      return tags;
    } catch (err) {
      setError(err.message);
      setInitialLoad(false);
      console.error('Error fetching tags:', err);
      return [];
    } finally {
      setLoading(false);
    }
  }, []);
  const resetInitialLoad = useCallback(() => {
    setInitialLoad(true);
  }, []);

  const createTag = useCallback(async (tagData) => {
    try {
      setLoading(true);
      setError(null);
      const createdTag = await invoke('create_tag', {
        name: tagData.name,
        color: tagData.color
      });
      return createdTag;
    } catch (err) {
      setError(err.message);
      console.error('Error creating tag:', err);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const deleteTag = useCallback(async (tagId) => {
    try {
      setLoading(true);
      setError(null);
      const success = await invoke('delete_tag', { tagId });
      return success;
    } catch (err) {
      setError(err.message);
      console.error('Error deleting tag:', err);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  return {
    getTags,
    createTag,
    initialLoad,
    resetInitialLoad,
    deleteTag,
    loading,
    error,
  };
}