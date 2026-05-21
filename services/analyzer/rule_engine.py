from typing import Any, Dict, Optional


def _neutral(reason: str) -> Dict[str, Any]:
    return {
        "market_impact": {
            "USD": "neutral",
            "GOLD": "neutral",
            "SP500": "neutral",
            "BTC": "neutral",
        },
        "final_signal": "neutral",
        "reason": reason,
        "confidence": 0.35,
    }


def _sentiment_bias(sentiment_result: Dict[str, Any]) -> str:
    sentiment = (sentiment_result.get("sentiment") or "neutral").lower()
    if sentiment in {"positive", "negative", "mixed"}:
        return sentiment
    return "neutral"


def interpret_market(event: Optional[Dict[str, Any]], sentiment_result: Dict[str, Any]) -> Dict[str, Any]:
    """
    Converts text sentiment + extracted macro facts into asset impact.
    This is not financial advice; it is a deterministic news interpretation layer.
    """
    if not event:
        sentiment = _sentiment_bias(sentiment_result)
        if sentiment == "positive":
            return {
                "market_impact": {"USD": "neutral", "GOLD": "neutral", "SP500": "mild_bullish", "BTC": "mild_bullish"},
                "final_signal": "text_positive_no_macro_event",
                "reason": "Tidak ada event makro terstruktur yang terdeteksi; sinyal hanya dari sentimen teks.",
                "confidence": 0.45,
            }
        if sentiment == "negative":
            return {
                "market_impact": {"USD": "neutral", "GOLD": "neutral", "SP500": "mild_bearish", "BTC": "mild_bearish"},
                "final_signal": "text_negative_no_macro_event",
                "reason": "Tidak ada event makro terstruktur yang terdeteksi; sinyal hanya dari sentimen teks.",
                "confidence": 0.45,
            }
        return _neutral("Tidak ada event makro terstruktur yang terdeteksi dan sentimen teks tidak kuat.")

    etype = event.get("type")
    actual = event.get("actual")
    forecast = event.get("forecast")
    previous = event.get("previous")

    if etype in {"US_INITIAL_JOBLESS_CLAIMS", "US_CONTINUING_JOBLESS_CLAIMS"}:
        # For jobless claims, lower is generally better for USD because labor market looks stronger.
        if forecast is not None:
            if actual is not None and actual < forecast:
                return {
                    "market_impact": {"USD": "bullish", "GOLD": "bearish", "SP500": "mixed", "BTC": "mixed"},
                    "final_signal": "usd_bullish_labor_better_than_forecast",
                    "reason": "Klaim pengangguran lebih rendah dari forecast; pasar tenaga kerja terlihat lebih kuat dari ekspektasi.",
                    "confidence": 0.75,
                }
            if actual is not None and actual > forecast:
                return {
                    "market_impact": {"USD": "bearish", "GOLD": "bullish", "SP500": "mixed", "BTC": "mixed"},
                    "final_signal": "usd_bearish_labor_worse_than_forecast",
                    "reason": "Klaim pengangguran lebih tinggi dari forecast; pasar tenaga kerja terlihat lebih lemah dari ekspektasi.",
                    "confidence": 0.75,
                }

        if previous is not None:
            if actual is not None and actual < previous:
                return {
                    "market_impact": {"USD": "mild_bullish", "GOLD": "mild_bearish", "SP500": "mixed", "BTC": "mixed"},
                    "final_signal": "usd_mild_bullish_labor_improving_vs_previous",
                    "reason": "Klaim pengangguran lebih rendah dari periode sebelumnya; sinyal ringan positif untuk USD.",
                    "confidence": 0.62,
                }
            if actual is not None and actual > previous:
                return {
                    "market_impact": {"USD": "mild_bearish", "GOLD": "mild_bullish", "SP500": "mixed", "BTC": "mixed"},
                    "final_signal": "usd_mild_bearish_labor_softening_vs_previous",
                    "reason": "Klaim pengangguran naik dari periode sebelumnya; sinyal ringan negatif untuk USD.",
                    "confidence": 0.58,
                }
        return _neutral("Event klaim pengangguran terdeteksi, tetapi actual/forecast/previous tidak cukup lengkap.")

    if etype == "CPI":
        if actual is not None and forecast is not None:
            if actual > forecast:
                return {
                    "market_impact": {"USD": "bullish", "GOLD": "mixed", "SP500": "bearish", "BTC": "bearish"},
                    "final_signal": "hotter_cpi_hawkish",
                    "reason": "CPI lebih tinggi dari forecast; pasar cenderung membaca ini sebagai tekanan inflasi dan risiko kebijakan moneter lebih hawkish.",
                    "confidence": 0.78,
                }
            if actual < forecast:
                return {
                    "market_impact": {"USD": "bearish", "GOLD": "bullish", "SP500": "bullish", "BTC": "bullish"},
                    "final_signal": "cooler_cpi_dovish",
                    "reason": "CPI lebih rendah dari forecast; tekanan inflasi mereda dan pasar cenderung membaca ini lebih dovish.",
                    "confidence": 0.78,
                }
        return _neutral("Event CPI terdeteksi, tetapi actual dan forecast tidak lengkap.")

    if etype == "PMI":
        if actual is not None and forecast is not None:
            if actual > forecast:
                return {
                    "market_impact": {"USD": "mild_bullish", "GOLD": "mild_bearish", "SP500": "mild_bullish", "BTC": "mixed"},
                    "final_signal": "pmi_better_than_forecast",
                    "reason": "PMI lebih tinggi dari forecast; aktivitas bisnis terlihat lebih kuat dari ekspektasi.",
                    "confidence": 0.65,
                }
            if actual < forecast:
                return {
                    "market_impact": {"USD": "mild_bearish", "GOLD": "mild_bullish", "SP500": "mild_bearish", "BTC": "mixed"},
                    "final_signal": "pmi_worse_than_forecast",
                    "reason": "PMI lebih rendah dari forecast; aktivitas bisnis terlihat lebih lemah dari ekspektasi.",
                    "confidence": 0.65,
                }
        return _neutral("Event PMI terdeteksi, tetapi actual dan forecast tidak lengkap.")

    if etype == "US_NFP":
        if actual is not None and forecast is not None:
            if actual > forecast:
                return {
                    "market_impact": {"USD": "bullish", "GOLD": "bearish", "SP500": "mixed", "BTC": "mixed"},
                    "final_signal": "nfp_better_than_forecast",
                    "reason": "NFP lebih tinggi dari forecast; pasar tenaga kerja terlihat kuat sehingga USD cenderung didukung.",
                    "confidence": 0.75,
                }
            if actual < forecast:
                return {
                    "market_impact": {"USD": "bearish", "GOLD": "bullish", "SP500": "mixed", "BTC": "mixed"},
                    "final_signal": "nfp_worse_than_forecast",
                    "reason": "NFP lebih rendah dari forecast; pasar tenaga kerja terlihat melemah sehingga USD cenderung tertekan.",
                    "confidence": 0.75,
                }
        return _neutral("Event NFP terdeteksi, tetapi actual dan forecast tidak lengkap.")

    return _neutral(f"Event {etype} terdeteksi, tetapi rule khusus belum tersedia atau data angkanya belum lengkap.")
