import React from "react";

export default function Tags({ tags, activeTag, setActiveTag, tagsLoading, tagsError }) {
  return (
    <div className="bg-white px-2 pt-0.5 flex-shrink-0">
      <div className="flex items-center gap-1 overflow-x-auto pb-0.5">
        {/* All button */}
        <button
          className={`flex items-center gap-0.5 py-0.5 px-1.5 text-xs font-medium rounded-full border transition-all whitespace-nowrap shadow-sm ${
            activeTag === "all"
              ? "bg-blue-500 text-white border-transparent shadow-md"
              : "bg-white text-gray-700 border-gray-300 hover:border-gray-400 hover:shadow"
          }`}
          onClick={() => setActiveTag("all")}
        >
          All
        </button>

        {/* Tags */}
        {tagsLoading ? (
          <div className="text-xs text-gray-500">Loading tags...</div>
        ) : (
          tags.map((tag) => (
            <button
              key={tag.id}
              className={`flex items-center gap-0.5 py-0.5 px-1.5 text-xs font-medium rounded-full border transition-all whitespace-nowrap shadow-sm ${
                activeTag === tag.name
                  ? "text-white border-transparent shadow-md"
                  : "bg-white text-gray-700 border-gray-300 hover:border-gray-400 hover:shadow"
              }`}
              style={{
                backgroundColor:
                  activeTag === tag.name ? tag.color : "transparent",
                borderColor: activeTag === tag.name ? tag.color : "",
              }}
              onClick={() => setActiveTag(tag.name)}
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
  );
}


