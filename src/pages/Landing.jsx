import React, { useEffect, useMemo, useRef, useState } from "react";
import {
  Search, Copy, MoreHorizontal, X, Plus, LogOut, Settings
} from "lucide-react";
import { useClipboardDB } from "../hooks/useClipboardDB";
import { useTagsDB } from "../hooks/useTagsDB"; // Add this import
import { INITIAL_TAGS } from "../mock/data";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentUser, signOutUser } from "../libs/firebaseAuth";
import { useNavigate,Link } from "react-router-dom";

function isTauri() {
  return "__TAURI__" in window;
}

export default function ClipTray() {
  const {
    getClipboardEntries,
    updateEntryContent,
    deleteEntry,
    startPolling
  } = useClipboardDB();

  // Add tags hook
  const {
    getTags,
    createTag,
    deleteTag: deleteTagBackend,
    loading: tagsLoading,
    error: tagsError
  } = useTagsDB();

  const [localItems, setLocalItems] = useState([]);
  const [isLoggingOut, setIsLoggingOut] = useState(false);
  const navigate = useNavigate();

  // ✅ Auto-fetch with polling - no manual refresh needed
  useEffect(() => {
    if (!isTauri()) return;
    
    const loadEntries = async () => {
      try {
        const data = await getClipboardEntries(100);
        if (Array.isArray(data)) setLocalItems(data);
      } catch (err) {
        console.error("Error loading entries:", err);
      }
    };

    // Load entries initially
    loadEntries();

    // Start polling for real-time updates every 3 seconds
    const cleanup = startPolling((newEntries) => {
      if (Array.isArray(newEntries)) {
        setLocalItems(newEntries);
      }
    }, 3000);

    return cleanup; // Cleanup on unmount
  }, [getClipboardEntries, startPolling]);

  const [q, setQ] = useState("");
  const [tags, setTags] = useState([]); // Start with empty array, fetch from backend
  const [menu, setMenu] = useState(null);
  const [activeTag, setActiveTag] = useState("all");
  const [tagDropdown, setTagDropdown] = useState(null);
  const [createTagModal, setCreateTagModal] = useState(false);
  const [newTagName, setNewTagName] = useState("");
  const [pinnedItems, setPinnedItems] = useState(new Set());
  const [itemTags, setItemTags] = useState({});

  // ✅ Fetch tags from backend on component mount
  useEffect(() => {
    const loadTags = async () => {
      try {
        const backendTags = await getTags();
        if (Array.isArray(backendTags)) {
          setTags(backendTags);
        }
      } catch (err) {
        console.error("Error loading tags:", err);
        // Fallback to initial tags if backend fails
        setTags(INITIAL_TAGS);
      }
    };

    loadTags();
  }, [getTags]);

  // ✅ Update pinned items from database data
  useEffect(() => {
    const pinnedIds = new Set();
    localItems.forEach(item => {
      if (item.is_pinned) {
        pinnedIds.add(item.id);
      }
    });
    setPinnedItems(pinnedIds);
  }, [localItems]);

  // ✅ Adapt DB data to your UI
const items = useMemo(() => {
  return localItems.map((item, index) => {
    
    // Parse tags from database
    let tagsArray = [];
    
    if (item.tags) {
      if (typeof item.tags === 'string') {
        try {
          // Handle JSON string like "[\"Work\", \"Important\"]"
          let cleanTags = item.tags.trim();
          
          // Remove extra backslashes if they exist
          cleanTags = cleanTags.replace(/\\/g, '');
          
          // Parse the JSON
          tagsArray = JSON.parse(cleanTags);
        } catch (e) {
          console.error("Error parsing tags JSON:", e, "Raw tags:", item.tags);
          tagsArray = [];
        }
      } else if (Array.isArray(item.tags)) {
        // If it's already an array, use it directly
        tagsArray = item.tags;
      }
    }
    
    const processedItem = {
      id: item.id || `${item.timestamp}-${index}`,
      content: item.text || item.content || "",
      timestamp: item.timestamp,
      content_type: item.content_type || "text",
      source_app: item.source_app || "Unknown",
      source_window: item.source_window || "",
      pinned: item.is_pinned || pinnedItems.has(item.id),
      tags: tagsArray
    };
    
    return processedItem;
  });
}, [localItems, pinnedItems]);

 

// Update the filtered items logic to work with tag names
const filtered = useMemo(() => {
  const s = q.trim().toLowerCase();
  let filteredItems = items;
  
  // Search filter
  if (s) {
    filteredItems = filteredItems.filter(x => x.content.toLowerCase().includes(s));
  }
  
  // Tag filter - now using tag names
  if (activeTag !== "all") {
    filteredItems = filteredItems.filter(item => 
      item.tags && item.tags.includes(activeTag) // Check if item has this tag name
    );
  }
  
 
  
  return filteredItems;
}, [q, items, activeTag]);

  const pinned = filtered.filter(x => x.pinned);
  const recent = filtered.filter(x => !x.pinned);

  // UI refs
  const menuRef = useRef(null);
  const tagDropdownRef = useRef(null);
  const createTagModalRef = useRef(null);

  useEffect(() => {
    function onKey(e) {
      if (e.key === "Escape") {
        setMenu(null);
        setTagDropdown(null);
        setCreateTagModal(false);
      }
    }
    function onClick(e) {
      if (menuRef.current && menu && !menuRef.current.contains(e.target)) setMenu(null);
      if (tagDropdownRef.current && tagDropdown && !tagDropdownRef.current.contains(e.target)) setTagDropdown(null);
      if (createTagModalRef.current && createTagModal && !createTagModalRef.current.contains(e.target)) setCreateTagModal(false);
    }
    window.addEventListener("keydown", onKey);
    window.addEventListener("mousedown", onClick);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("mousedown", onClick);
    };
  }, [menu, tagDropdown, createTagModal]);

  // Logout function
  const handleLogout = async () => {
    try {
      setIsLoggingOut(true);
      await signOutUser();
      const result = await invoke('logout_user');
    console.log(result);
    
    localStorage.removeItem('user');
    sessionStorage.clear();
      navigate("/login");
    } catch (error) {
      console.error("Logout failed:", error);
      alert("Logout failed. Please try again.");
    } finally {
      setIsLoggingOut(false);
    }
  };

  const togglePin = async (id) => {
    const currentItem = items.find(x => x.id === id);
    const newPinnedState = !currentItem?.pinned;
    
    try {
      await invoke("update_entry", { 
        id: id,
        updates: { is_pinned: newPinnedState }
      });
      
      setPinnedItems(prev => {
        const newPinned = new Set(prev);
        if (newPinnedState) {
          newPinned.add(id);
        } else {
          newPinned.delete(id);
        }
        return newPinned;
      });
      
    } catch (err) {
      console.error("Failed to update pin state:", err);
    }
    
    setMenu(null);
  };

  const editItem = async (id) => {
    const current = items.find(x => x.id === id);
    if (!current) return;
    try {
      const edited = await invoke("open_in_notepad_and_wait", { content: current.content });
      if (edited && edited.trim() !== current.content.trim()) {
        await updateEntryContent(id, edited);
        setLocalItems(prev => prev.map(i => (i.id === id ? { ...i, text: edited } : i)));
        await navigator.clipboard.writeText(edited);
        alert("✅ Edited content updated!");
      }
    } catch (err) {
      console.error("Error editing content:", err);
    }
    setMenu(null);
  };

  const deleteItem = async (id) => {
    try {
      await deleteEntry(id);
      setLocalItems(prev => prev.filter(x => x.id !== id));
    } catch (err) {
      console.error("Failed to delete entry:", err);
    }
    setMenu(null);
  };

  const copyToClipboard = (text) => navigator.clipboard.writeText(text);

  // ✅ Updated createNewTag to use backend
  const createNewTag = async () => {
    if (!newTagName.trim()) return;
    
    try {
      const newTag = await createTag({
        name: newTagName.trim(),
        color: `#${Math.floor(Math.random() * 16777215).toString(16).padStart(6, "0")}`,
      });
      
      if (newTag) {
        setTags(prev => [...prev, newTag]);
        setNewTagName("");
        setCreateTagModal(false);
      }
    } catch (err) {
      console.error("Failed to create tag:", err);
      alert("Failed to create tag: " + err.message);
    }
  };

