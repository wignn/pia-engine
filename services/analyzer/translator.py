import os
import re
import logging
from typing import Tuple

logger = logging.getLogger("finbert-service.translator")

ENABLE_TRANSLATION = os.getenv("ENABLE_TRANSLATION", "true").lower() == "true"
TRANSLATION_TARGET = os.getenv("TRANSLATION_TARGET", "en")
TRANSLATION_MAX_CHARS = int(os.getenv("TRANSLATION_MAX_CHARS", "4500"))
TRANSLATION_CHUNK_CHARS = int(os.getenv("TRANSLATION_CHUNK_CHARS", "1400"))
RETURN_TRANSLATED_TEXT = os.getenv("RETURN_TRANSLATED_TEXT", "true").lower() == "true"
RETURN_TRANSLATED_TEXT_MAX_CHARS = int(os.getenv("RETURN_TRANSLATED_TEXT_MAX_CHARS", "2500"))

IMPORTANT_KEYWORDS = [
    # English
    "cpi", "inflation", "core", "nfp", "nonfarm", "payroll", "jobless",
    "claims", "unemployment", "pmi", "gdp", "fed", "fomc", "rate",
    "interest", "earnings", "revenue", "eps", "guidance", "dxy", "gold",
    "dollar", "forecast", "expected", "previous", "actual", "yield",
    # Indonesian
    "inflasi", "inti", "klaim", "pengangguran", "tenaga kerja", "pmi",
    "imp", "pdb", "suku bunga", "bank indonesia", "bi", "the fed",
    "perkiraan", "diprakirakan", "ekspektasi", "sebelumnya", "aktual",
    "imbal hasil", "dolar", "emas", "pendapatan", "laba",
]


def _normalize_space(text: str) -> str:
    return re.sub(r"\s+", " ", text or "").strip()


def build_translation_payload(title: str | None, content: str) -> str:
    """
    Build a compact text for translation.
    Do not translate the full scraped page because it can include footers, disclaimers,
    related news, and broker promos. The model only needs title + lead + macro facts.
    """
    parts: list[str] = []

    if title and title.strip():
        parts.append(_normalize_space(title))

    raw = content or ""
    paragraphs = [p.strip() for p in re.split(r"\n+", raw) if len(p.strip()) > 25]

    # Lead paragraphs usually contain the actual news.
    parts.extend(_normalize_space(p) for p in paragraphs[:6])

    # Add paragraphs/sentences containing numbers or important economic keywords.
    seen = set(parts)
    for p in paragraphs:
        lower = p.lower()
        has_number = bool(re.search(r"\d", p))
        has_keyword = any(k in lower for k in IMPORTANT_KEYWORDS)
        if not (has_number or has_keyword):
            continue

        compact = _normalize_space(p)
        if compact and compact not in seen:
            parts.append(compact)
            seen.add(compact)

        if sum(len(x) for x in parts) >= TRANSLATION_MAX_CHARS:
            break

    payload = "\n".join(parts)
    return payload[:TRANSLATION_MAX_CHARS]


def _split_chunks(text: str, max_chars: int) -> list[str]:
    text = text.strip()
    if len(text) <= max_chars:
        return [text]

    # Prefer paragraph boundaries, then sentence boundaries.
    units = re.split(r"(?<=\.)\s+|\n+", text)
    chunks: list[str] = []
    current = ""

    for unit in units:
        unit = unit.strip()
        if not unit:
            continue

        if len(unit) > max_chars:
            # Hard split very long units.
            for i in range(0, len(unit), max_chars):
                if current:
                    chunks.append(current.strip())
                    current = ""
                chunks.append(unit[i:i + max_chars].strip())
            continue

        candidate = f"{current}\n{unit}".strip() if current else unit
        if len(candidate) <= max_chars:
            current = candidate
        else:
            if current:
                chunks.append(current.strip())
            current = unit

    if current:
        chunks.append(current.strip())

    return chunks


def translate_to_english(text: str, source_lang: str = "auto") -> Tuple[str, bool, str | None]:
    if not ENABLE_TRANSLATION:
        return text, False, None

    if not text or not text.strip():
        return text, False, None

    if source_lang == "en":
        return text, False, None

    try:
        from deep_translator import GoogleTranslator

        translator = GoogleTranslator(source="auto", target=TRANSLATION_TARGET)
        chunks = _split_chunks(text[:TRANSLATION_MAX_CHARS], TRANSLATION_CHUNK_CHARS)
        translated_chunks: list[str] = []

        for chunk in chunks:
            translated = translator.translate(chunk)
            if translated and translated.strip():
                translated_chunks.append(translated.strip())

        translated_text = "\n".join(translated_chunks).strip()
        if not translated_text:
            return text, False, "translator_returned_empty_text"

        return translated_text, True, None

    except Exception as e:
        logger.warning("Translation failed; using original text. error=%s", e)
        return text, False, str(e)


def maybe_trim_translated_text(text: str | None) -> str | None:
    if not RETURN_TRANSLATED_TEXT or not text:
        return None
    return text[:RETURN_TRANSLATED_TEXT_MAX_CHARS]
