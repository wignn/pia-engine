import logging
from lingua import Language, LanguageDetectorBuilder

logger = logging.getLogger("finbert-service.language")

_detector = (
    LanguageDetectorBuilder
    .from_languages(Language.INDONESIAN, Language.ENGLISH)
    .with_low_accuracy_mode()
    .build()
)

_LANG_MAP = {
    Language.INDONESIAN: "id",
    Language.ENGLISH: "en",
}


def detect_language(text: str) -> str:
    if not text or not text.strip():
        return "unknown"

    result = _detector.detect_language_of(text)
    if result is None:
        return "unknown"

    return _LANG_MAP.get(result, "unknown")
