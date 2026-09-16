"use client";

import React, { useState, useEffect, useId } from "react";
import { Tv, ExternalLink, Volume2, VolumeX, Maximize2, Sparkles, Radio } from "lucide-react";

interface StreamChannel {
  id: string;
  name: string;
  category: "news" | "markets" | "macro";
  youtubeId: string;
  description: string;
}

const PRESET_CHANNELS: StreamChannel[] = [
  {
    id: "bloomberg",
    name: "Bloomberg Television",
    category: "markets",
    youtubeId: "dp8PhLsUcFE", // Bloomberg Global Financial News Live
    description: "24/7 Global Business, Markets & Economic News",
  },
  {
    id: "cnbc",
    name: "CNBC International",
    category: "news",
    youtubeId: "9NyxcX14vhk", // CNBC Live Markets Feed
    description: "Live Financial News, Wall St Opening & Closing Bell",
  },
  {
    id: "yahoo_finance",
    name: "Yahoo Finance Live",
    category: "markets",
    youtubeId: "141xLq6wY4k", // Yahoo Finance Live
    description: "US Equity Action, Ticker Breakdown & Earnings Coverage",
  },
  {
    id: "fed_reserve",
    name: "Federal Reserve Live",
    category: "macro",
    youtubeId: "19106093498", // Federal Reserve Channel / FOMC stream
    description: "FOMC Rate Decisions & Fed Chair Press Conferences",
  },
];

interface LiveStreamPanelProps {
  theme?: "dark" | "light";
}

