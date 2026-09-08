"use client";
import React, { useEffect, useState, useCallback } from "react";
import { ExternalLink, MessageCircle, Repeat2, Heart } from "lucide-react";

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

export const SocialPanel: React.FC = () => {
  const [items, setItems] = useState<SocialPostItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [hasMore, setHasMore] = useState(false);
  const [nextBefore, setNextBefore] = useState<string | null>(null);

  const fetchPosts = useCallback(async (before?: string | null) => {
    const isInitial = !before;
    if (isInitial) setLoading(true);
    else setLoadingMore(true);

    try {
      const params = new URLSearchParams({ platform: "twitter", limit: "30" });
      if (before) params.set("before", before);

      const res = await fetch(`/api/social?${params.toString()}`, { cache: "no-store" });
      if (!res.ok) throw new Error("Fetch failed");
      const data: SocialApiResponse = await res.json();

      const newItems = Array.isArray(data.items) ? data.items : [];
      setItems((prev) => {
        if (isInitial) return newItems;
        const seen = new Set(prev.map((p) => p.event_id));
        return [...prev, ...newItems.filter((p) => !seen.has(p.event_id))];
      });

      setNextBefore(data.next_before ?? null);
      setHasMore(Boolean(data.has_more && data.next_before));
    } catch {
      if (isInitial) setItems([]);
    } finally {
      if (isInitial) setLoading(false);
      else setLoadingMore(false);
    }
  }, []);

  useEffect(() => {
    fetchPosts();
  }, [fetchPosts]);

  return (
    <aside className="flex h-full flex-col overflow-hidden border-l border-[#2a2e39] bg-[#1e222d] text-xs text-[#d1d4dc]">
      <div className="flex shrink-0 items-center justify-between border-b border-[#2a2e39] px-3 py-2.5">
        <div>
          <div className="font-bold text-white">Social pulse</div>
          <div className="text-[10px] text-[#787b86]">Twitter/X posts collected by ATLSD</div>
        </div>
        <span className="rounded border border-[#2a2e39] bg-[#141722] px-1.5 py-0.5 text-[9px] font-semibold uppercase text-[#787b86]">
          Realtime
        </span>
      </div>

      <div className="flex-1 overflow-y-auto p-3">
        {loading ? (
          <div className="py-8 text-center text-[11px] text-[#787b86]">Loading social posts…</div>
        ) : items.length === 0 ? (
          <div className="rounded border border-[#2a2e39] bg-[#181b27] p-3 text-center text-[11px] text-[#787b86]">
            No social posts available.
          </div>
        ) : (
          <div className="space-y-2.5">
            {items.map((item) => (
              <article
                key={item.event_id}
                className="rounded border border-[#2a2e39] bg-[#181b27] p-2.5 transition-colors hover:border-[#363a45]"
              >
                <div className="flex items-start justify-between gap-2">
                  <div className="min-w-0">
                    <div className="truncate font-semibold text-white">
                      {item.author_display_name || item.author_username}
                    </div>
                    <div className="text-[10px] text-[#787b86]">
                      @{item.author_username} · {formatDate(item.created_at)}
                    </div>
                  </div>
                  <a
                    href={item.url}
                    target="_blank"
                    rel="noreferrer"
                    className="shrink-0 p-0.5 text-[#787b86] transition-colors hover:text-white"
                    title="Open on X"
                  >
                    <ExternalLink className="h-3.5 w-3.5" />
                  </a>
                </div>

                <p className="mt-2 whitespace-pre-wrap break-words leading-relaxed text-[#d1d4dc]">
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
                        className="group relative block overflow-hidden rounded border border-[#2a2e39] bg-[#131722]"
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

                <div className="mt-2.5 flex items-center gap-3.5 border-t border-[#2a2e39]/60 pt-2 text-[10px] text-[#787b86]">
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
              </article>
            ))}

            <div className="pt-2 text-center">
              {hasMore ? (
                <button
                  type="button"
                  disabled={loadingMore}
                  onClick={() => fetchPosts(nextBefore)}
                  className="w-full rounded border border-[#2a2e39] bg-[#181b27] py-1.5 text-[11px] font-medium text-[#d1d4dc] transition-colors hover:border-[#363a45] hover:text-white disabled:opacity-50"
                >
                  {loadingMore ? "Loading history…" : "Load older posts"}
                </button>
              ) : (
                <span className="text-[10px] text-[#787b86]">End of social history</span>
              )}
            </div>
          </div>
        )}
      </div>
    </aside>
  );
};
