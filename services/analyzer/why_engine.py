import json
import os
import re
from datetime import datetime, timezone
from typing import Any

import httpx

from schemas import (
    WhyMoveCluster,
    WhyMoveConfidence,
    WhyMoveDriver,
    WhyMoveEngineStatus,
    WhyMoveNarrative,
    WhyMoveNewsItem,
    WhyMoveRequest,
    WhyMoveResponse,
)

ENGINE_VERSION = "why-engine-v1"

SYMBOL_GRAPH = {
    "XAUUSD": {
        "terms": ["xauusd", "xau", "gold", "emas", "usd", "fed", "inflation", "yield", "treasury", "safe haven", "geopolitical"],
        "drivers": ["USD weakness", "real yields", "Fed policy", "inflation hedge", "safe-haven demand"],
    },
    "DXY": {
        "terms": ["dxy", "dollar", "usd", "greenback", "fed", "treasury", "yield", "inflation"],
        "drivers": ["USD strength", "Fed policy", "Treasury yields", "inflation expectations"],
    },
    "SPX": {
        "terms": ["spx", "s&p 500", "s&p500", "us500", "stocks", "equities", "earnings", "fed", "risk"],
        "drivers": ["risk sentiment", "Fed policy", "earnings", "inflation expectations"],
    },
    "BTCUSDT": {
        "terms": ["btcusdt", "btc", "bitcoin", "crypto", "kripto", "liquidity", "etf", "risk"],
        "drivers": ["crypto risk appetite", "ETF flow", "USD liquidity", "macro risk"],
    },
    "ETHUSDT": {
        "terms": ["ethusdt", "eth", "ethereum", "crypto", "kripto", "liquidity", "etf", "risk"],
        "drivers": ["crypto risk appetite", "ETF flow", "USD liquidity", "macro risk"],
    },
}

THEME_TERMS = {
    "Fed/rates/yields": ["fed", "fomc", "rate", "rates", "yield", "treasury", "hawkish", "dovish"],
    "Inflation/jobs macro": ["inflation", "cpi", "ppi", "payroll", "nfp", "jobless", "claims", "labor"],
    "USD pressure": ["usd", "dollar", "greenback", "dxy"],
    "Risk-on/risk-off": ["risk", "stocks", "equities", "spx", "s&p", "safe haven", "volatility"],
    "Crypto flow": ["bitcoin", "btc", "ethereum", "eth", "crypto", "etf"],
    "Stock/index specific": ["earnings", "shares", "index", "nasdaq", "dow"],
}


def _norm_symbol(symbol: str) -> str:
    return re.sub(r"[^A-Z0-9]", "", (symbol or "").upper())


def _symbol_terms(symbol: str) -> list[str]:
    sym = _norm_symbol(symbol)
    graph = SYMBOL_GRAPH.get(sym)
    terms = [sym.lower()]
    if graph:
        terms.extend(graph["terms"])
    elif sym.endswith("USDT"):
        terms.extend([sym[:-4].lower(), "crypto", "kripto", "usdt"])
    elif len(sym) == 6:
        terms.extend([sym[:3].lower(), sym[3:].lower()])
    return sorted(set(t for t in terms if t))


def _driver_names(symbol: str) -> list[str]:
    sym = _norm_symbol(symbol)
    if sym in SYMBOL_GRAPH:
        return SYMBOL_GRAPH[sym]["drivers"]
    if sym.endswith("USDT"):
        return ["crypto risk appetite", "USD liquidity", "market momentum"]
    if len(sym) == 6:
        return [f"{sym[:3]}/{sym[3:]} currency flow", "central bank expectations", "USD liquidity"]
    return ["symbol-specific news", "market momentum", "risk sentiment"]


def _text(item: WhyMoveNewsItem) -> str:
    return " ".join([item.title or "", item.summary or "", " ".join(item.matched_terms or [])]).lower()


def _parse_time(value: str | None) -> datetime | None:
    if not value:
        return None
    try:
        return datetime.fromisoformat(value.replace("Z", "+00:00")).astimezone(timezone.utc)
    except ValueError:
        return None


def _timing_score(item: WhyMoveNewsItem, latest_at: datetime | None) -> float:
    if not latest_at:
        return 0.45
    event_at = _parse_time(item.processed_at) or _parse_time(item.published_at)
    if not event_at:
        return 0.35
    minutes = abs((latest_at - event_at).total_seconds()) / 60
    if minutes <= 15:
        return 1.0
    if minutes <= 60:
        return 0.8
    if minutes <= 180:
        return 0.55
    return 0.25


