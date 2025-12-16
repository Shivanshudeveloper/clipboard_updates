import React, { useState } from "react";
import GeneralSettings from "../components/settings/General";
import TagsSettings from "../components/settings/Tags";
import { ArrowLeft, Info, Crown } from "lucide-react";
import { Link } from 'react-router-dom';


const ClipTraySettings = () => {
  const [activeTab, setActiveTab] = useState("General");

  const [startAtLogin, setStartAtLogin] = useState(true);
  const [autoPurgeUnpinned, setAutoPurgeUnpinned] = useState(true);
  const [purgeCadence, setPurgeCadence] = useState("Every 24 hours");

  const purgeAllUnpinned = () => {
    console.log("Purging all unpinned items");
    alert("All unpinned items have been purged.");
  };

  return (
    <div className="w-auto h-[565px] bg-gray-50 border border-gray-300 rounded-xl shadow-sm p-5 font-sans flex flex-col overflow-hidden">

      {/* Back Button */}
      <Link to="/home">
        <button className="flex items-center gap-2 text-gray-700 hover:text-black transition mt-3">
          <ArrowLeft size={20} />
          <span>Back</span>
        </button>
      </Link>

      {/* Header */}
      <div className="flex justify-between items-center mb-3">
        <h2 className="text-base font-semibold text-gray-800">
          ClipTray Settings
        </h2>
      </div>

      {/* Tabs */}
      <div className="flex gap-2 mb-1 flex-shrink-0">
        {["General", "Tags"].map((tab) => (
          <button
            key={tab}
            className={`px-3.5 py-1.5 text-sm border border-gray-300 rounded-lg cursor-pointer transition-all duration-200 ${
              activeTab === tab
                ? "bg-blue-500 text-white border-blue-500 shadow-sm shadow-blue-500/30"
                : "bg-gray-200 hover:bg-gray-300"
            }`}
            onClick={() => setActiveTab(tab)}
          >
            {tab}
          </button>
        ))}
      </div>

      {/* ✅ Free Plan Limits / Upgrade box (below General/Tags buttons) */}



      <hr className="border-none border-t border-gray-300 my-2.5 mb-3.5" />

      {/* General Settings */}
      {activeTab === "General" && (
        <div className="space-y-4 overflow-y-auto flex-1 pr-2">
          <GeneralSettings
            startAtLogin={startAtLogin}
            setStartAtLogin={setStartAtLogin}
            autoPurgeUnpinned={autoPurgeUnpinned}
            setAutoPurgeUnpinned={setAutoPurgeUnpinned}
            purgeCadence={purgeCadence}
            setPurgeCadence={setPurgeCadence}
            purgeAllUnpinned={purgeAllUnpinned}
          />
        </div>
      )}

      {/* Tags Settings */}
      {activeTab === "Tags" && (
        <TagsSettings />
      )}

      
    </div>
  );
};

export default ClipTraySettings;