export const LiveStreamPanel: React.FC<LiveStreamPanelProps> = ({ theme = "dark" }) => {
  const isLight = theme === "light";
  const customInputId = useId();
  const [selectedChannel, setSelectedChannel] = useState<string>("bloomberg");
  const [customYoutubeId, setCustomYoutubeId] = useState<string>("");
  const [customInput, setCustomInput] = useState<string>("");
  const [isMuted, setIsMuted] = useState<boolean>(true);
  const [isFullscreen, setIsFullscreen] = useState<boolean>(false);

  // Load last watched stream from localStorage
  useEffect(() => {
    try {
      const saved = localStorage.getItem("atlsd_terminal_live_stream");
      if (saved) setSelectedChannel(saved);
      const savedCustom = localStorage.getItem("atlsd_terminal_live_custom");
      if (savedCustom) {
        setCustomYoutubeId(savedCustom);
        setCustomInput(savedCustom);
      }
    } catch {
      // ignore
    }
  }, []);

  const handleSelectChannel = (id: string) => {
    setSelectedChannel(id);
    try {
      localStorage.setItem("atlsd_terminal_live_stream", id);
    } catch {
      // ignore
    }
  };

  const handleCustomSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const clean = extractYoutubeId(customInput);
    if (clean) {
      setCustomYoutubeId(clean);
      setSelectedChannel("custom");
      try {
        localStorage.setItem("atlsd_terminal_live_stream", "custom");
        localStorage.setItem("atlsd_terminal_live_custom", clean);
      } catch {
        // ignore
      }
    }
  };

  const extractYoutubeId = (urlOrId: string): string => {
    const trimmed = urlOrId.trim();
    if (!trimmed) return "";
    // If it's already an 11-char ID
    if (/^[a-zA-Z0-9_-]{11}$/.test(trimmed)) return trimmed;

    // Matches youtube.com/watch?v=ID, youtu.be/ID, youtube.com/live/ID
    const match = trimmed.match(/(?:youtu\.be\/|youtube\.com\/(?:embed\/|v\/|watch\?v=|live\/|shorts\/))([a-zA-Z0-9_-]{11})/);
    return match ? match[1] : trimmed;
  };

  const activeChannel = PRESET_CHANNELS.find((c) => c.id === selectedChannel);
  const currentVideoId =
    selectedChannel === "custom"
      ? customYoutubeId || PRESET_CHANNELS[0].youtubeId
      : activeChannel?.youtubeId || PRESET_CHANNELS[0].youtubeId;

  return (
    <div
      className={`w-full flex flex-col h-full select-none text-xs transition-colors ${
        isLight ? "bg-[#ffffff] border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-[#d1d4dc]"
      }`}
    >
      {/* Top Header */}
      <div
        className={`h-[44px] border-b flex items-center justify-between px-3 shrink-0 ${
          isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
        }`}
      >
        <div className="flex items-center gap-2">
          <Tv className="w-4 h-4 text-[#f23645]" />
          <span className="font-bold text-sm tracking-wide">Live Broadcast</span>
          <span className="inline-flex items-center gap-1 text-[10px] font-semibold tracking-wider uppercase px-1.5 py-0.5 rounded bg-[#f23645]/15 text-[#f23645]">
            <span className="w-1.5 h-1.5 rounded-full bg-[#f23645] animate-pulse" />
            LIVE
          </span>
        </div>

        <div className="flex items-center gap-1">
          <button
            onClick={() => setIsMuted(!isMuted)}
            className={`p-1.5 rounded transition-colors ${
              isLight ? "hover:bg-[#f0f3fa] text-[#5d606b]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]"
            }`}
            title={isMuted ? "Unmute Audio" : "Mute Audio"}
          >
            {isMuted ? <VolumeX className="w-4 h-4 text-[#f23645]" /> : <Volume2 className="w-4 h-4 text-[#089981]" />}
          </button>
          <a
            href={`https://www.youtube.com/watch?v=${currentVideoId}`}
            target="_blank"
            rel="noopener noreferrer"
            className={`p-1.5 rounded transition-colors ${
              isLight ? "hover:bg-[#f0f3fa] text-[#5d606b]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc]"
            }`}
            title="Open in YouTube"
          >
            <ExternalLink className="w-4 h-4" />
          </a>
        </div>
      </div>

      {/* Channel Switcher Pills */}
      <div
        className={`px-3 py-2 border-b flex items-center gap-1.5 overflow-x-auto shrink-0 scrollbar-none ${
          isLight ? "bg-[#f8f9fd] border-[#e0e3eb]" : "bg-[#181b27] border-[#2a2e39]"
        }`}
      >
        {PRESET_CHANNELS.map((ch) => {
          const isActive = selectedChannel === ch.id;
          return (
            <button
              key={ch.id}
              onClick={() => handleSelectChannel(ch.id)}
              className={`px-2.5 py-1 rounded text-[11px] font-medium whitespace-nowrap transition-all flex items-center gap-1.5 ${
                isActive
                  ? "bg-[#2962ff] text-white shadow-sm"
                  : isLight
                  ? "bg-[#ffffff] text-[#5d606b] hover:text-[#131722] hover:bg-[#f0f3fa] border border-[#e0e3eb]"
                  : "bg-[#1e222d] text-[#787b86] hover:text-[#d1d4dc] hover:bg-[#2a2e39] border border-[#2a2e39]"
              }`}
            >
              <Radio className={`w-3 h-3 ${isActive ? "text-white" : "text-[#787b86]"}`} />
              {ch.name}
            </button>
          );
        })}

        {/* Custom Channel Pill */}
        <button
          onClick={() => handleSelectChannel("custom")}
          className={`px-2.5 py-1 rounded text-[11px] font-medium whitespace-nowrap transition-all flex items-center gap-1.5 ${
            selectedChannel === "custom"
              ? "bg-[#2962ff] text-white shadow-sm"
              : isLight
              ? "bg-[#ffffff] text-[#5d606b] hover:text-[#131722] border border-[#e0e3eb]"
              : "bg-[#1e222d] text-[#787b86] hover:text-[#d1d4dc] border border-[#2a2e39]"
          }`}
        >
          <Sparkles className="w-3 h-3" />
          Custom Feed
        </button>
      </div>

      {/* Custom Stream Input Form (if custom selected) */}
      {selectedChannel === "custom" && (
        <form
          onSubmit={handleCustomSubmit}
          className={`p-2.5 border-b flex items-center gap-2 ${
            isLight ? "bg-[#f0f3fa] border-[#e0e3eb]" : "bg-[#181b27] border-[#2a2e39]"
          }`}
        >
          <label htmlFor={customInputId} className="sr-only">
            YouTube Live URL or Video ID
          </label>
          <input
            id={customInputId}
            type="text"
            value={customInput}
            onChange={(e) => setCustomInput(e.target.value)}
            placeholder="Paste YouTube Live URL or Video ID..."
            className={`flex-1 px-2.5 py-1.5 rounded text-xs outline-none border transition-colors ${
              isLight
                ? "bg-white border-[#e0e3eb] text-[#131722] focus:border-[#2962ff]"
                : "bg-[#131722] border-[#2a2e39] text-[#d1d4dc] focus:border-[#2962ff]"
            }`}
          />
          <button
            type="submit"
            className="px-3 py-1.5 rounded bg-[#2962ff] hover:bg-[#1e53e5] text-white text-xs font-medium transition-colors cursor-pointer"
          >
            Load
          </button>
        </form>
      )}

      {/* Main Video Stream Container */}
      <div className="flex-1 min-h-0 relative bg-black flex flex-col items-center justify-center overflow-hidden">
        {currentVideoId ? (
          <iframe
            key={`${currentVideoId}-${isMuted}`}
            src={`https://www.youtube-nocookie.com/embed/${currentVideoId}?autoplay=1&mute=${
              isMuted ? "1" : "0"
            }&playsinline=1&rel=0&modestbranding=1&enablejsapi=1`}
            title={activeChannel?.name || "YouTube Live Stream"}
            allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
            allowFullScreen
            className="w-full h-full border-0 absolute inset-0"
          />
        ) : (
          <div className="flex flex-col items-center gap-2 text-center p-6 text-[#787b86]">
            <Tv className="w-8 h-8 opacity-40" />
            <p className="font-medium">No live stream selected</p>
            <p className="text-[11px] opacity-70">Pick a financial news stream above or enter a custom YouTube URL.</p>
          </div>
        )}
      </div>

      {/* Stream Info Footer */}
      <div
        className={`p-3 border-t flex flex-col gap-1 shrink-0 ${
          isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
        }`}
      >
        <div className="flex items-center justify-between">
          <div className="font-bold text-xs truncate">
            {selectedChannel === "custom" ? "Custom Financial Broadcast" : activeChannel?.name}
          </div>
          <span className="text-[10px] text-[#787b86] shrink-0 font-mono">
            {isMuted ? "Audio Muted (Click speaker to listen)" : "Audio Active"}
          </span>
        </div>
        <div className="text-[11px] text-[#787b86] truncate">
          {selectedChannel === "custom"
            ? `Active Video ID: ${customYoutubeId || "None"}`
            : activeChannel?.description}
        </div>
      </div>
    </div>
  );
};