def _sentiment_score(item: WhyMoveNewsItem, direction: str) -> float:
    sentiment = (item.sentiment or "neutral").lower()
    if direction == "up" and sentiment in {"positive", "bullish"}:
        return 1.0
    if direction == "down" and sentiment in {"negative", "bearish"}:
        return 1.0
    if sentiment in {"mixed", "neutral"}:
        return 0.55
    return 0.25


def _impact_score(item: WhyMoveNewsItem) -> float:
    impact = (item.impact_level or "").lower()
    if impact == "high":
        return 1.0
    if impact == "medium":
        return 0.7
    if impact == "low":
        return 0.4
    return 0.5


def _lexical_score(item: WhyMoveNewsItem, terms: list[str]) -> tuple[float, list[str]]:
    haystack = _text(item)
    matches = sorted({term for term in terms if term and term in haystack})
    if not matches:
        return 0.0, []
    direct = any(len(m) >= 5 for m in matches)
    score = min(1.0, 0.35 + len(matches) * 0.15 + (0.2 if direct else 0.0))
    return score, matches


def _cross_asset_score(cross_assets: list[Any]) -> tuple[float, list[str]]:
    if not cross_assets:
        return 0.0, []
    evidence = []
    score = 0.0
    for asset in cross_assets[:6]:
        relationship = asset.relationship or "same-window move"
        evidence.append(f"{asset.symbol} {asset.move_pct:+.2f}%: {relationship}")
        score += min(abs(asset.move_pct) / 0.5, 1.0) * 0.12
    return min(score, 1.0), evidence


def rank_news(request: WhyMoveRequest) -> tuple[list[WhyMoveNewsItem], dict[str, float]]:
    terms = _symbol_terms(request.symbol)
    direction = request.move.direction if request.move else "none"
    latest_at = _parse_time(request.move.latest_at) if request.move else None
    ranked = []
    totals = {"timing": 0.0, "news_relevance": 0.0, "sentiment_alignment": 0.0, "impact": 0.0}
    news = request.causes.get("news", []) if request.causes else []

    for item in news:
        lexical, matches = _lexical_score(item, terms)
        if lexical <= 0:
            continue
        timing = _timing_score(item, latest_at)
        sentiment = _sentiment_score(item, direction)
        impact = _impact_score(item)
        score = lexical * 0.38 + timing * 0.22 + sentiment * 0.2 + impact * 0.2
        item.score = round(score * 100, 1)
        item.matched_terms = sorted(set([*(item.matched_terms or []), *[m.upper() for m in matches]]))
        item.reason = "Matched symbol drivers near the market move"
        ranked.append(item)
        totals["timing"] += timing
        totals["news_relevance"] += lexical
        totals["sentiment_alignment"] += sentiment
        totals["impact"] += impact

    ranked.sort(key=lambda i: i.score, reverse=True)
    count = max(len(ranked), 1)
    return ranked[:10], {k: round(v / count, 3) for k, v in totals.items()}


def cluster_news(items: list[WhyMoveNewsItem]) -> list[WhyMoveCluster]:
    clusters: list[WhyMoveCluster] = []
    for theme, terms in THEME_TERMS.items():
        hits = [item for item in items if any(term in _text(item) for term in terms)]
        if not hits:
            continue
        pos = sum(1 for h in hits if (h.sentiment or "").lower() in {"positive", "bullish"})
        neg = sum(1 for h in hits if (h.sentiment or "").lower() in {"negative", "bearish"})
        sentiment = "positive" if pos > neg else "negative" if neg > pos else "neutral"
        clusters.append(WhyMoveCluster(
            theme=theme,
            score=round(sum(h.score for h in hits) / len(hits), 1),
            sentiment=sentiment,
            headlines=[h.title for h in hits[:4]],
        ))
    clusters.sort(key=lambda c: c.score, reverse=True)
    return clusters[:5]


def build_drivers(request: WhyMoveRequest, ranked_news: list[WhyMoveNewsItem], cross_evidence: list[str]) -> list[WhyMoveDriver]:
    names = _driver_names(request.symbol)
    drivers = []
    for name in names[:5]:
        name_terms = [part.lower() for part in re.split(r"\W+", name) if len(part) > 2]
        evidence = [item.title for item in ranked_news if any(term in _text(item) for term in name_terms)][:3]
        if not evidence and cross_evidence:
            evidence = cross_evidence[:2]
        score = 0.45 + min(len(evidence) * 0.18, 0.45)
        drivers.append(WhyMoveDriver(name=name, score=round(score, 2), evidence=evidence))
    drivers.sort(key=lambda d: d.score, reverse=True)
    return drivers


def confidence_label(score: float) -> str:
    if score >= 0.74:
        return "high"
    if score >= 0.48:
        return "medium"
    return "low"


