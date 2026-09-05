"use client";

import React, { useState, useEffect } from "react";
import { 
  Newspaper, 
  ExternalLink, 
  Flame, 
  RefreshCw, 
  Filter, 
  Clock, 
  ChevronRight,
  TrendingUp,
  TrendingDown,
  Globe
} from "lucide-react";
import { NewsArticle } from "@/types";

interface NewsPanelProps {
  symbol: string;
}

export const NewsPanel: React.FC<NewsPanelProps> = ({ symbol }) => {
  const [articles, setArticles] = useState<NewsArticle[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState<"all" | "high" | "forex" | "crypto">("all");

  const fetchNews = async () => {
    setLoading(true);
    try {
      const res = await fetch("/api/news");
      if (res.ok) {
        const data = await res.json();
        const mapped: NewsArticle[] = (data.items || []).map((item: any) => ({
          id: item.id || Math.random().toString(),
          title: item.original_title || item.title,
          source: item.source_name || "Market Wire",
          url: item.original_url || item.url || "#",
          published_at: item.published_at || new Date().toISOString(),
          impact_level: item.impact_level || "medium",
          sentiment: item.sentiment || "neutral",
          summary: item.summary || item.translated_title || "",
        }));
        setArticles(mapped);
      }
    } catch (e) {
      console.error("Failed to fetch news", e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchNews();
    const interval = setInterval(fetchNews, 60000);
    return () => clearInterval(interval);
  }, []);

  const filtered = articles.filter((a) => {
    if (filter === "high") return a.impact_level === "high";
    return true;
  });

  return (
    <div className="flex flex-col h-full bg-[#1e222d] text-xs text-[#d1d4dc] select-none">
      {/* Top Header */}
      <div className="h-[46px] border-b border-[#2a2e39] flex items-center justify-between px-3">
        <div className="flex items-center gap-2">
          <div className="w-5 h-5 rounded bg-[#f23645]/20 flex items-center justify-center text-[#f23645]">
            <Flame className="w-3.5 h-3.5" />
          </div>
          <span className="font-bold text-sm text-white">Live News & Calendar</span>
        </div>
        <div className="flex items-center gap-1">
          <button 
            onClick={fetchNews} 
            className={`p-1.5 rounded hover:bg-[#2a2e39] text-[#787b86] hover:text-[#d1d4dc] transition-colors ${loading ? "animate-spin" : ""}`}
            title="Refresh news"
          >
            <RefreshCw className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Filter Tabs */}
      <div className="flex items-center gap-1 px-3 py-1.5 border-b border-[#2a2e39] bg-[#181b27]">
        {(["all", "high"] as const).map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={`px-2 py-0.5 rounded capitalize font-medium transition-colors ${
              filter === f ? "bg-[#2a2e39] text-white font-bold" : "text-[#787b86] hover:text-[#d1d4dc]"
            }`}
          >
            {f === "high" ? "🔥 High Impact" : "All Feed"}
          </button>
        ))}
      </div>

      {/* News Feed Stream */}
      <div className="flex-1 overflow-y-auto divide-y divide-[#2a2e39]/50">
        {loading && articles.length === 0 ? (
          <div className="p-8 text-center text-[#787b86] flex flex-col items-center gap-2">
            <RefreshCw className="w-5 h-5 animate-spin" />
            <span>Fetching live market headlines...</span>
          </div>
        ) : filtered.length === 0 ? (
          <div className="p-8 text-center text-[#787b86]">No news articles found</div>
        ) : (
          filtered.map((item) => {
            const isHigh = item.impact_level === "high";

            return (
              <a
                key={item.id}
                href={item.url}
                target="_blank"
                rel="noopener noreferrer"
                className="p-3 flex flex-col gap-1.5 hover:bg-[#262b37] transition-colors group block"
              >
                <div className="flex items-center justify-between text-[10px] text-[#787b86]">
                  <span className="font-semibold text-[#2962ff] uppercase tracking-wider">{item.source}</span>
                  <div className="flex items-center gap-1">
                    <Clock className="w-3 h-3" />
                    <span>{new Date(item.published_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}</span>
                  </div>
                </div>

                <h4 className="text-[12px] font-medium text-white group-hover:text-[#2962ff] transition-colors leading-snug line-clamp-2">
                  {item.title}
                </h4>

                {item.summary && (
                  <p className="text-[11px] text-[#787b86] line-clamp-2 leading-relaxed">
                    {item.summary}
                  </p>
                )}

                <div className="flex items-center justify-between mt-1 pt-1 border-t border-[#2a2e39]/40">
                  <span
                    className={`text-[9px] font-bold uppercase px-1.5 py-0.5 rounded ${
                      isHigh ? "bg-[#f23645]/15 text-[#f23645]" : "bg-[#2a2e39] text-[#787b86]"
                    }`}
                  >
                    {item.impact_level || "Medium"} Impact
                  </span>

                  <span className="flex items-center gap-1 text-[10px] text-[#787b86] group-hover:text-white transition-colors">
                    Read <ExternalLink className="w-2.5 h-2.5" />
                  </span>
                </div>
              </a>
            );
          })
        )}
      </div>
    </div>
  );
};
