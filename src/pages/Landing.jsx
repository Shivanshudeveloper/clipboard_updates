import React, { useEffect, useMemo, useRef, useState } from "react";
import {
  Search, Copy, MoreHorizontal, X, Plus, LogOut, Settings
} from "lucide-react";
import { useClipboardDB } from "../hooks/useClipboardDB";
import { useTagsDB } from "../hooks/useTagsDB";
import { INITIAL_TAGS } from "../mock/data";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentUser, signOutUser } from "../libs/firebaseAuth";
import { useNavigate,Link } from "react-router-dom";
import { getAuth, onAuthStateChanged } from "firebase/auth";
import { SkeletonClipItem, SkeletonHeader, SkeletonTags } from "../components/home/SkeletonLoader";
import ClipItem from "../components/home/content";
import Tags from "../components/home/Tags";
import Header from "../components/common/header";
import ContextMenu from "../components/home/ContextMenu";
import TagDropdown from "../components/home/TagDropdown";
import CreateTagModal from "../components/home/CreateTagModal";
import { useUserPlan } from "../hooks/useUserPlan";



function isTauri() {
  return "__TAURI__" in window;
}

export default function ClipTray() {
  const {
    getClipboardEntries,
    updateEntryContent,
    deleteEntry,
    startPolling,
    initialLoad
  } = useClipboardDB();

  const {
    getTags,
    createTag,
    deleteTag: deleteTagBackend,
    loading: tagsLoading,
    error: tagsError,
    initialLoad: tagsInitialLoad
  } = useTagsDB();

  const [localItems, setLocalItems] = useState([]);
  const [isLoggingOut, setIsLoggingOut] = useState(false);
  const [sessionValid, setSessionValid] = useState(false);
  const [sessionChecking, setSessionChecking] = useState(true);
  const [notification, setNotification] = useState(null);

  const navigate = useNavigate();

   const { plan, isFree, isPro, loading: planLoading } = useUserPlan();
  const FREE_PIN_LIMIT = 3;

  const [showPinLimitBanner, setShowPinLimitBanner] = useState(false);

  // count pinned from current localItems
  const pinnedCount = useMemo(() => {
    return localItems.filter((i) => i.is_pinned).length;
  }, [localItems]);

  // auto-hide banner if user unpins or is pro
  useEffect(() => {
    if (isPro) setShowPinLimitBanner(false);
    if (isFree && pinnedCount < FREE_PIN_LIMIT) setShowPinLimitBanner(false);
  }, [isFree, isPro, pinnedCount]);

  const showNotification = (message, type = "error") => {
    setNotification({ message, type });
    setTimeout(() => {
      setNotification(null);
    }, 2500);
  };

  useEffect(() => {
    const checkAndRestoreSession = async () => {
      try {
        console.log("🔍 Checking session state...");
        
        const sessionState = await invoke('debug_session_state');
        
        if (sessionState.is_logged_in) {
          setSessionValid(true);
          setSessionChecking(false);
          return;
        }
                
        const auth = getAuth();
        const firebaseUser = auth.currentUser;
        
        if (firebaseUser) {
          const dbReady = await invoke('check_database_status');          
          if (!dbReady) {
            setTimeout(() => {
              checkAndRestoreSession();
            }, 1000);
            return;
          }          
          try {
            const idToken = await firebaseUser.getIdToken(true);
            const userResponse = await invoke('login_user', {
              firebaseToken: idToken,
              displayName: firebaseUser.displayName || "User",
            });
             setSessionValid(true);
            
          } catch (restoreError) {
            console.error("❌ Rust session restoration failed:", restoreError);
            
            try {
              const sessionValid = await invoke('validate_session', {
                firebaseToken: idToken,
              });
              
              if (sessionValid) {
                setSessionValid(true);
              } else {
                throw new Error("Alternative method failed");
              }
            } catch (altError) {
              console.error("❌ Alternative restoration failed:", altError);
              console.log("🟡 Continuing with Firebase user only");
              setSessionValid(true);
            }
          }
        } else {
          console.log("🔴 No Firebase user, redirecting to login");
          navigate("/login");
          return;
        }
        
      } catch (error) {
        console.error("❌ Session restoration failed:", error);
        if (error.toString().includes('state not managed') || error.toString().includes('pool')) {
          setTimeout(() => {
            checkAndRestoreSession();
          }, 1000);
        } else {
          navigate("/login");
        }
      } finally {
        setSessionChecking(false);
      }
    };

    setTimeout(() => {
      checkAndRestoreSession();
    }, 1000);
  }, [navigate]);

  useEffect(() => {
    if (!isTauri() || !sessionValid) return;
    
    const loadEntries = async () => {
      try {
        console.log("📥 Loading clipboard entries...");
        const data = await getClipboardEntries(2000);
        if (Array.isArray(data)) {
          console.log(`✅ Loaded ${data.length} entries`);
          setLocalItems(data);
        }
      } catch (err) {
        console.error("Error loading entries:", err);
        if (err.toString().includes('not logged in') || err.toString().includes('User not logged in')) {
          navigate("/login");
        }
      }
    };

    loadEntries();

    const cleanup = startPolling((newEntries) => {
      if (Array.isArray(newEntries)) {
        setLocalItems(newEntries);
      }
    }, 1000);

    return cleanup;
  }, [getClipboardEntries, startPolling, sessionValid, navigate]);

  const [q, setQ] = useState("");
  const [tags, setTags] = useState([]);
  const [menu, setMenu] = useState(null);
  const [activeTag, setActiveTag] = useState("all");
  const [tagDropdown, setTagDropdown] = useState(null);
  const [createTagModal, setCreateTagModal] = useState(false);
  const [newTagName, setNewTagName] = useState("");
  const [pinnedItems, setPinnedItems] = useState(new Set());
  const [itemTags, setItemTags] = useState({});


const openContextMenu = (itemId, rect) => {
  const menuWidth = 140;   // approximate
  const menuHeight = 130;  // approximate
  const padding = 8;

  const vw = window.innerWidth;
  const vh = window.innerHeight;

  let x = rect.right - menuWidth;
  let y = rect.bottom + 4;

  // Clamp horizontally
  if (x < padding) x = padding;
  if (x + menuWidth > vw - padding) x = vw - menuWidth - padding;

  // If bottom overflows, open above
  if (y + menuHeight > vh - padding) {
    y = rect.top - menuHeight - 4;
  }

  if (y < padding) y = padding;

  setMenu({ id: itemId, x, y });
};


  useEffect(() => {
    if (!sessionValid) return;

    const loadTags = async () => {
      try {
        console.log("🏷️ Loading tags...");
        const backendTags = await getTags(localItems);
        
        if (Array.isArray(backendTags)) {
          console.log(`✅ Loaded ${backendTags.length} pre-sorted tags`);
          setTags(backendTags);
        }
      } catch (err) {
        console.error("Error loading tags:", err);
        if (tags.length === 0) {
          setTags(INITIAL_TAGS);
        }
      }
    };

    if (localItems.length > 0 || tags.length === 0) {
      loadTags();
    }
  }, [sessionValid, localItems.length]);

  useEffect(() => {
    const pinnedIds = new Set();
    localItems.forEach(item => {
      if (item.is_pinned) {
        pinnedIds.add(item.id);
      }
    });
    setPinnedItems(pinnedIds);
  }, [localItems]);

  const items = useMemo(() => {
    return localItems.map((item, index) => {
      let tagsArray = [];
      
      if (item.tags) {
        if (typeof item.tags === 'string') {
          try {
            let cleanTags = item.tags.trim().replace(/\\/g, '');
            tagsArray = JSON.parse(cleanTags);
          } catch (e) {
            console.error("Error parsing tags JSON:", e, "Raw tags:", item.tags);
            tagsArray = [];
          }
        } else if (Array.isArray(item.tags)) {
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

  const filtered = useMemo(() => {
    const s = q.trim().toLowerCase();
    let filteredItems = items;
    
    if (s) {
      filteredItems = filteredItems.filter(x => x.content.toLowerCase().includes(s));
    }
    
    if (activeTag !== "all") {
      filteredItems = filteredItems.filter(item => 
        item.tags && item.tags.includes(activeTag)
      );
    }
    
    return filteredItems;
  }, [q, items, activeTag]);

  const pinned = filtered
    .filter(x => x.pinned)
    .sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp));

  const recent = filtered
    .filter(x => !x.pinned)
    .sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp));

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

  const handleLogout = async () => {
    try {
      setIsLoggingOut(true);
      await signOutUser();
      const result = await invoke('logout_user');
      console.log(result);
      localStorage.removeItem('cliptray_user');
      sessionStorage.clear();
      
      navigate("/login");
      window.location.reload();
    } catch (error) {
      console.error("Logout failed:", error);
      showNotification("Logout failed. Please try again.", "error");
    } finally {
      setIsLoggingOut(false);
    }
  };

  const togglePin = async (id) => {
  const currentItem = items.find((x) => x.id === id);
  const newPinnedState = !currentItem?.pinned;

  // ✅ enforce limit only when trying to PIN (not unpin)
  if (!planLoading && isFree && newPinnedState && pinnedCount >= FREE_PIN_LIMIT) {
    setShowPinLimitBanner(true);
    showNotification("Limit reached. Upgrade to Pro to pin more than 3 items.", "error");
    setMenu(null);
    return;
  }

  try {
    await invoke("update_entry", {
      id,
      updates: { is_pinned: newPinnedState },
    });

    setPinnedItems((prev) => {
      const next = new Set(prev);
      if (newPinnedState) next.add(id);
      else next.delete(id);
      return next;
    });
  } catch (err) {
    console.error("Failed to update pin state:", err);
    showNotification("Failed to update pin state.", "error");
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
        showNotification("Edited content updated!", "success");
      }
    } catch (err) {
      console.error("Error editing content:", err);
      showNotification("Failed to edit content.", "error");
    }
    setMenu(null);
  };

  // 🔔 updated deleteItem with in-app notification
  const deleteItem = async (id) => {
    if (!navigator.onLine) {
      showNotification("You are offline. Connect to the internet to delete items.", "error");
      setMenu(null);
      return;
    }

    try {
      await deleteEntry(id);
      setLocalItems((prev) => prev.filter((x) => x.id !== id));
      showNotification("Item deleted.", "success");
    } catch (err) {
      console.error("Failed to delete entry:", err);
      showNotification("Failed to delete item.", "error");
    }

    setMenu(null);
  };

  const copyToClipboard = (text) => navigator.clipboard.writeText(text);

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
        showNotification("Tag created.", "success");
      }
    } catch (err) {
      console.error("Failed to create tag:", err);
      showNotification("Failed to create tag.", "error");
    }
  };

  const refreshClipboardData = async () => {
    try {
      const data = await getClipboardEntries(200);
      if (Array.isArray(data)) {
        setLocalItems(data);
      }
    } catch (err) {
      console.error("Failed to refresh clipboard data:", err);
    }
  };

  const removeTagFromItem = async (itemId, tagName, e) => {
    if (e) e.stopPropagation();
    
    try {
      console.log("🔴 REMOVING tag:", tagName, "from item:", itemId);
      
      setLocalItems(prev => prev.map(item => {
        if (item.id === parseInt(itemId)) {
          const currentTags = Array.isArray(item.tags) ? item.tags : [];
          const newTags = currentTags.filter(t => t !== tagName);
          return { ...item, tags: newTags };
        }
        return item;
      }));

      setTagDropdown(null);
      setMenu(null);
      
      const updatedEntry = await invoke("remove_tag_from_entry", {
        clipboardEntryId: parseInt(itemId),
        tagName: tagName
      });
      
      let parsedTags = [];
      if (updatedEntry.tags) {
        if (typeof updatedEntry.tags === 'string') {
          try {
            let cleanTags = updatedEntry.tags.trim().replace(/\\"/g, '"').replace(/\\\\/g, '\\');
            if (cleanTags.startsWith('[') && cleanTags.endsWith(']')) {
              parsedTags = JSON.parse(cleanTags);
            } else {
              parsedTags = [cleanTags];
            }
          } catch (e) {
            console.error("Error parsing tags:", e);
            parsedTags = [];
          }
        } else if (Array.isArray(updatedEntry.tags)) {
          parsedTags = updatedEntry.tags;
        }
      }
      
      setLocalItems(prev => prev.map(item => {
        if (item.id === parseInt(itemId)) {
          return { ...item, tags: parsedTags };
        }
        return item;
      }));

      showNotification("Tag removed.", "success");
      
    } catch (err) {
      console.error("❌ Failed to remove tag:", err);
      showNotification("Failed to remove tag.", "error");
      refreshClipboardData();
    }
  };

  const assignTagToItem = async (itemId, tagId) => {
    try {
      const tag = tags.find(t => t.id === tagId);
      if (!tag) {
        console.error("Tag not found with ID:", tagId);
        return;
      }

      console.log("🟢 ASSIGNING tag:", tag.name, "to item:", itemId);

      setLocalItems(prev => prev.map(item => {
        if (item.id === parseInt(itemId)) {
          const currentTags = Array.isArray(item.tags) ? item.tags : [];
          const newTags = [...currentTags, tag.name];
          return { ...item, tags: newTags };
        }
        return item;
      }));

      setTagDropdown(null);

      const updatedEntry = await invoke("assign_tag_to_entry", {
        clipboardEntryId: parseInt(itemId),
        tagName: tag.name
      });
      
      let parsedTags = [];
      if (updatedEntry.tags) {
        if (typeof updatedEntry.tags === 'string') {
          try {
            let cleanTags = updatedEntry.tags.trim().replace(/\\"/g, '"').replace(/\\\\/g, '\\');
            if (cleanTags.startsWith('[') && cleanTags.endsWith(']')) {
              parsedTags = JSON.parse(cleanTags);
            } else {
              parsedTags = [cleanTags];
            }
          } catch (e) {
            console.error("Error parsing tags in assign:", e);
            parsedTags = [];
          }
        } else if (Array.isArray(updatedEntry.tags)) {
          parsedTags = updatedEntry.tags;
        }
      }
      
      setLocalItems(prev => prev.map(item => 
        item.id === parseInt(itemId) ? { 
          ...item, 
          tags: parsedTags 
        } : item
      ));

      showNotification("Tag assigned.", "success");
      
    } catch (err) {
      console.error("❌ Failed to assign tag:", err);
      showNotification("Failed to assign tag.", "error");
      refreshClipboardData();
    }
  };

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
        showNotification("Tag deleted.", "success");
      }
    } catch (err) {
      console.error("Failed to delete tag:", err);
      showNotification("Failed to delete tag.", "error");
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

  if (sessionChecking) {
    return (
      <div className="flex flex-col bg-white relative" style={{ height: '565px' }}>
        <div className="flex items-center justify-center h-full">
          <div className="text-center">
            <div className="w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
            <p className="text-sm text-gray-600">Loading Page.....</p>
          </div>
        </div>
      </div>
    );
  }

  if (initialLoad || tagsInitialLoad) {
    return (
      <div className="flex flex-col bg-white relative" style={{ height: '565px' }}>
        <SkeletonHeader />
        <SkeletonTags />
        <div className="flex flex-col flex-1 mb-1">
          <div className="flex justify-between items-center p-2 text-xs font-semibold text-gray-500 uppercase tracking-wider flex-shrink-0">
            <span>Pinned</span>
            <div className="w-6 h-4 bg-gray-200 rounded-full"></div>
          </div>
          <div className="flex-1 overflow-y-auto min-h-0 p-2 space-y-1.5">
            {[...Array(1)].map((_, i) => (
              <SkeletonClipItem key={i} />
            ))}
          </div>
        </div>
        <div className="flex flex-col flex-1">
          <div className="flex justify-between items-center p-2 text-xs font-semibold text-gray-500 uppercase tracking-wider flex-shrink-0">
            <span>Recent</span>
            <div className="w-6 h-4 bg-gray-200 rounded-full"></div>
          </div>
          <div className="flex-1 overflow-y-auto min-h-0 p-2 space-y-1.5">
            {[...Array(2)].map((_, i) => (
              <SkeletonClipItem key={i} />
            ))}
          </div>
        </div>
        <div className="p-2 text-center text-xs text-gray-400 bg-white border-t border-gray-200 flex-shrink-0">
          Create with ❤️ by MakerStudio
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col bg-white relative" style={{ height: '565px'}}>
      {/* Header */}
      <Header
      q={q}
      setQ={setQ}
      onLogout={handleLogout}
      isLoggingOut={isLoggingOut}
      showUpgradeBanner={showPinLimitBanner}
      onUpgradeClick={() => navigate("/settings")}   // or your upgrade page
      onDismissUpgrade={() => setShowPinLimitBanner(false)}
      />



      {/* Tags */}
        <Tags
        tags={tags}
        activeTag={activeTag}
        setActiveTag={setActiveTag}
        tagsLoading={tagsLoading}
        tagsError={tagsError}
        />


      {/* Content */}
      <div className="flex-1 overflow-hidden flex flex-col p-2">
        {/* Pinned */}
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
                    onCopy={copyToClipboard}
                    onMenuOpen={openContextMenu}
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

        {/* Recent */}
        <div className="flex flex-col" style={{ height: '40%', minHeight: '50%' }}>
          <div className="flex justify-between items-center p-2 text-xs font-semibold text-gray-500 uppercase tracking-wider flex-shrink-0">
            <div className="flex items-center gap-2">
            <span>Recent</span>

            {isFree && (
              <span className="normal-case text-[10px] font-medium text-gray-400">
                Deletes every 24 hrs
              </span>
            )}
            </div>
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
                    onCopy={copyToClipboard}
                    onMenuOpen={openContextMenu}
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
      <ContextMenu
      ref={menuRef}
      menu={menu}
      items={items}
      isTauri={isTauri()}
      onTogglePin={togglePin}
      onEdit={editItem}
      onOpenTags={openTagDropdownForMenu}
      onDelete={deleteItem}
      />


      {/* Tag Dropdown */}
      <TagDropdown
      ref={tagDropdownRef}
      dropdown={tagDropdown}
      tags={tags}
      items={items}
      onClose={() => setTagDropdown(null)}
      onAssignTag={assignTagToItem}
      onRemoveTag={removeTagFromItem}
      onOpenCreateTag={() => {
      setTagDropdown(null);
      setCreateTagModal(true);
      }}
      />


      {/* Create Tag Modal */}
      <CreateTagModal
      ref={createTagModalRef}
      isOpen={createTagModal}
      tags={tags}
      newTagName={newTagName}
      setNewTagName={setNewTagName}
      tagsLoading={tagsLoading}
      onAddTag={createNewTag}
      onDeleteTag={handleDeleteTag}
      onClose={() => setCreateTagModal(false)}
      />


      {/* 🔔 Notification Toast */}
      {notification && (
        <div className="absolute bottom-10 left-1/2 transform -translate-x-1/2 z-50">
          <div
            className={`px-3 py-2 rounded-md shadow-md text-xs flex items-center gap-2 ${
              notification.type === "success"
                ? "bg-green-50 text-green-700 border border-green-200"
                : "bg-red-50 text-red-700 border border-red-200"
            }`}
          >
            <span>{notification.message}</span>
            <button
              className="ml-1 text-[10px] text-gray-400 hover:text-gray-600"
              onClick={() => setNotification(null)}
            >
              <X size={10} />
            </button>
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