def deterministic_narrative(request: WhyMoveRequest, drivers: list[WhyMoveDriver], confidence: WhyMoveConfidence) -> WhyMoveNarrative:
    symbol = _norm_symbol(request.symbol)
    move_pct = request.move.move_pct if request.move and request.move.move_pct is not None else 0.0
    direction = request.move.direction if request.move else "none"
    top_driver = drivers[0].name if drivers else "market momentum"
    headline = f"{symbol} moved {direction} as {top_driver} dominated the evidence"
    explanation = (
        f"{symbol} moved {direction} {abs(move_pct):.2f}% over {request.window}. "
        f"The strongest available driver is {top_driver}, based on nearby news, sentiment alignment, and same-window cross-asset behavior."
    )
    caveats = []
    if confidence.label == "low":
        caveats.append("Evidence is thin or not tightly aligned with the move window.")
    if not request.calendar:
        caveats.append("No persisted economic calendar surprise was included in the evidence.")
    return WhyMoveNarrative(
        headline=headline,
        explanation=explanation,
        drivers=[d.name for d in drivers[:4]],
        confidence=confidence.label,
        caveats=caveats,
    )


def _gemini_enabled() -> bool:
    raw = os.getenv("WHY_LLM_ENABLED", "auto").lower()
    has_key = bool(os.getenv("GEMINI_API_KEY", "").strip())
    if raw in {"1", "true", "yes", "on"}:
        return has_key
    if raw in {"0", "false", "no", "off"}:
        return False
    return has_key


async def generate_gemini_narrative(payload: dict[str, Any]) -> WhyMoveEngineStatus:
    if not _gemini_enabled():
        return WhyMoveEngineStatus(provider="gemini", model=os.getenv("GEMINI_MODEL", "gemini-1.5-flash"), status="disabled")

    api_key = os.getenv("GEMINI_API_KEY", "").strip()
    model = os.getenv("GEMINI_MODEL", "gemini-1.5-flash")
    prompt = (
        "You are a market analyst. Use only the supplied JSON evidence. "
        "Return strict JSON with headline, explanation, drivers, confidence, caveats. "
        f"Evidence: {json.dumps(payload, ensure_ascii=False)}"
    )
    body = {
        "contents": [{"parts": [{"text": prompt}]}],
        "generationConfig": {"temperature": 0.2, "maxOutputTokens": 512, "responseMimeType": "application/json"},
    }
    url = f"https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}"
    try:
        async with httpx.AsyncClient(timeout=12.0) as client:
            response = await client.post(url, json=body)
        response.raise_for_status()
        data = response.json()
        text = data["candidates"][0]["content"]["parts"][0]["text"]
        narrative = WhyMoveNarrative.model_validate_json(text.strip().removeprefix("```json").removesuffix("```").strip())
        return WhyMoveEngineStatus(provider="gemini", model=model, status="generated", narrative=narrative)
    except Exception:
        return WhyMoveEngineStatus(provider="gemini", model=model, status="failed")


async def explain_why(request: WhyMoveRequest) -> WhyMoveResponse:
    ranked_news, breakdown = rank_news(request)
    cross_score, cross_evidence = _cross_asset_score(request.cross_assets)
    clusters = cluster_news(ranked_news)
    drivers = build_drivers(request, ranked_news, cross_evidence)

    breakdown["cross_asset_confirmation"] = round(cross_score, 3)
    combined = (
        breakdown.get("timing", 0) * 0.2
        + breakdown.get("news_relevance", 0) * 0.28
        + breakdown.get("sentiment_alignment", 0) * 0.18
        + breakdown.get("impact", 0) * 0.14
        + cross_score * 0.2
    )
    confidence = WhyMoveConfidence(label=confidence_label(combined), score=round(combined, 3), breakdown=breakdown)
    fallback = deterministic_narrative(request, drivers, confidence)

    llm_payload = {
        "symbol": request.symbol,
        "window": request.window,
        "move": request.move.model_dump() if request.move else None,
        "drivers": [d.model_dump() for d in drivers],
        "clusters": [c.model_dump() for c in clusters],
        "cross_assets": [c.model_dump() for c in request.cross_assets[:6]],
        "confidence": confidence.model_dump(),
        "fallback": fallback.model_dump(),
    }
    llm = await generate_gemini_narrative(llm_payload)
    narrative = llm.narrative or fallback

    return WhyMoveResponse(
        symbol=_norm_symbol(request.symbol),
        window=request.window,
        move=request.move,
        headline=narrative.headline,
        explanation=narrative.explanation,
        drivers=drivers,
        news_clusters=clusters,
        confidence=confidence,
        caveats=narrative.caveats,
        causes={"news": ranked_news, "calendar": request.causes.get("calendar", []) if request.causes else []},
        cross_assets=request.cross_assets,
        llm=llm,
    )
