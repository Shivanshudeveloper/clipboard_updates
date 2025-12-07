import React, { useState, useEffect } from "react";
import { X, Plus, RefreshCw } from "lucide-react";
import { useTagsDB } from "../../hooks/useTagsDB";

const TagsSettings = ({ organizationId, onClose }) => {
  const { getTags, createTag, deleteTag, loading, error } = useTagsDB();
  const [tags, setTags] = useState([]);
  const [newTagName, setNewTagName] = useState("");
  const [newTagColor, setNewTagColor] = useState("#3b82f6"); // Default blue
  const [alert, setAlert] = useState({ show: false, message: "", type: "" });

  const showAlert = (message, type) => {
    setAlert({ show: true, message, type });
    setTimeout(() => {
      setAlert({ show: false, message: "", type: "" });
    }, 2000);
  };

  useEffect(() => {
    loadTags();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [organizationId]);

  const loadTags = async () => {
    try {
      const tagsData = await getTags();
      console.log(tagsData);
      if (Array.isArray(tagsData)) {
        setTags(tagsData);
      }
    } catch (err) {
      console.error("Failed to load tags:", err);
    }
  };

  const handleAddTag = async () => {
    if (!newTagName.trim()) return;

    try {
      const newTag = await createTag({
        name: newTagName.trim(),
        color: newTagColor,
      });

      if (newTag) {
        setTags((prev) => [...prev, newTag]);
        setNewTagName("");
        setNewTagColor("#3b82f6");
        showAlert("Tag created successfully!", "success");
      }
    } catch (err) {
      console.error("Failed to create tag:", err);
      showAlert("Failed to create tag", "error");
    }
  };

  const handleDeleteTag = async (tag) => {
    // 🔴 If offline, do NOT call backend – just show notification
    if (!navigator.onLine) {
      showAlert(
        "You are offline. Please connect to the internet to delete tags.",
        "error"
      );
      return;
    }

    console.log(tag);
    const dbId = tag.server_id??tag.id ;
    console.log("Deleting tag with dbId:", dbId);

    try {
      const success = await deleteTag(dbId);
      if (success) {
        setTags((prev) => prev.filter((t) => t.id !== tag.id));
        showAlert("Tag deleted successfully!", "success");
      } else {
        showAlert("Tag could not be deleted.", "error");
      }
    } catch (err) {
      console.error("Failed to delete tag:", err);
      showAlert("Failed to delete tag", "error");
    }
  };

  const generateRandomColor = () => {
    const colors = [
      "#3b82f6", "#ef4444", "#10b981", "#f59e0b", "#8b5cf6",
      "#ec4899", "#06b6d4", "#84cc16", "#f97316", "#6366f1",
      "#14b8a6", "#f43f5e", "#84cc16", "#eab308", "#a855f7"
    ];
    return colors[Math.floor(Math.random() * colors.length)];
  };

  const handleRefresh = () => {
    loadTags();
  };

  return (
    <div className="flex flex-col relative" style={{ height: "390px" }}>
      {/* Centered Alert Box */}
      {alert.show && (
        <div
          className={`absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 z-50 px-6 py-4 rounded-lg shadow-lg border ${
            alert.type === "success"
              ? "bg-green-50 text-green-800 border-green-200"
              : "bg-red-50 text-red-800 border-red-200"
          }`}
        >
          <div className="flex items-center gap-2">
            {alert.type === "success" ? (
              <div className="w-5 h-5 bg-green-500 rounded-full flex items-center justify-center">
                <span className="text-white text-xs">✓</span>
              </div>
            ) : (
              <div className="w-5 h-5 bg-red-500 rounded-full flex items-center justify-center">
                <span className="text-white text-xs">!</span>
              </div>
            )}
            <span className="font-medium">{alert.message}</span>
          </div>
        </div>
      )}

      {/* Header */}
      <div className="flex-shrink-0 pb-4 border-b border-gray-200">
        <div className="flex justify-between items-center mb-2">
          <h2 className="text-lg font-semibold text-gray-800">Manage Tags</h2>
          <div className="flex gap-2">
            <button
              onClick={handleRefresh}
              disabled={loading}
              className="p-1.5 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-md transition-colors"
              title="Refresh tags"
            >
              <RefreshCw size={16} className={loading ? "animate-spin" : ""} />
            </button>
            {onClose && (
              <button
                onClick={onClose}
                className="p-1.5 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-md transition-colors"
              >
                <X size={16} />
              </button>
            )}
          </div>
        </div>

        {error && (
          <div className="p-3 text-sm text-red-600 bg-red-50 rounded-md border border-red-200">
            <strong>Error:</strong> {error}
          </div>
        )}
      </div>

      {/* Create Tag */}
      <div className="flex-shrink-0 py-4 border-b border-gray-200">
        <h3 className="text-sm font-medium text-gray-700 mb-2">
          Create New Tag
        </h3>
        <div className="space-y-3">
          <div className="flex gap-2">
            <input
              type="text"
              placeholder="Enter tag name"
              value={newTagName}
              onChange={(e) => setNewTagName(e.target.value)}
              onKeyPress={(e) => e.key === "Enter" && handleAddTag()}
              className="flex-1 px-2 py-2 text-sm border border-gray-300 rounded-lg outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
              disabled={loading}
              maxLength={50}
            />

            <button
              onClick={handleAddTag}
              disabled={!newTagName.trim() || loading}
              className="flex items-center gap-2 px-3 py-1 bg-blue-500 text-white text-xs font-medium rounded-lg hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {loading ? (
                <div className="w-2 h-1 border-2 border-white border-t-transparent rounded-full animate-spin" />
              ) : (
                <Plus size={12} />
              )}
              Add Tag
            </button>
          </div>
        </div>
      </div>

      {/* Existing Tags */}
      <div
        className={
          "flex-1 min-h-0 py-2 " +
          (tags.length > 3 ? "overflow-y-auto" : "")
        }
      >
        <div className="flex justify-between items-center mb-10">
          <h3 className="text-sm font-medium text-gray-700">
            Existing Tags {tags.length > 0 && `(${tags.length})`}
          </h3>
          {tags.length > 0 && (
            <span className="text-xs text-gray-500">Click × to delete</span>
          )}
        </div>

        {loading && tags.length === 0 ? (
          <div className="h-full flex items-center justify-center text-gray-500 text-sm">
            <div className="text-center">
              <RefreshCw size={20} className="animate-spin mx-auto mb-2" />
              Loading tags...
            </div>
          </div>
        ) : tags.length === 0 ? (
          <div className="h-full flex items-center justify-center">
            <div className="text-center py-8 text-gray-500 text-sm border-2 border-dashed border-gray-200 rounded-lg w-full">
              <div className="mb-2"></div>
              No tags created yet
              <div className="text-xs mt-1">
                Create your first tag above to get started
              </div>
            </div>
          </div>
        ) : (
          <div className="h-full flex flex-col">
            <div className="flex-1 pr-1 space-y-2">
              {tags.map((tag) => (
                <div
                  key={tag.id}
                  className="flex items-center gap-3 p-3 bg-white rounded-lg border border-gray-200 hover:border-gray-300 transition-colors group"
                >
                  <div className="flex items-center gap-3 flex-1 min-w-0">
                    <div className="min-w-0 flex-1">
                      <span className="text-sm font-medium text-gray-800 block truncate">
                        {tag.name}
                      </span>
                      {/* Hide color if you don't want to show it:
                      <div className="text-xs text-gray-500 font-mono">
                        {tag.color}
                      </div> */}
                    </div>
                  </div>

                  <button
                    onClick={() => handleDeleteTag(tag)}
                    disabled={loading}
                    className="p-1.5 text-gray-400 hover:text-red-500 hover:bg-red-50 rounded-md transition-colors disabled:opacity-50 opacity-0 group-hover:opacity-100"
                    title="Delete tag"
                  >
                    <X size={16} />
                  </button>
                </div>
              ))}
            </div>

            <div className="flex-shrink-0 pt-3 mt-3 border-t border-gray-200">
              <div className="text-xs text-gray-500 text-center">
                Showing {tags.length} tag{tags.length !== 1 ? "s" : ""}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default TagsSettings;
