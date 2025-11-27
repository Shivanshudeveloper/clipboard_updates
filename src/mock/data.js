export const INITIAL = [
  { id: "1", content: "CliptrayNotary app specific password is quae-fxze-...", timestamp: new Date(Date.now() - 1000 * 60).toISOString(), pinned: true, tags: ["1"] },
  { id: "2", content: "Package a fresh release build using Developer ID A...", timestamp: new Date(Date.now() - 1000 * 60 * 8).toISOString(), pinned: false, tags: ["2"] },
  { id: "3", content: "give me the product-implementation-bridge doc fo..", timestamp: new Date(Date.now() - 1000 * 60 * 60 * 24 * 6).toISOString(), pinned: false, tags: ["1"] },
  
  // Recent entries (last few minutes)
  { id: "4", content: "Meeting notes: Discuss Q3 roadmap and resource allocation for new features", timestamp: new Date(Date.now() - 1000 * 60 * 2).toISOString(), pinned: false, tags: ["2"] },
  { id: "5", content: "API endpoint: https://api.example.com/v1/users/current", timestamp: new Date(Date.now() - 1000 * 60 * 5).toISOString(), pinned: true, tags: ["3"] },
  { id: "6", content: "Error fix: Resolved null pointer exception in user authentication module", timestamp: new Date(Date.now() - 1000 * 60 * 12).toISOString(), pinned: true, tags: ["2"] },
  
  { id: "7", content: "Shopping list: milk, eggs, bread, coffee, fruits", timestamp: new Date(Date.now() - 1000 * 60 * 60 * 3).toISOString(), pinned: true, tags: ["3"] },
  { id: "8", content: "Code snippet: const debounce = (fn, delay) => { let timeoutId; return (...args) => { clearTimeout(timeoutId); timeoutId = setTimeout(() => fn(...args), delay); }; }", timestamp: new Date(Date.now() - 1000 * 60 * 60 * 5).toISOString(), pinned: true, tags: ["2"] },
  
];


export const INITIAL_TAGS = [
  { id: "1", name: "keys", color: "#3B82F6" },
  { id: "2", name: "mcp", color: "#3B82F6" },
  { id: "3", name: "prompts", color: "#3B82F6" },
];

