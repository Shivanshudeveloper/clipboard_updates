import React, { useRef, useState } from "react";
import { MoreHorizontal } from "lucide-react";

export default function ClipItem({ item, tags, onCopy, onMenuOpen, onTagClick, onRemoveTag, formatTime }) {
  const contentRef = useRef(null);
  const [isContainerHovered, setIsContainerHovered] = useState(false);

  const handleContainerMouseEnter = () => {
    setIsContainerHovered(true);
    if (contentRef.current) {
      contentRef.current.style.webkitLineClamp = '2';
      contentRef.current.style.maxHeight = '2.8em';
    }
  };

  const handleContainerMouseLeave = () => {
    setIsContainerHovered(false);
    if (contentRef.current) {
      contentRef.current.style.webkitLineClamp = '1';
      contentRef.current.style.maxHeight = '1.4em';
    }
  };

  return (
    <div 
      className="bg-white p-1.5 rounded-md shadow-sm cursor-pointer hover:shadow transition-all border border-gray-100"
      onClick={() => onCopy(item.content)}
      onMouseEnter={handleContainerMouseEnter}
      onMouseLeave={handleContainerMouseLeave}
    >
      <div 
        ref={contentRef}
        className="text-xs text-gray-800 mb-0.5 leading-tight overflow-hidden transition-all duration-300 ease-in-out"
        style={{
          display: '-webkit-box',
          WebkitBoxOrient: 'vertical',
          WebkitLineClamp: 1,
          maxHeight: '1.4em',
        }}
      >
        {item.content}
      </div>
      
      {item.tags && item.tags.length > 0 ? (
        <div className="flex flex-wrap gap-0.5 mb-0.5">
          {item.tags.map((tagName, index) => {
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
        <div className="text-xs text-gray-400 mb-0.5">No tags</div>
      )}
      
      <div className="flex justify-between items-center">
        <span className="text-xs text-gray-400">{formatTime(item.timestamp)}</span>
        
        {isContainerHovered && (
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
    onMenuOpen(item.id, rect);      // ✅ pass (id, rect)
  }}
>
  <MoreHorizontal size={12} />
</button>

          </div>
        )}
      </div>
    </div>
  );
}