const assignTagToItem = async (itemId, tagId) => {
  try {
    // First, find the tag name from the tag ID
    const tag = tags.find(t => t.id === tagId);
    if (!tag) {
      console.error("Tag not found with ID:", tagId);
      return;
    }

    const updatedEntry = await invoke("assign_tag_to_entry", {
      clipboardEntryId: parseInt(itemId),
      tagName: tag.name // Pass the tag name, not the ID
    });
    
    setLocalItems(prev => prev.map(item => 
      item.id === parseInt(itemId) ? { 
        ...item, 
        tags: updatedEntry.tags 
      } : item
    ));
    
    
  } catch (err) {
    console.error("Failed to assign tag:", err);
    alert("Failed to assign tag: " + err);
  }
};

const removeTagFromItem = async (itemId, tagName, e) => {
  e.stopPropagation();
  
  try {
    console.log("🔴 REMOVING tag:", tagName, "from item:", itemId);
    
    const updatedEntry = await invoke("remove_tag_from_entry", {
      clipboardEntryId: parseInt(itemId),
      tagName: tagName
    });
    
    console.log("✅ Remove response:", updatedEntry);
    
    setLocalItems(prev => prev.map(item => 
      item.id === parseInt(itemId) ? { 
        ...item, 
        tags: updatedEntry.tags 
      } : item
    ));
    
  } catch (err) {
    console.error("❌ Failed to remove tag:", err);
  }
};



  // ✅ Updated deleteTag to use backend
  const handleDeleteTag = async (tagId) => {
    try {
      const success = await deleteTagBackend(tagId);
      if (success) {
        setItemTags(prev => {
          const newItemTags = { ...prev };
          Object.keys(newItemTags).forEach(itemId => {
            newItemTags[itemId] = newItemTags[itemId].filter(id => id !== tagId);
          });
          return newItemTags;
        });
        setTags(prev => prev.filter(tag => tag.id !== tagId));
        if (activeTag === tagId.toString()) setActiveTag("all");
      }
    } catch (err) {
      console.error("Failed to delete tag:", err);
      alert("Failed to delete tag: " + err.message);
    }
  };

  const formatTime = (timestamp) => {
    const now = Date.now();
    const diff = now - new Date(timestamp).getTime();
    const mins = Math.floor(diff / (1000 * 60));
    const hours = Math.floor(diff / (1000 * 60 * 60));
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    if (mins < 60) return `${mins} min ago`;
    if (hours < 24) return `${hours} hour${hours > 1 ? "s" : ""} ago`;
    return `${days} day${days > 1 ? "s" : ""} ago`;
  };

  const getTagById = (tagId) => tags.find(tag => tag.id.toString() === tagId.toString());

  const openTagDropdownForMenu = (itemId) => {
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;
    const dropdownWidth = 176;
    const dropdownHeight = 200;
    const x = (viewportWidth - dropdownWidth) / 2;
    const y = (viewportHeight - dropdownHeight) / 2;
    setTagDropdown({
      itemId,
      x: Math.max(10, x),
      y: Math.max(10, y)
    });
  };

  return (
    <div className="flex flex-col bg-white relative" style={{ height: '565px'}}>
      {/* Header - With logout button */}
      <div className="bg-white p-2 flex-shrink-0">
        <div className="flex items-center justify-between mb-1">
          <div className="flex items-center gap-2">
            <div className="w-5 h-5 rounded-md bg-gradient-to-r from-blue-500 to-blue-400 flex items-center justify-center text-white text-xs font-semibold">
              ⌘
            </div>
            <h1 className="text-sm font-semibold text-gray-800">ClipTray</h1>


          </div>
          <div className="flex gap-2 ">
            <div className="mt-1">
              <Link to="/settings">
          <Settings size={18} className="text-gray-600" />
          </Link>
          </div>
          {/* Logout Button */}
          <button
            onClick={handleLogout}
            disabled={isLoggingOut}
            className="flex items-center gap-1 px-2 py-1 text-xs text-gray-600 bg-white border border-gray-300 rounded-md hover:bg-gray-50 hover:border-gray-400 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            title="Logout"
          >
            {isLoggingOut ? (
              <div className="w-3 h-3 border-2 border-gray-400 border-t-transparent rounded-full animate-spin" />
            ) : (
              <LogOut size={12} />
            )}
            Logout
          </button>
          </div>
        </div>
        
        <div className="relative">
          <Search size={12} className="absolute left-2 top-1/2 transform -translate-y-1/2 text-gray-400" />
          <input
            className="w-full h-6 pl-7 pr-2 border border-gray-300 rounded-md bg-gray-50 text-gray-800 text-xs outline-none focus:ring-1 focus:ring-blue-500"
            placeholder="Search clips"
            value={q}
            onChange={(e) => {
              if (e.target.checked) {
                assignTagToItem(tagDropdown.itemId, tag.name); // Pass tag name directly
              } else {
                removeTagFromItem(tagDropdown.itemId, tag.name, e); // Pass tag name directly
              }
            }}          />
        </div>
      </div>

      {/* Tags */}
<div className="bg-white px-2 pt-0.5 flex-shrink-0">
  <div className="flex items-center gap-1 overflow-x-auto pb-0.5">
    <button
      className={`flex items-center gap-0.5 py-0.5 px-1.5 text-xs font-medium rounded-full border transition-all whitespace-nowrap ${
        activeTag === "all" 
          ? "bg-blue-500 text-white border-transparent" 
          : "bg-white text-gray-700 border-gray-300 hover:border-gray-400"
      }`}
      onClick={() => setActiveTag("all")}
    >
      All
    </button>
    
    {tagsLoading ? (
      <div className="text-xs text-gray-500">Loading tags...</div>
    ) : (
      tags.map(tag => (
        <button
          key={tag.id}
          className={`flex items-center gap-0.5 py-0.5 px-1.5 text-xs font-medium rounded-full border transition-all whitespace-nowrap ${
            activeTag === tag.name // Use tag.name instead of tag.id.toString()
              ? "text-white border-transparent" 
              : "bg-white text-gray-700 border-gray-300 hover:border-gray-400"
          }`}
          style={{
            backgroundColor: activeTag === tag.name ? tag.color : 'transparent',
            borderColor: activeTag === tag.name ? tag.color : ''
          }}
          onClick={() => setActiveTag(tag.name)} // Set to tag name
        >
          {tag.name}
        </button>
      ))
    )}
  </div>
  {tagsError && (
    <div className="text-xs text-red-500 mt-1">{tagsError}</div>
  )}
</div>

      {/* Content Area */}
      <div className="flex-1 overflow-hidden flex flex-col p-2">
        {/* Pinned Section */}
        <div className="mb-2 flex flex-col" style={{ height: '50%', minHeight: '40%' }}>
          <div className="flex justify-between items-center p-2 text-xs font-semibold text-gray-500 uppercase tracking-wider flex-shrink-0">
            <span>Pinned</span>
            <span className="bg-gray-100 text-gray-500 text-xs font-semibold py-0.5 px-1.5 rounded-full">
              {pinned.length}
            </span>
          </div>
          <div className="flex-1 overflow-y-auto min-h-0">
            {pinned.length > 0 ? (
              <div className="space-y-0.5 pr-0.5">
                {pinned.map((item) => (
                  <ClipItem 
                    key={item.id} 
                    item={item} 
                    tags={tags}
                    getTagById={getTagById}
                    onCopy={copyToClipboard}
                    onMenuOpen={setMenu}
                    onTagClick={openTagDropdownForMenu}
                    onRemoveTag={removeTagFromItem}
                    formatTime={formatTime}
                  />
                ))}
              </div>
            ) : (
              <div className="flex items-center justify-center h-full text-gray-400 text-xs">
                No pinned clips - Right click items and select "Pin"
              </div>
            )}
          </div>
        </div>

        {/* Recent Section */}
        <div className="flex flex-col" style={{ height: '40%', minHeight: '50%' }}>
          <div className="flex justify-between items-center p-2 text-xs font-semibold text-gray-500 uppercase tracking-wider flex-shrink-0">
            <span>Recent</span>
            <span className="bg-gray-100 text-gray-500 text-xs font-semibold py-0.5 px-1.5 rounded-full">
              {recent.length}
            </span>
          </div>
          <div className="flex-1 overflow-y-auto min-h-0">
            {recent.length > 0 ? (
              <div className="space-y-0.5 pr-0.5">
                {recent.map((item) => (
                  <ClipItem 
                    key={item.id} 
                    item={item} 
                    tags={tags}
                    getTagById={getTagById}
                    onCopy={copyToClipboard}
                    onMenuOpen={setMenu}
                    onTagClick={openTagDropdownForMenu}
                    onRemoveTag={removeTagFromItem}
                    formatTime={formatTime}
                  />
                ))}
              </div>
            ) : (
              <div className="flex items-center justify-center h-full text-gray-400 text-xs">
                No recent clips - Copy some text to see it here!
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Context Menu */}
      {menu && (
        <div
          ref={menuRef}
          className="absolute bg-white rounded-lg shadow-lg p-1.5 min-w-[120px] z-50"
          style={{ left: menu.x, top: menu.y }}
        >
          <button
            className="w-full py-1 text-left text-xs text-gray-800 hover:bg-gray-100 rounded-md"
            onClick={() => togglePin(menu.id)}
          >
            {items.find((x) => x.id === menu.id)?.pinned ? "Unpin" : "Pin"}
          </button>
          <button
            className="w-full py-1 text-left text-xs text-gray-800 hover:bg-gray-100 rounded-md"
            onClick={() => editItem(menu.id)}
          >
            {isTauri() ? "Edit" : "Edit Not"}
          </button>
          <button 
            className="w-full py-1 text-left text-xs text-gray-800 hover:bg-gray-100 rounded-md"
            onClick={(e) => {
              const rect = e.currentTarget.getBoundingClientRect();
              openTagDropdownForMenu(menu.id, rect);
            }}
          >
            Tags
          </button>
          <div className="h-px bg-gray-200 my-1"></div>
          <button
            className="w-full py-1 text-left text-xs text-red-500 hover:bg-gray-100 rounded-md"
            onClick={() => deleteItem(menu.id)}
          >
            Delete
          </button>
        </div>
      )}

      {/* Tag Assignment Dropdown */}
      {tagDropdown && (
        <div
          ref={tagDropdownRef}
          className="absolute bg-white rounded-lg shadow-lg p-2 w-44 z-50 border border-gray-200"
          style={{ 
            left: tagDropdown.x, 
            top: tagDropdown.y,
            transform: 'translate(-10%, -30%)'
          }}
        >
          <div className="flex justify-between items-center mb-1">
            <h3 className="text-xs font-semibold text-gray-800">Assign Tags</h3>
            <button
              className="text-gray-400 hover:text-gray-600 p-0.5"
              onClick={() => setTagDropdown(null)}
            >
              <X size={12} />
            </button>
          </div>

<div className="space-y-0.5 max-h-32 overflow-y-auto">
  {tags.map(tag => {
    const currentItem = items.find(item => item.id === tagDropdown.itemId);
    const hasTag = currentItem?.tags.includes(tag.name); // Check by name
    
    return (  
      <label key={tag.id} className="flex items-center gap-1.5 p-1 hover:bg-gray-50 rounded-md cursor-pointer">
        <div className="relative inline-flex items-center">
          <input
            type="checkbox"
            checked={hasTag}
            onChange={(e) => {
              console.log("🔄 Checkbox changed:", {
                checked: e.target.checked,
                tagName: tag.name,
                itemId: tagDropdown.itemId
              });
              if (e.target.checked) {
                assignTagToItem(tagDropdown.itemId, tag.id); 
              } else {
                removeTagFromItem(tagDropdown.itemId, tag.name, e); 
              }
            }}
            className="absolute opacity-0 w-4 h-4 cursor-pointer z-10"
          />
          <div className={`w-4 h-4 flex items-center justify-center ${hasTag ? 'text-blue-500' : 'text-gray-300'}`}>
            {hasTag ? (
              <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
              </svg>
            ) : (
              <div className="w-4 h-4 border border-gray-300 rounded-sm" />
            )}
          </div>
        </div>
      
        <span className="text-xs text-gray-700 flex-1">{tag.name}</span>
      </label>
    );
  })}
</div>
          
          <div className="mt-1 pt-1 border-t border-gray-200">
            <button
              className="w-full flex items-center justify-center gap-1 py-1 text-xs text-blue-500 hover:bg-blue-50 rounded-md"
              onClick={() => {
                setTagDropdown(null);
                setCreateTagModal(true);
              }}
            >
              <Plus size={10} />
              Create New Tag
            </button>
          </div>
        </div>
      )}

      {/* Create Tag Modal */}
      {createTagModal && (
        <div className="absolute inset-0 flex items-center justify-center z-50">
          <div
            ref={createTagModalRef}
            className="bg-white rounded-lg shadow-lg p-2 w-64 max-w-full mx-4 border border-gray-200"
          >
            <div className="flex justify-between items-center mb-2">
              <h3 className="text-sm font-semibold text-gray-800">Manage Tags</h3>
              <button
                className="text-gray-400 hover:text-gray-600 p-0.5"
                onClick={() => setCreateTagModal(false)}
              >
                <X size={16} />
              </button>
            </div>
            
            <div className="space-y-1 mb-2">
              <div>
                <label className="block text-xs font-medium text-gray-700 mb-0.5">
                  Create New Tag
                </label>
                <div className="flex gap-1">
                  <input
                    type="text"
                    value={newTagName}
                    onChange={(e) => setNewTagName(e.target.value)}
                    className="flex-1 px-2 py-1 border border-gray-300 rounded-md focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-xs"
                    placeholder="Enter tag name"
                    onKeyPress={(e) => e.key === 'Enter' && createNewTag()}
                  />
                  <button
                    className="py-1 px-2 text-xs text-white bg-blue-500 rounded-md hover:bg-blue-600 disabled:opacity-50"
                    onClick={createNewTag}
                    disabled={!newTagName.trim() || tagsLoading}
                  >
                    {tagsLoading ? "..." : "Add"}
                  </button>
                </div>
              </div>
            </div>

            {/* Existing Tags List */}
            {tags.length > 0 && (
              <div>
                <h4 className="text-xs font-semibold text-gray-700 mb-1">Existing Tags</h4>
                <div className="space-y-1 max-h-32 overflow-y-auto">
                  {tags.map(tag => (
                    <div key={tag.id} className="flex items-center justify-between p-1.5 bg-gray-50 rounded-md">
                      <div className="flex items-center gap-1.5">
                        <div 
                          className="w-2 h-2 rounded-full"
                          style={{ backgroundColor: tag.color }}
                        ></div>
                        <span className="text-xs text-gray-700">{tag.name}</span>
                      </div>
                      <button
                        onClick={() => handleDeleteTag(tag.id)}
                        className="text-gray-400 hover:text-red-500 p-0.5"
                        disabled={tagsLoading}
                      >
                        <X size={12} />
                      </button>
                    </div>
                  ))}
                </div>
              </div>
            )}
            
            <div className="flex gap-1 mt-3">
              <button
                className="flex-1 py-1 text-xs text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50"
                onClick={() => setCreateTagModal(false)}
                disabled={tagsLoading}
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Footer */}
      <div className="p-2 text-center text-xs text-gray-400 bg-white border-t border-gray-200 flex-shrink-0">
        Create with ❤️ by MakerStudio
      </div>
    </div>
  );
}

// ClipItem component remains the same
function ClipItem({ item, tags, onCopy, onMenuOpen, onTagClick, onRemoveTag, formatTime }) {
  
  return (
    <div 
      className="bg-white p-1.5 rounded-md shadow-sm cursor-pointer hover:shadow transition-all border border-gray-100"
      onClick={() => onCopy(item.content)}
    >
      <div className="text-xs text-gray-800 mb-0.5 line-clamp-2 leading-tight">
        {item.content}
      </div>
      
      {/* Tags display - with better debugging */}
      {item.tags && item.tags.length > 0 ? (
        <div className="flex flex-wrap gap-0.5 mb-0.5">
          {item.tags.map((tagName, index) => {


            // Find the tag to get its color
            const tag = tags.find(t => t.name === tagName);
            const tagColor = tag?.color || '#cccccc';
            
            return (
              <span
                key={`${tagName}-${index}`}
                className="inline-flex items-center gap-0.5 py-0.5 px-1 text-xs font-medium rounded-full border"
                style={{
                  backgroundColor: `${tagColor}20`,
                  color: tagColor,
                  borderColor: `${tagColor}40`
                }}
              >
                {tagName}
              </span>
            );
          })}
        </div>
      ) : (
        <div className="text-xs text-gray-400 mb-0.5">No tags</div> // Show when no tags
      )}
      
      <div className="flex justify-between items-center">
        <span className="text-xs text-gray-400">{formatTime(item.timestamp)}</span>
        <div className="flex gap-0.5">
          <button
            className="flex items-center gap-0.5 bg-gray-100 rounded-md py-0.5 px-1.5 text-xs font-medium text-gray-600 hover:bg-gray-200"
            onClick={(e) => {
              e.stopPropagation();
              onCopy(item.content);
            }}
          >
            Copy
          </button>
         
          <button
            className="bg-transparent rounded-full p-0.5 text-gray-600 hover:bg-gray-100"
            onClick={(e) => {
              e.stopPropagation();
              const rect = e.currentTarget.getBoundingClientRect();
              onMenuOpen({
                id: item.id,
                x: rect.right - 120,
                y: rect.bottom + 4,
              });
            }}
          >
            <MoreHorizontal size={12} />
          </button>
        </div>
      </div>
    </div>
  );
}