"use client";
import React, { useEffect, useState } from "react";
import { ExternalLink, MessageCircle, Repeat2, Heart } from "lucide-react";

export const SocialPanel: React.FC = () => {
  const [items, setItems] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  useEffect(() => { fetch("/api/social?platform=twitter&limit=30", { cache: "no-store" }).then((r) => r.json()).then((p) => setItems(p.items ?? [])).catch(() => setItems([])).finally(() => setLoading(false)); }, []);
  return <aside className="h-full overflow-y-auto bg-[#1e222d] border-l border-[#2a2e39] p-3 text-xs text-[#d1d4dc]"><div className="mb-3"><div className="font-bold text-white">Social pulse</div><div className="text-[10px] text-[#787b86]">Twitter/X posts collected by ATLSD</div></div>{loading ? <div className="text-[#787b86]">Loading social posts…</div> : items.length === 0 ? <div className="rounded border border-[#2a2e39] bg-[#181b27] p-3 text-[11px] text-[#787b86]">No social posts available.</div> : <div className="space-y-2">{items.map((item) => <article key={item.event_id} className="rounded border border-[#2a2e39] bg-[#181b27] p-2.5"><div className="flex justify-between gap-2"><div><div className="font-semibold text-white">{item.author_display_name || item.author_username}</div><div className="text-[10px] text-[#787b86]">@{item.author_username} · {new Date(item.created_at).toLocaleString()}</div></div><a href={item.url} target="_blank" rel="noreferrer" className="text-[#787b86] hover:text-white"><ExternalLink className="h-3.5 w-3.5" /></a></div><p className="mt-2 whitespace-pre-wrap leading-4 text-[#d1d4dc]">{item.text}</p><div className="mt-2 flex gap-3 text-[10px] text-[#787b86]"><span><MessageCircle className="mr-1 inline h-3 w-3" />{item.reply_count}</span><span><Repeat2 className="mr-1 inline h-3 w-3" />{item.retweet_count}</span><span><Heart className="mr-1 inline h-3 w-3" />{item.like_count}</span></div></article>)}</div>}</aside>;
};
