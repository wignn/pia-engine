"use client";
import React, { useEffect, useState, useCallback } from "react";
import { ExternalLink, MessageCircle, Repeat2, Heart, RefreshCw } from "lucide-react";

export interface SocialPostItem {
  event_id: string;
  post_id: string;
  platform: string;
  source_account: string;
  author_username: string;
  author_display_name: string;
  text: string;
  url: string;
  created_at: string;
  reply_count: number;
  retweet_count: number;
  like_count: number;
  media_urls?: string[];
}

interface SocialApiResponse {
  items?: SocialPostItem[];
  next_before?: string | null;
  has_more?: boolean;
}

interface SocialPanelProps {
  theme?: "dark" | "light";
}

function formatDate(isoString: string): string {
  try {
    const d = new Date(isoString);
    if (isNaN(d.getTime())) return isoString;
    return new Intl.DateTimeFormat(undefined, {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    }).format(d);
  } catch {
    return isoString;
  }
}

export const SocialPanel: React.FC<SocialPanelProps> = ({ theme = "dark" }) => {
  const isLight = theme === "light";
  const [posts, setPosts] = useState<SocialPostItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const [nextBefore, setNextBefore] = useState<string | null>(null);
  const [hasMore, setHasMore] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchPosts = useCallback(async (before?: string) => {
    const isInitial = !before;
    if (isInitial) {
      setLoading(true);
      setError(null);
    } else {
      setLoadingMore(true);
    }

    try {
      const url = new URL("/api/social", window.location.origin);
      url.searchParams.set("limit", "25");
      if (before) {
        url.searchParams.set("before", before);
      }

      const res = await fetch(url.toString());
      if (!res.ok) {
        throw new Error(`HTTP ${res.status}: Failed to load social feed`);
      }

      const data: SocialApiResponse = await res.json();
      const newItems = data.items ?? [];

      if (isInitial) {
        setPosts(newItems);
      } else {
        setPosts((prev) => {
          const seen = new Set(prev.map((p) => p.post_id));
          const unique = newItems.filter((p) => !seen.has(p.post_id));
          return [...prev, ...unique];
        });
      }

      setNextBefore(data.next_before ?? null);
      setHasMore(Boolean(data.has_more && data.next_before));
    } catch (err: any) {
      setError(err?.message || "Failed to load social posts");
    } finally {
      setLoading(false);
      setLoadingMore(false);
    }
  }, []);

  useEffect(() => {
    fetchPosts();
    const interval = setInterval(() => {
      fetchPosts();
    }, 45000);
    return () => clearInterval(interval);
  }, [fetchPosts]);

  return (
    <aside
      className={`flex h-full w-full flex-col overflow-hidden text-xs transition-colors ${
        isLight ? "bg-[#ffffff] text-[#131722]" : "bg-[#1e222d] text-[#d1d4dc]"
      }`}
    >
      {/* Header */}
      <div
        className={`flex h-11 items-center justify-between border-b px-3 shrink-0 ${
          isLight ? "bg-[#ffffff] border-[#e0e3eb]" : "bg-[#1e222d] border-[#2a2e39]"
        }`}
      >
        <div className="flex items-center gap-2">
          <span className="relative flex h-2 w-2">
            <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-400 opacity-75" />
            <span className="relative inline-flex h-2 w-2 rounded-full bg-emerald-500" />
          </span>
          <span className={`text-sm font-bold ${isLight ? "text-[#131722]" : "text-white"}`}>
            Social Pulse
          </span>
          <span className="text-[10px] text-[#787b86]">X/Twitter</span>
        </div>

        <button
          onClick={() => fetchPosts()}
          disabled={loading}
          className={`p-1.5 rounded transition-colors cursor-pointer ${
            isLight ? "hover:bg-[#f0f3fa] text-[#5d606b] hover:text-[#131722]" : "hover:bg-[#2a2e39] text-[#787b86] hover:text-white"
          }`}
          title="Refresh Feed"
        >
          <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
        </button>
      </div>

      {/* Feed List */}
      <div className="flex-1 overflow-y-auto p-3">
        {error && (
          <div className="mb-3 rounded-lg border border-red-500/20 bg-red-500/10 p-2.5 text-[11px] text-red-400">
            {error}
          </div>
        )}

        {loading && posts.length === 0 ? (
          <div className="space-y-3">
            {[1, 2, 3].map((i) => (
              <div
                key={i}
                className={`animate-pulse rounded-lg border p-3 ${
                  isLight ? "border-[#e0e3eb] bg-[#f8f9fc]" : "border-[#2a2e39] bg-[#141722]"
                }`}
              >
                <div className={`h-4 w-1/3 rounded ${isLight ? "bg-[#e0e3eb]" : "bg-[#2a2e39]"}`} />
                <div className={`mt-2 h-10 rounded ${isLight ? "bg-[#e0e3eb]" : "bg-[#2a2e39]"}`} />
              </div>
            ))}
          </div>
        ) : posts.length === 0 ? (
          <div className="flex h-48 flex-col items-center justify-center text-center text-[#787b86]">
            <p>No social posts captured yet.</p>
            <p className="mt-1 text-[11px] opacity-75">
              Live tweets from financial sources will appear here.
            </p>
          </div>
        ) : (
          <div className="space-y-2.5">
            {posts.map((item) => (
              <div
                key={item.post_id}
                className={`rounded-lg border p-3 transition-colors ${
                  isLight
                    ? "border-[#e0e3eb] bg-[#f8f9fc] hover:border-[#2962ff]/40"
                    : "border-[#2a2e39] bg-[#141722] hover:border-[#2962ff]/40"
                }`}
              >
                <div className="flex items-start justify-between gap-2">
                  <div className="min-w-0">
                    <div className="flex items-center gap-1.5 font-medium">
                      <span className={`truncate text-[12px] font-bold ${isLight ? "text-[#131722]" : "text-white"}`}>
                        {item.author_display_name || item.author_username}
                      </span>
                      <span className="text-[11px] text-[#787b86]">
                        @{item.author_username}
                      </span>
                    </div>
                    <div className="text-[10px] text-[#787b86]">
                      {formatDate(item.created_at)}
                    </div>
                  </div>

                  <a
                    href={item.url}
                    target="_blank"
                    rel="noreferrer"
                    className="p-1 text-[#787b86] transition-colors hover:text-[#2962ff]"
                    title="Open on X"
                  >
                    <ExternalLink className="h-3.5 w-3.5" />
                  </a>
                </div>

                <p className={`mt-2 whitespace-pre-wrap break-words leading-relaxed ${isLight ? "text-[#131722]" : "text-[#d1d4dc]"}`}>
                  {item.text}
                </p>

                {Array.isArray(item.media_urls) && item.media_urls.length > 0 && (
                  <div
                    className={`mt-2.5 grid gap-1.5 ${
                      item.media_urls.length > 1 ? "grid-cols-2" : "grid-cols-1"
                    }`}
                  >
                    {item.media_urls.slice(0, 4).map((url, idx) => (
                      <a
                        key={idx}
                        href={url}
                        target="_blank"
                        rel="noreferrer"
                        className={`group relative block overflow-hidden rounded border ${
                          isLight ? "border-[#e0e3eb] bg-[#f0f3fa]" : "border-[#2a2e39] bg-[#131722]"
                        }`}
                      >
                        <img
                          src={url}
                          alt="Media attachment"
                          loading="lazy"
                          referrerPolicy="no-referrer"
                          className="max-h-48 w-full object-cover transition-transform duration-200 group-hover:scale-105"
                        />
                      </a>
                    ))}
                  </div>
                )}

                <div
                  className={`mt-2.5 flex items-center gap-3.5 border-t pt-2 text-[10px] text-[#787b86] ${
                    isLight ? "border-[#e0e3eb]" : "border-[#2a2e39]/60"
                  }`}
                >
                  <span className="flex items-center gap-1">
                    <MessageCircle className="h-3 w-3" />
                    {item.reply_count ?? 0}
                  </span>
                  <span className="flex items-center gap-1">
                    <Repeat2 className="h-3 w-3" />
                    {item.retweet_count ?? 0}
                  </span>
                  <span className="flex items-center gap-1">
                    <Heart className="h-3 w-3" />
                    {item.like_count ?? 0}
                  </span>
                </div>
              </div>
            ))}

            {hasMore && (
              <button
                onClick={() => nextBefore && fetchPosts(nextBefore)}
                disabled={loadingMore}
                className={`w-full rounded-lg border py-2 text-center text-[11px] font-medium transition-colors cursor-pointer ${
                  isLight
                    ? "border-[#e0e3eb] bg-[#f8f9fc] text-[#5d606b] hover:bg-[#f0f3fa] hover:text-[#131722]"
                    : "border-[#2a2e39] bg-[#141722] text-[#787b86] hover:bg-[#2a2e39] hover:text-white"
                }`}
              >
                {loadingMore ? "Loading earlier posts..." : "Load Earlier Posts"}
              </button>
            )}
          </div>
        )}
      </div>
    </aside>
  );
};
