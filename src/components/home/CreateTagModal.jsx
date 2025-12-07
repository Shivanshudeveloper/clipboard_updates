// src/components/home/CreateTagModal.jsx
import React, { forwardRef } from "react";
import { X } from "lucide-react";

const CreateTagModal = forwardRef(
  (
    {
      isOpen,
      tags,
      newTagName,
      setNewTagName,
      tagsLoading,
      onAddTag,      // () => void
      onDeleteTag,   // (tagId) => void
      onClose,       // () => void
    },
    ref
  ) => {
    if (!isOpen) return null;

    return (
      <div className="absolute inset-0 flex items-center justify-center z-50">
        <div
          ref={ref}
          className="bg-white rounded-lg shadow-lg p-2 w-64 max-w-full mx-4 border border-gray-200"
        >
          {/* Header */}
          <div className="flex justify-between items-center mb-2">
            <h3 className="text-sm font-semibold text-gray-800">Manage Tags</h3>
            <button
              className="text-gray-400 hover:text-gray-600 p-0.5"
              onClick={onClose}
            >
              <X size={16} />
            </button>
          </div>

          {/* Create new tag */}
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
                  onKeyPress={(e) => e.key === "Enter" && onAddTag()}
                />
                <button
                  className="py-1 px-2 text-xs text-white bg-blue-500 rounded-md hover:bg-blue-600 disabled:opacity-50"
                  onClick={onAddTag}
                  disabled={!newTagName.trim() || tagsLoading}
                >
                  {tagsLoading ? "..." : "Add"}
                </button>
              </div>
            </div>
          </div>

          {/* Existing tags list */}
          {tags.length > 0 && (
            <div>
              <h4 className="text-xs font-semibold text-gray-700 mb-1">
                Existing Tags
              </h4>
              <div className="space-y-1 max-h-32 overflow-y-auto">
                {tags.map((tag) => (
                  <div
                    key={tag.id}
                    className="flex items-center justify-between p-1.5 bg-gray-50 rounded-md"
                  >
                    <div className="flex items-center gap-1.5">
                      <div
                        className="w-2 h-2 rounded-full"
                        style={{ backgroundColor: tag.color }}
                      ></div>
                      <span className="text-xs text-gray-700">{tag.name}</span>
                    </div>
                    <button
                      onClick={() => onDeleteTag(tag.id)}
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

          {/* Footer */}
          <div className="flex gap-1 mt-3">
            <button
              className="flex-1 py-1 text-xs text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50"
              onClick={onClose}
              disabled={tagsLoading}
            >
              Close
            </button>
          </div>
        </div>
      </div>
    );
  }
);

export default CreateTagModal;
