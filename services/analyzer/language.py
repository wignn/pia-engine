import re

ID_WORDS = {
    "yang", "dan", "atau", "dengan", "menjadi", "naik", "turun", "minggu",
    "bulan", "tahun", "perkiraan", "sebelumnya", "klaim", "pengangguran",
    "suku", "bunga", "rupiah", "dolar", "emas", "berita", "pasar",
}

EN_WORDS = {
    "the", "and", "or", "with", "rose", "fell", "increased", "decreased",
    "forecast", "previous", "jobless", "claims", "inflation", "rate", "market",
    "dollar", "gold", "stocks", "economy", "economic",
}


def detect_language(text: str) -> str:
    """
    Lightweight language detector without new heavy dependencies.
    Returns: id, en, or unknown.
    """
    if not text or not text.strip():
        return "unknown"

    words = re.findall(r"[A-Za-zÀ-ÿ]+", text.lower())
    if not words:
        return "unknown"

    sample = words[:300]
    id_score = sum(1 for w in sample if w in ID_WORDS)
    en_score = sum(1 for w in sample if w in EN_WORDS)

    id_score += len(re.findall(r"\b(mem|men|meng|ber|ter|ke|se)[a-z]+", " ".join(sample))) * 0.15

    if id_score >= en_score + 2:
        return "id"
    if en_score >= id_score + 2:
        return "en"
    return "unknown"
