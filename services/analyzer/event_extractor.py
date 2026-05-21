import re
from typing import Any, Dict, Optional


def _normalize_decimal(value: str, unit: str | None = None) -> float:
    value = value.strip().replace(" ", "")
    unit_lower = (unit or "").lower()

    if unit_lower in {"juta", "million", "m"} and "," in value and "." not in value:
        value = value.replace(",", ".")
    else:
        value = value.replace(",", "")

    try:
        num = float(value)
    except ValueError:
        return 0.0

    if unit_lower in {"ribu", "thousand", "k"}:
        return num * 1_000
    if unit_lower in {"juta", "million", "m"}:
        return num * 1_000_000
    if unit_lower in {"billion", "b"}:
        return num * 1_000_000_000
    return num


def _number_pattern() -> str:
    return r"([0-9]+(?:[\.,][0-9]+)?)\s*(ribu|juta|thousand|million|billion|k|m|b)?"


def _first_number_after(pattern: str, text: str) -> tuple[Optional[float], Optional[str], Optional[str]]:
    m = re.search(pattern + r"[^0-9]{0,80}" + _number_pattern(), text, re.IGNORECASE)
    if not m:
        return None, None, None
    raw_num = m.group(m.lastindex - 1)
    unit = m.group(m.lastindex)
    return _normalize_decimal(raw_num, unit), unit, raw_num


def _extract_actual(text: str) -> tuple[Optional[float], Optional[str], Optional[str]]:
    patterns = [
        r"(?:meningkat|naik|turun|turun sebesar|menjadi|mencapai|berada di|tercatat|came in at|rose to|increased to|fell to|declined to)",
        r"(?:actual|aktual)",
    ]
    for p in patterns:
        value, unit, raw = _first_number_after(p, text)
        if value is not None:
            return value, unit, raw
    return None, None, None


def _extract_previous(text: str) -> tuple[Optional[float], Optional[str], Optional[str]]:
    patterns = [
        r"(?:sebelumnya|minggu sebelumnya|bulan sebelumnya|previous|prior|from)",
        r"(?:direvisi dari|revised from)",
        r"(?:dari|from)",
    ]
    for p in patterns:
        value, unit, raw = _first_number_after(p, text)
        if value is not None:
            return value, unit, raw
    return None, None, None


def _extract_forecast(text: str) -> tuple[Optional[float], Optional[str], Optional[str]]:
    patterns = [
        r"(?:forecast|consensus|expected|expectation|perkiraan|diperkirakan|ekspektasi|konsensus)",
    ]
    for p in patterns:
        value, unit, raw = _first_number_after(p, text)
        if value is not None:
            return value, unit, raw
    return None, None, None


def extract_event(text: str, language: str = "unknown") -> Optional[Dict[str, Any]]:
    """
    Extracts simple macro event facts from multilingual finance news.
    This is intentionally regex-based so it stays light on a 4 CPU / 8 GB RAM VPS.
    """
    if not text or not text.strip():
        return None

    t = re.sub(r"\s+", " ", text.strip())
    tl = t.lower()

    event_type: Optional[str] = None
    country: Optional[str] = None

    if any(k in tl for k in [
        "initial jobless claims", "jobless claims", "klaim tunjangan pengangguran", "klaim pengangguran",
    ]):
        event_type = "US_INITIAL_JOBLESS_CLAIMS"
        country = "US"
    elif any(k in tl for k in ["continuing claims", "klaim tunjangan pengangguran lanjutan", "klaim lanjutan"]):
        event_type = "US_CONTINUING_JOBLESS_CLAIMS"
        country = "US"
    elif any(k in tl for k in ["consumer price index", "cpi", "indeks harga konsumen", "inflasi"]):
        event_type = "CPI"
    elif any(k in tl for k in ["nonfarm payroll", "non-farm payroll", "nfp", "payroll"]):
        event_type = "US_NFP"
        country = "US"
    elif any(k in tl for k in ["pmi", "purchasing managers index", "indeks manajer pembelian", "imp"]):
        event_type = "PMI"
    elif any(k in tl for k in ["fed", "fomc", "federal reserve", "suku bunga fed"]):
        event_type = "FED_RATE_OR_GUIDANCE"
        country = "US"
    elif any(k in tl for k in ["bank indonesia", "bi rate", "suku bunga bi"]):
        event_type = "BI_RATE"
        country = "ID"
    elif any(k in tl for k in ["gdp", "gross domestic product", "pdb", "produk domestik bruto"]):
        event_type = "GDP"

    if not event_type:
        return None

    actual, actual_unit, actual_raw = _extract_actual(t)
    previous, previous_unit, previous_raw = _extract_previous(t)
    forecast, forecast_unit, forecast_raw = _extract_forecast(t)

    unit = actual_unit or forecast_unit or previous_unit

    return {
        "type": event_type,
        "country": country,
        "actual": actual,
        "forecast": forecast,
        "previous": previous,
        "unit": unit,
        "raw": {
            "actual": actual_raw,
            "forecast": forecast_raw,
            "previous": previous_raw,
            "language": language,
        },
    }
