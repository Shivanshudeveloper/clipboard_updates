// src/components/home/ContextMenu.jsx
import React, { forwardRef } from "react";

const ContextMenu = forwardRef(
  (
    {
      menu,       // { id, x, y } or null
      items,      // full items array, used to check pinned state
      isTauri,    // boolean
      onTogglePin,
      onEdit,
      onOpenTags,
      onDelete,
    },
    ref
  ) => {
    if (!menu) return null;

    const item = items.find((x) => x.id === menu.id);
    const isPinned = item?.pinned;

    return (
      <div
        ref={ref}
        className="absolute bg-white rounded-lg shadow-lg p-1.5 min-w-[120px] z-50"
        style={{ left: menu.x, top: menu.y }}
      >
        <button
          className="w-full py-1 text-left text-xs text-gray-800 hover:bg-gray-100 rounded-md"
          onClick={() => onTogglePin(menu.id)}
        >
          {isPinned ? "Unpin" : "Pin"}
        </button>

        <button
          className="w-full py-1 text-left text-xs text-gray-800 hover:bg-gray-100 rounded-md"
          onClick={() => onEdit(menu.id)}
        >
          {isTauri ? "Edit" : "Edit Not"}
        </button>

        <button
          className="w-full py-1 text-left text-xs text-gray-800 hover:bg-gray-100 rounded-md"
          onClick={(e) => {
            const rect = e.currentTarget.getBoundingClientRect();
            onOpenTags(menu.id, rect); // extra arg is fine, your function can ignore it
          }}
        >
          Tags
        </button>

        <div className="h-px bg-gray-200 my-1"></div>

        <button
          className="w-full py-1 text-left text-xs text-red-500 hover:bg-gray-100 rounded-md"
          onClick={() => onDelete(menu.id)}
        >
          Delete
        </button>
      </div>
    );
  }
);

export default ContextMenu;
