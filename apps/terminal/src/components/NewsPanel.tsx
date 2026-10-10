"use client";

import React, { useState, useEffect, useMemo } from "react";
import {
  Newspaper,
  ExternalLink,
  Flame,
  RefreshCw,
  Clock,
  TrendingUp,
  TrendingDown,
  Search,
  BookOpen,
  Image as ImageIcon,
  Sparkles,
} from "lucide-react";
import { NewsArticle } from "@/types";
import { NewsArticleModal } from "./NewsArticleModal";

interface NewsPanelProps {
  symbol?: string;
  theme?: "dark" | "light";
  onSelectSymbol?: (symbol: string) => void;
}

function timeAgo(dateStr: string): string {
  try {
    const past = new Date(dateStr).getTime();
    const now = Date.now();
    const diffSec = Math.floor((now - past) / 1000);
    if (diffSec < 60) return "Just now";
    const diffMin = Math.floor(diffSec / 60);
    if (diffMin < 60) return `${diffMin}m ago`;
    const diffHour = Math.floor(diffMin / 60);
    if (diffHour < 24) return `${diffHour}h ago`;
    const diffDay = Math.floor(diffHour / 24);
    return `${diffDay}d ago`;
  } catch {
    return "";
  }
}

export const NewsPanel: React.FC<NewsPanelProps> = ({
  symbol,
  theme = "dark",
  onSelectSymbol,
}) => {
  const isLight = theme === "light";
  const [articles, setArticles] = useState<NewsArticle[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState<
    "all" | "high" | "forex" | "crypto" | "commodities" | "bullish" | "bearish"
  >("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedArticle, setSelectedArticle] = useState<NewsArticle | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);

  const fetchNews = async () => {
    setLoading(true);
    try {
      const res = await fetch("/api/news?limit=50");
      if (res.ok) {
        const data = await res.json();
        const mapped: NewsArticle[] = (data.items || []).map((item: any) => ({
          id: item.id || Math.random().toString(),
          title: item.original_title || item.title || "Market Wire",
          source: item.source_name || item.source || "Market Wire",
          url: item.original_url || item.url || "#",
          published_at: item.published_at || new Date().toISOString(),
          impact_level: item.impact_level || "medium",
          sentiment: item.sentiment || "neutral",
          summary: item.summary || "",
          content: item.content || item.original_content || item.summary || "",
          media_url: item.media_url || item.mediaUrl || item.imageUrl || undefined,
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

  const handleOpenArticle = (article: NewsArticle) => {
    setSelectedArticle(article);
    setIsModalOpen(true);
  };

  const filteredArticles = useMemo(() => {
    return articles.filter((a) => {
      // 1. Tab filter
      if (filter === "high" && a.impact_level !== "high") return false;
      if (
        filter === "bullish" &&
        a.sentiment !== "bullish" &&
        a.sentiment !== "positive"
      )
        return false;
      if (
        filter === "bearish" &&
        a.sentiment !== "bearish" &&
        a.sentiment !== "negative"
      )
        return false;

      const titleLower = a.title.toLowerCase();
      const contentLower = (a.content || a.summary || "").toLowerCase();

      if (filter === "forex") {
        const isFx =
          titleLower.includes("eur") ||
          titleLower.includes("usd") ||
          titleLower.includes("gbp") ||
          titleLower.includes("jpy") ||
          titleLower.includes("aud") ||
          titleLower.includes("cad") ||
          titleLower.includes("chf") ||
          titleLower.includes("central bank") ||
          titleLower.includes("fed") ||
          titleLower.includes("rate");
        if (!isFx) return false;
      }

      if (filter === "crypto") {
        const isCrypto =
          titleLower.includes("btc") ||
          titleLower.includes("bitcoin") ||
          titleLower.includes("eth") ||
          titleLower.includes("crypto");
        if (!isCrypto) return false;
      }

      if (filter === "commodities") {
        const isComm =
          titleLower.includes("gold") ||
          titleLower.includes("xau") ||
          titleLower.includes("silver") ||
          titleLower.includes("xag") ||
          titleLower.includes("oil") ||
          titleLower.includes("crude") ||
          titleLower.includes("gas");
        if (!isComm) return false;
      }

      // 2. Search query filter
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        return (
          titleLower.includes(q) ||
          contentLower.includes(q) ||
          a.source.toLowerCase().includes(q)
        );
      }

      return true;
    });
  }, [articles, filter, searchQuery]);

  return (
    <div
      className="w-full flex flex-col h-full select-none text-xs transition-colors bg-card border-border text-foreground"
    >
      {/* Header */}
      <div
        className="h-[44px] border-b border-border flex items-center justify-between px-3 shrink-0 bg-card/60"
      >
        <div className="flex items-center gap-2 font-bold text-sm">
          <Newspaper className="w-4 h-4 text-primary" />
          <span className="text-foreground">
            News Stream
          </span>
          <span
            className="text-[10px] font-mono px-1.5 py-0.2 rounded bg-muted text-muted-foreground"
          >
            {filteredArticles.length}
          </span>
        </div>

        <button
          onClick={fetchNews}
          className="p-1.5 rounded-lg transition-colors cursor-pointer text-muted-foreground hover:text-foreground hover:bg-muted"
          title="Refresh Feed"
        >
          <RefreshCw className={`w-3.5 h-3.5 ${loading ? "animate-spin" : ""}`} />
        </button>
      </div>

      {/* Search Input Bar */}
      <div
        className="px-3 py-2 border-b border-border flex items-center gap-2 bg-background/50"
      >
        <Search className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
        <input
          type="text"
          placeholder="Search news, forex, metals, stocks..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full bg-transparent border-none outline-hidden text-[11px] font-medium text-foreground placeholder-muted-foreground"
        />
        {searchQuery && (
          <button
            onClick={() => setSearchQuery("")}
            className="text-[10px] text-muted-foreground hover:text-foreground"
          >
            Clear
          </button>
        )}
      </div>

      {/* Filter Tabs */}
      <div
        className="flex items-center gap-1 px-3 py-1.5 border-b border-border overflow-x-auto text-[11px] shrink-0 scrollbar-none bg-background/30"
      >
        {(
          [
            { id: "all", label: "All" },
            { id: "high", label: "🔥 High Impact" },
            { id: "forex", label: "Forex" },
            { id: "commodities", label: "Commodities" },
            { id: "crypto", label: "Crypto" },
            { id: "bullish", label: "Bullish" },
            { id: "bearish", label: "Bearish" },
          ] as const
        ).map((t) => (
          <button
            key={t.id}
            onClick={() => setFilter(t.id)}
            className={`px-2.5 py-1 rounded-md whitespace-nowrap font-medium transition-all cursor-pointer ${
              filter === t.id
                ? "bg-primary/15 text-primary border border-primary/30 font-bold shadow-xs"
                : "text-muted-foreground hover:text-foreground hover:bg-muted/60"
            }`}
          >
            {t.label}
          </button>
        ))}
      </div>

      {/* Articles Stream List */}
      <div
        className="flex-1 overflow-y-auto divide-y divide-border/60"
      >
        {loading && articles.length === 0 ? (
          <div className="p-8 text-center text-[#787b86] flex flex-col items-center gap-2">
            <RefreshCw className="w-5 h-5 animate-spin text-[#2962ff]" />
            <span>Streaming market headlines...</span>
          </div>
        ) : filteredArticles.length === 0 ? (
          <div className="p-8 text-center text-[#787b86]">
            No news articles found for this filter.
          </div>
        ) : (
          filteredArticles.map((article) => {
            const isHigh = article.impact_level === "high";
            const ago = timeAgo(article.published_at);
            const isPositive =
              article.sentiment === "bullish" || article.sentiment === "positive";
            const isNegative =
              article.sentiment === "bearish" || article.sentiment === "negative";

            return (
              <div
                key={article.id}
                onClick={() => handleOpenArticle(article)}
                className="p-3.5 transition-all cursor-pointer group flex gap-3 hover:bg-muted/40"
              >
                {/* Left Content Area */}
                <div className="flex-1 min-w-0 flex flex-col justify-between">
                  {/* Metadata Row */}
                  <div className="flex items-center gap-2 mb-1.5 flex-wrap">
                    <span
                      className={`text-[9px] font-mono px-1.5 py-0.5 rounded font-bold uppercase ${
                        isHigh
                          ? "bg-down/15 text-down"
                          : "bg-muted text-muted-foreground"
                      }`}
                    >
                      {article.source}
                    </span>

                    {ago && (
                      <div className="flex items-center gap-1 text-[10px] text-muted-foreground font-mono">
                        <Clock className="w-3 h-3" />
                        <span>{ago}</span>
                      </div>
                    )}

                    {isHigh && <Flame className="w-3 h-3 text-down" />}
                  </div>

                  {/* Title Headline */}
                  <h3 className="font-semibold text-xs leading-snug line-clamp-2 mb-1.5 transition-colors text-foreground group-hover:text-primary">
                    {article.title}
                  </h3>

                  {/* Summary Snippet */}
                  {article.summary && (
                    <p className="text-[11px] text-[#787b86] line-clamp-1 mb-2 leading-relaxed">
                      {article.summary}
                    </p>
                  )}

                  {/* Sentiment & Quick Actions */}
                  <div className="flex items-center justify-between text-[10px] pt-0.5">
                    <div className="flex items-center gap-2">
                      {article.sentiment && (
                        <span
                          className={`capitalize font-semibold inline-flex items-center gap-1 ${
                            isPositive
                              ? "text-[#089981]"
                              : isNegative
                              ? "text-[#f23645]"
                              : "text-[#787b86]"
                          }`}
                        >
                          {isPositive && <TrendingUp className="w-3 h-3" />}
                          {isNegative && <TrendingDown className="w-3 h-3" />}
                          {article.sentiment}
                        </span>
                      )}
                    </div>

                    <div className="flex items-center gap-2 text-[#787b86]">
                      <span className="text-[10px] text-[#2962ff] group-hover:underline flex items-center gap-0.5">
                        <BookOpen className="w-3 h-3" />
                        Read
                      </span>
                      <a
                        href={article.url}
                        target="_blank"
                        rel="noreferrer"
                        onClick={(e) => e.stopPropagation()}
                        className="p-1 hover:text-white hover:bg-black/20 rounded transition-colors"
                        title="Open original link"
                      >
                        <ExternalLink className="w-3 h-3 opacity-60 hover:opacity-100" />
                      </a>
                    </div>
                  </div>
                </div>

                {/* Right Image Thumbnail (If Available) */}
                {article.media_url ? (
                  <div className="w-16 h-16 sm:w-20 sm:h-20 rounded-lg overflow-hidden border border-[#2a2e39]/60 shrink-0 bg-black/20 self-center">
                    <img
                      src={article.media_url}
                      alt={article.title}
                      className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                      onError={(e) => {
                        (e.target as HTMLElement).style.display = "none";
                      }}
                    />
                  </div>
                ) : (
                  <div
                    className={`w-14 h-14 rounded-lg flex items-center justify-center shrink-0 border self-center ${
                      isLight
                        ? "bg-[#f0f3fa] border-[#e0e3eb] text-[#8e929d]"
                        : "bg-[#141720] border-[#2a2e39] text-[#787b86]"
                    }`}
                  >
                    <Newspaper className="w-5 h-5 opacity-40" />
                  </div>
                )}
              </div>
            );
          })
        )}
      </div>

      {/* Institutional Article Modal */}
      <NewsArticleModal
        article={selectedArticle}
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        onSelectSymbol={onSelectSymbol}
        theme={theme}
      />
    </div>
  );
};
