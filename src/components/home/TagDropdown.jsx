// src/components/home/TagDropdown.jsx
import React, { forwardRef } from "react";
import { X, Plus } from "lucide-react";

const TagDropdown = forwardRef(
  (
    {
      dropdown,          // { itemId, x, y } or null
      tags,
      items,
      onClose,
      onAssignTag,       // (itemId, tagId)
      onRemoveTag,       // (itemId, tagName)
      onOpenCreateTag,   // () => void
    },
    ref
  ) => {
    if (!dropdown) return null;

    const currentItem = items.find(item => item.id === dropdown.itemId);

    return (
      <div
        ref={ref}
        className="absolute bg-white rounded-lg shadow-lg p-2 w-44 z-50 border border-gray-200"
        style={{
          left: dropdown.x,
          top: dropdown.y,
          transform: "translate(-10%, -30%)",
        }}
      >
        <div className="flex justify-between items-center mb-1">
          <h3 className="text-xs font-semibold text-gray-800">Assign Tags</h3>
          <button
            className="text-gray-400 hover:text-gray-600 p-0.5"
            onClick={onClose}
          >
            <X size={12} />
          </button>
        </div>

        {/* Tag list */}
        <div className="space-y-0.5 max-h-32 overflow-y-auto">
          {tags.map((tag) => {
            const hasTag = currentItem?.tags?.includes(tag.name);

            return (
              <label
                key={tag.id}
                className="flex items-center gap-1.5 p-1 hover:bg-gray-50 rounded-md cursor-pointer"
              >
                <div className="relative inline-flex items-center">
                  <input
                    type="checkbox"
                    checked={!!hasTag}
                    onChange={(e) => {
                      const shouldAssign = e.target.checked;

                      if (shouldAssign && !hasTag) {
                        onAssignTag(dropdown.itemId, tag.id);
                      } else if (!shouldAssign && hasTag) {
                        onRemoveTag(dropdown.itemId, tag.name);
                      }
                    }}
                    className="absolute opacity-0 w-4 h-4 cursor-pointer z-10"
                  />
                  <div
                    className={`w-4 h-4 flex items-center justify-center ${
                      hasTag ? "text-blue-500" : "text-gray-300"
                    }`}
                  >
                    {hasTag ? (
                      <svg
                        className="w-4 h-4"
                        fill="currentColor"
                        viewBox="0 0 20 20"
                      >
                        <path
                          fillRule="evenodd"
                          d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                          clipRule="evenodd"
                        />
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

        {/* Footer: create new tag */}
        <div className="mt-1 pt-1 border-t border-gray-200">
          <button
            className="w-full flex items-center justify-center gap-1 py-1 text-xs text-blue-500 hover:bg-blue-50 rounded-md"
            onClick={onOpenCreateTag}
          >
            <Plus size={10} />
            Create New Tag
          </button>
        </div>
      </div>
    );
  }
);

export default TagDropdown;
