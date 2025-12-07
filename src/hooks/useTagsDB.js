import { useState, useCallback } from 'react';  // Add this line
import { invoke } from '@tauri-apps/api/core';

export function useTagsDB() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [initialLoad, setInitialLoad] = useState(true);

// In useTagsDB.js - update the getTags function
const getTags = useCallback(async (localItems = []) => {

  try {

    setLoading(true);
    setError(null);

    // Fetch tags from the backend
    const tags = await invoke('get_tags');
    
    // If no localItems provided, return unsorted tags
    if (!localItems || localItems.length === 0) {
      setInitialLoad(false);
      return tags;
    }
    
    // Calculate tag content count
    const tagContentCount = {};
    
    localItems.forEach((item) => {
      if (item.tags) {
        let tagsArray = Array.isArray(item.tags) ? item.tags : [];

        if (typeof item.tags === 'string') {
          try {
            tagsArray = JSON.parse(item.tags);
          } catch (e) {
            console.error("Error parsing tags string:", e);
            tagsArray = [];
          }
        }

        tagsArray.forEach((tagName) => {
          tagContentCount[tagName] = (tagContentCount[tagName] || 0) + 1;
        });
      }
    });

    // Sort tags by content count (descending order)
    const sortedTags = [...tags].sort((a, b) => {
      const countA = tagContentCount[a.name] || 0;
      const countB = tagContentCount[b.name] || 0;
      return countB - countA; // Sort in decreasing order
    });

    console.log("📊 Tags sorted by content count:", 
      sortedTags.map(t => `${t.name}: ${tagContentCount[t.name] || 0}`));

    setInitialLoad(false);
    return sortedTags;
  } catch (err) {
    const errorMessage = err instanceof Error ? err.message : 'Unknown error';
    setError(errorMessage);
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


const deleteTag = useCallback(async (dbId) => {
  try {
    setLoading(true);
    setError(null);

    const success = await invoke("delete_tag", { tagId: dbId });

    return success;
  } catch (err) {
    console.error("Error deleting tag:", err);
    setError(err.message || "Failed to delete tag");
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
