"use client";

import React, { useMemo } from "react";
import {
  X,
  ExternalLink,
  Clock,
  Flame,
  TrendingUp,
  TrendingDown,
  Share2,
  Copy,
  Check,
  Globe,
  Sparkles,
  BookOpen,
  ArrowRight,
  ShieldAlert,
} from "lucide-react";
import { NewsArticle } from "@/types";

interface NewsArticleModalProps {
  article: NewsArticle | null;
  isOpen: boolean;
  onClose: () => void;
  onSelectSymbol?: (symbol: string) => void;
  theme?: "dark" | "light";
}

// Helper to decode HTML entities
function decodeHtml(html: string): string {
  if (!html) return "";
  return html
    .replace(/&nbsp;/g, " ")
    .replace(/&#8230;/g, "…")
    .replace(/&#8217;/g, "’")
    .replace(/&#8216;/g, "‘")
    .replace(/&#8220;/g, "“")
    .replace(/&#8221;/g, "”")
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'");
}

// Extract tickers mentioned in text
function extractTickers(text: string): string[] {
  if (!text) return [];
  const matches = text.match(/\b(EUR\/GBP|EUR\/USD|GBP\/USD|USD\/JPY|AUD\/USD|USD\/CAD|USD\/CHF|NZD\/USD|XAU\/USD|XAG\/USD|XAUUSD|EURUSD|GBPUSD|USDJPY|BTCUSD|BTCUSDT|ETHUSD|EURGBP|USOIL|UKOIL|SPX|DXY|AAPL|TSLA|NVDA)\b/gi);
  if (!matches) return [];
  const normalized = matches.map((m) => m.replace("/", "").toUpperCase());
  return Array.from(new Set(normalized)).slice(0, 5);
}

export const NewsArticleModal: React.FC<NewsArticleModalProps> = ({
  article,
  isOpen,
  onClose,
  onSelectSymbol,
  theme = "dark",
}) => {
  const [copied, setCopied] = React.useState(false);
  const isLight = theme === "light";

  const rawContent = useMemo(
    () => decodeHtml(article?.content || article?.summary || ""),
    [article?.content, article?.summary]
  );
  const cleanSummary = useMemo(
    () => decodeHtml(article?.summary || ""),
    [article?.summary]
  );

  // Parse bullet points / key takeaways
  const { bullets, paragraphs } = useMemo(() => {
    if (!rawContent) return { bullets: [], paragraphs: [] };

    // Split text into lines or sentences
    const lines = rawContent
      .split(/\n+/)
      .map((l) => l.trim())
      .filter((l) => l.length > 0);

    const extractedBullets: string[] = [];
    const regularParagraphs: string[] = [];

    for (const line of lines) {
      if (
        line.startsWith("•") ||
        line.startsWith("-") ||
        line.startsWith("*") ||
        line.startsWith("–")
      ) {
        extractedBullets.push(line.replace(/^[•\-*–]\s*/, ""));
      } else if (
        extractedBullets.length < 3 &&
        line.length > 25 &&
        line.length < 160 &&
        lines.length > 2 &&
        !line.toLowerCase().includes("faq") &&
        !line.toLowerCase().includes("disclaimer")
      ) {
        // Many financial news feeds (like FXStreet) put 3 short summary punchlines at the start
        extractedBullets.push(line);
      } else {
        regularParagraphs.push(line);
      }
    }

    // If no bullets were found, extract 2 key points from summary or first paragraph
    if (extractedBullets.length === 0 && cleanSummary) {
      const sentences = cleanSummary
        .split(/(?<=[.?!])\s+/)
        .filter((s) => s.length > 20);
      if (sentences.length > 1) {
        extractedBullets.push(...sentences.slice(0, 3));
      }
    }

    return {
      bullets: extractedBullets.slice(0, 4),
      paragraphs: regularParagraphs,
    };
  }, [rawContent, cleanSummary]);

  const tickers = useMemo(
    () => (article ? extractTickers(`${article.title} ${rawContent}`) : []),
    [article, rawContent]
  );

  const formattedDate = useMemo(() => {
    if (!article?.published_at) return "";
    try {
      const d = new Date(article.published_at);
      return d.toLocaleDateString("en-US", {
        day: "numeric",
        month: "short",
        year: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch {
      return article.published_at;
    }
  }, [article?.published_at]);

  if (!isOpen || !article) return null;

  const isHigh = article.impact_level === "high";
  const isPositive =
    article.sentiment === "bullish" || article.sentiment === "positive";
  const isNegative =
    article.sentiment === "bearish" || article.sentiment === "negative";

  const handleCopy = () => {
    if (article.url) {
      navigator.clipboard.writeText(article.url);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-3 sm:p-4 bg-black/75 backdrop-blur-sm animate-in fade-in duration-200"
      onClick={onClose}
    >
      <div
        className={`w-full max-w-[760px] max-h-[90vh] rounded-2xl shadow-2xl flex flex-col overflow-hidden border transition-colors ${
          isLight
            ? "bg-[#ffffff] border-[#e0e3eb] text-[#131722]"
            : "bg-[#181b24] border-[#2a2e39] text-[#d1d4dc]"
        }`}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Top Header Bar */}
        <div
          className={`px-4 sm:px-6 py-3.5 border-b flex items-center justify-between gap-3 shrink-0 ${
            isLight
              ? "bg-[#f8f9fc] border-[#e0e3eb]"
              : "bg-[#141720] border-[#2a2e39]"
          }`}
        >
          <div className="flex items-center gap-2 overflow-hidden text-xs">
            <span
              className={`font-mono font-bold px-2 py-0.5 rounded text-[10px] tracking-wide uppercase ${
                isLight
                  ? "bg-[#2962ff]/10 text-[#2962ff]"
                  : "bg-[#2962ff]/20 text-[#2962ff]"
              }`}
            >
              {article.source || "FINANCIAL WIRE"}
            </span>
            <span className="text-[#787b86] hidden sm:inline">•</span>
            <div className="flex items-center gap-1 text-[11px] text-[#787b86]">
              <Clock className="w-3.5 h-3.5" />
              <span>{formattedDate}</span>
            </div>
            {isHigh && (
              <span className="flex items-center gap-1 font-bold text-[10px] px-2 py-0.5 rounded bg-[#f23645]/15 text-[#f23645]">
                <Flame className="w-3 h-3 text-[#f23645]" />
                HIGH IMPACT
              </span>
            )}
          </div>

          <div className="flex items-center gap-1.5 shrink-0">
            <button
              onClick={handleCopy}
              className={`p-1.5 rounded-lg transition-colors cursor-pointer ${
                isLight
                  ? "hover:bg-[#e8ebf2] text-[#5d606b]"
                  : "hover:bg-[#252a36] text-[#787b86] hover:text-white"
              }`}
              title="Copy URL"
            >
              {copied ? (
                <Check className="w-4 h-4 text-[#089981]" />
              ) : (
                <Copy className="w-4 h-4" />
              )}
            </button>
            <a
              href={article.url}
              target="_blank"
              rel="noreferrer"
              className={`p-1.5 rounded-lg transition-colors cursor-pointer ${
                isLight
                  ? "hover:bg-[#e8ebf2] text-[#5d606b]"
                  : "hover:bg-[#252a36] text-[#787b86] hover:text-white"
              }`}
              title="Open Original Source"
            >
              <ExternalLink className="w-4 h-4" />
            </a>
            <button
              onClick={onClose}
              className={`p-1.5 rounded-lg transition-colors cursor-pointer ml-1 ${
                isLight
                  ? "hover:bg-[#e8ebf2] text-[#5d606b]"
                  : "hover:bg-[#252a36] text-[#787b86] hover:text-white"
              }`}
              title="Close (Esc)"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Scrollable Content Body */}
        <div className="flex-1 overflow-y-auto px-4 sm:px-6 py-5 space-y-5">
          {/* Headline */}
          <div>
            <h1
              className={`text-lg sm:text-xl font-extrabold leading-snug tracking-tight mb-3 ${
                isLight ? "text-[#131722]" : "text-white"
              }`}
            >
              {decodeHtml(article.title)}
            </h1>

            {/* Badges / Sentiment & Related Assets */}
            <div className="flex flex-wrap items-center gap-2 pt-1">
              {article.sentiment && (
                <span
                  className={`inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[11px] font-semibold capitalize ${
                    isPositive
                      ? "bg-[#089981]/15 text-[#089981]"
                      : isNegative
                      ? "bg-[#f23645]/15 text-[#f23645]"
                      : isLight
                      ? "bg-[#f0f3fa] text-[#5d606b]"
                      : "bg-[#252a36] text-[#787b86]"
                  }`}
                >
                  {isPositive && <TrendingUp className="w-3.5 h-3.5" />}
                  {isNegative && <TrendingDown className="w-3.5 h-3.5" />}
                  {article.sentiment} Sentiment
                </span>
              )}

              {tickers.length > 0 && (
                <div className="flex items-center gap-1.5 flex-wrap">
                  <span className="text-[11px] text-[#787b86]">Symbols:</span>
                  {tickers.map((sym) => (
                    <button
                      key={sym}
                      onClick={() => onSelectSymbol?.(sym)}
                      className={`px-2 py-0.5 rounded text-[11px] font-mono font-bold transition-all cursor-pointer ${
                        isLight
                          ? "bg-[#f0f3fa] hover:bg-[#2962ff] hover:text-white text-[#2962ff]"
                          : "bg-[#252a36] hover:bg-[#2962ff] hover:text-white text-[#2962ff]"
                      }`}
                      title={`Switch Chart to ${sym}`}
                    >
                      {sym}
                    </button>
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Hero Image / Illustration */}
          {article.media_url ? (
            <div className="rounded-xl overflow-hidden border border-[#2a2e39]/60 bg-black/20 shadow-md">
              <img
                src={article.media_url}
                alt={article.title}
                className="w-full max-h-[360px] object-contain sm:object-cover mx-auto bg-black/40"
                onError={(e) => {
                  (e.target as HTMLElement).style.display = "none";
                }}
              />
              <div
                className={`px-3 py-1.5 text-[10px] flex items-center justify-between border-t ${
                  isLight
                    ? "bg-[#f8f9fc] border-[#e0e3eb] text-[#787b86]"
                    : "bg-[#141720] border-[#2a2e39] text-[#787b86]"
                }`}
              >
                <span>Illustration: {article.source || "Market Wire"}</span>
                <span className="font-mono">Live Media Feed</span>
              </div>
            </div>
          ) : (
            // Sleek contextual financial banner fallback
            <div
              className={`rounded-xl p-4 sm:p-5 border flex items-center justify-between relative overflow-hidden ${
                isLight
                  ? "bg-gradient-to-r from-[#f0f4ff] to-[#f8f9fc] border-[#dbe4ff]"
                  : "bg-gradient-to-r from-[#172033] via-[#1a202c] to-[#141824] border-[#2962ff]/30"
              }`}
            >
              <div className="relative z-10 space-y-1">
                <div className="flex items-center gap-2">
                  <Sparkles className="w-4 h-4 text-[#2962ff]" />
                  <span className="text-[11px] font-bold uppercase tracking-wider text-[#2962ff]">
                    Institutional Wire Dispatch
                  </span>
                </div>
                <div
                  className={`text-sm font-semibold ${
                    isLight ? "text-[#131722]" : "text-white"
                  }`}
                >
                  {article.source} Real-Time Terminal Feed
                </div>
              </div>
              <div className="text-3xl sm:text-4xl font-mono font-black opacity-15 select-none text-[#2962ff]">
                {tickers[0] || "MARKET"}
              </div>
            </div>
          )}

          {/* Key Takeaways / Bullets (As seen in FXStreet user screenshot) */}
          {bullets.length > 0 && (
            <div
              className={`rounded-xl p-4 sm:p-4.5 border ${
                isLight
                  ? "bg-[#f8f9fd] border-[#d8e0f0]"
                  : "bg-[#131722] border-[#2a2e39]"
              }`}
            >
              <div className="flex items-center gap-2 mb-3">
                <div className="w-2 h-2 rounded-full bg-[#2962ff] animate-pulse" />
                <span
                  className={`text-xs font-bold uppercase tracking-wider ${
                    isLight ? "text-[#131722]" : "text-white"
                  }`}
                >
                  Key Takeaways
                </span>
              </div>
              <ul className="space-y-2 text-xs sm:text-[13px] leading-relaxed">
                {bullets.map((b, idx) => (
                  <li key={idx} className="flex items-start gap-2.5">
                    <span className="w-1.5 h-1.5 rounded-full bg-[#2962ff] mt-1.5 shrink-0" />
                    <span className={isLight ? "text-[#2e323e]" : "text-[#c5c9d6]"}>
                      {b}
                    </span>
                  </li>
                ))}
              </ul>
            </div>
          )}

          {/* Full Paragraphs */}
          <div className="space-y-4 text-xs sm:text-[13px] leading-relaxed">
            {paragraphs.length > 0 ? (
              paragraphs.map((p, idx) => (
                <p
                  key={idx}
                  className={
                    isLight
                      ? "text-[#2e323e] leading-relaxed"
                      : "text-[#c5c9d6] leading-relaxed"
                  }
                >
                  {p}
                </p>
              ))
            ) : cleanSummary ? (
              <p
                className={
                  isLight
                    ? "text-[#2e323e] leading-relaxed"
                    : "text-[#c5c9d6] leading-relaxed"
                }
              >
                {cleanSummary}
              </p>
            ) : (
              <p className="text-[#787b86] italic">
                Rangkuman lengkap tersedia di portal resmi sumber berita.
              </p>
            )}
          </div>

          {/* Bottom Call to Action */}
          <div
            className={`pt-4 border-t flex flex-col sm:flex-row items-center justify-between gap-3 ${
              isLight ? "border-[#e0e3eb]" : "border-[#2a2e39]"
            }`}
          >
            <div className="text-[11px] text-[#787b86]">
              Sumber resmi:{" "}
              <span className="font-semibold">{article.source}</span>
            </div>
            <a
              href={article.url}
              target="_blank"
              rel="noreferrer"
              className="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg bg-[#2962ff] hover:bg-[#1e53e5] text-white font-bold text-xs shadow-md hover:shadow-lg transition-all cursor-pointer"
            >
              <span>Open full article at {article.source}</span>
              <ArrowRight className="w-3.5 h-3.5" />
            </a>
          </div>
        </div>
      </div>
    </div>
  );
};
