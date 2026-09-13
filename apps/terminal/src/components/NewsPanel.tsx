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
  theme?: "dark" | "light";
}

export const NewsPanel: React.FC<NewsPanelProps> = ({ symbol, theme = "dark" }) => {
  const isLight = theme === "light";
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
        }));
        setArticles(mapped);
      }
    } catch {
      // ignore
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchNews();
    const interval = setInterval(fetchNews, 30000);
    return () => clearInterval(interval);
  }, []);

  const filteredArticles = articles.filter((a) => {
    if (filter === "high") return a.impact_level === "high";
    return true;
  });

  return (
    <div
      className={`w-full flex flex-col h-full select-none text-xs transition-colors ${
        isLight ? "bg-[#ffffff] border-[#e0e3eb] text-[#131722]" : "bg-[#1e222d] border-[#2a2e39] text-[#d1d4dc]"
      }`}
    >
      {/* Header */}
      <div
        className={`h-[44px] border-b flex items-center justify-between px-3 shrink-0 ${
          isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
        }`}
      >
        <div className="flex items-center gap-1.5 font-bold text-sm">
          <Newspaper className="w-4 h-4 text-[#2962ff]" />
          <span className={isLight ? "text-[#131722]" : "text-white"}>News Stream</span>
        </div>
        <button
          onClick={fetchNews}
          className={`p-1.5 rounded transition-colors cursor-pointer ${
            isLight ? "hover:bg-[#f0f3fa] text-[#5d606b] hover:text-[#131722]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-white"
          }`}
          title="Refresh Feed"
        >
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? "animate-spin" : ""}`} />
        </button>
      </div>

      {/* Filter Tabs */}
      <div
        className={`flex items-center gap-1 px-3 py-1.5 border-b overflow-x-auto text-[11px] shrink-0 ${
          isLight ? "bg-[#f0f3fa] border-[#e0e3eb]" : "bg-[#181b27] border-[#2a2e39]"
        }`}
      >
        {(["all", "high", "forex", "crypto"] as const).map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={`px-2 py-0.5 rounded capitalize font-medium transition-colors cursor-pointer ${
              filter === f
                ? isLight
                  ? "bg-[#ffffff] text-[#131722] font-bold shadow-xs"
                  : "bg-[#2a2e39] text-white font-bold shadow-xs"
                : isLight
                ? "text-[#5d606b] hover:text-[#131722]"
                : "text-[#787b86] hover:text-[#d1d4dc]"
            }`}
          >
            {f === "high" ? (
              <span className="flex items-center gap-1.5">
                <span className="w-1.5 h-1.5 rounded-full bg-[#f23645]" />
                <span>High Impact</span>
              </span>
            ) : (
              f
            )}
          </button>
        ))}
      </div>

      {/* Articles Stream */}
      <div className={`flex-1 overflow-y-auto divide-y ${isLight ? "divide-[#e0e3eb]" : "divide-[#2a2e39]/50"}`}>
        {loading && articles.length === 0 ? (
          <div className="p-8 text-center text-[#787b86]">Loading market news...</div>
        ) : filteredArticles.length === 0 ? (
          <div className="p-8 text-center text-[#787b86]">No news articles available</div>
        ) : (
          filteredArticles.map((article) => {
            const isHigh = article.impact_level === "high";
            const date = new Date(article.published_at);
            const timeStr = date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });

            return (
              <a
                key={article.id}
                href={article.url}
                target="_blank"
                rel="noreferrer"
                className={`block p-3 transition-colors cursor-pointer ${
                  isLight ? "hover:bg-[#f8f9fc]" : "hover:bg-[#262b37]"
                }`}
              >
                <div className="flex items-center justify-between mb-1.5">
                  <div className="flex items-center gap-1.5">
                    <span
                      className={`text-[9px] font-mono px-1.5 py-0.2 rounded font-bold uppercase ${
                        isHigh
                          ? "bg-[#f23645]/15 text-[#f23645]"
                          : isLight
                          ? "bg-[#f0f3fa] text-[#5d606b]"
                          : "bg-[#141722] text-[#787b86]"
                      }`}
                    >
                      {article.source}
                    </span>
                    {isHigh && <Flame className="w-3 h-3 text-[#f23645]" />}
                  </div>
                  <div className="flex items-center gap-1 text-[10px] text-[#787b86] font-mono">
                    <Clock className="w-3 h-3" />
                    <span>{timeStr}</span>
                  </div>
                </div>

                <div
                  className={`font-semibold text-xs leading-snug line-clamp-2 mb-1.5 ${
                    isLight ? "text-[#131722] hover:text-[#2962ff]" : "text-[#d1d4dc] hover:text-white"
                  }`}
                >
                  {article.title}
                </div>

                <div className="flex items-center justify-between text-[10px] text-[#787b86]">
                  <span
                    className={`capitalize font-medium ${
                      article.sentiment === "bullish"
                        ? "text-[#089981]"
                        : article.sentiment === "bearish"
                        ? "text-[#f23645]"
                        : "text-[#787b86]"
                    }`}
                  >
                    {article.sentiment}
                  </span>
                  <ExternalLink className="w-3 h-3 opacity-60" />
                </div>
              </a>
            );
          })
        )}
      </div>
    </div>
  );
};
