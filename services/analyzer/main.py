import os
import logging
from contextlib import asynccontextmanager
from fastapi import FastAPI, HTTPException

from analyzer import AdvancedSentimentAnalyzer
from event_extractor import extract_event
from language import detect_language
from rule_engine import interpret_market
from schemas import AnalysisRequest, AnalysisResponse
from translator import build_translation_payload, maybe_trim_translated_text, translate_to_english


logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s - %(message)s",
)
logger = logging.getLogger("finbert-service")

analyzer = AdvancedSentimentAnalyzer()


@asynccontextmanager
async def lifespan(app: FastAPI):
    logger.info("Warming up sentiment model...")
    analyzer.initialize()
    logger.info("Service initialized and ready to process requests.")
    yield
    logger.info("Service shutting down.")


app = FastAPI(
    title="Advanced Financial Sentiment + Market Impact API",
    description="Financial news sentiment API with macro event extraction and asset impact mapping.",
    version="3.0.0",
    lifespan=lifespan,
)


@app.get("/health")
def health():
    return {
        "status": "ready" if analyzer.pipeline is not None else "initializing",
        "model": analyzer.model_name,
    }


@app.post("/analyze", response_model=AnalysisResponse)
def analyze(request: AnalysisRequest):
    if not request.text and not request.url:
        raise HTTPException(status_code=422, detail="Either 'text' or 'url' must be provided.")

    title = None
    content = None

    if request.url:
        url_str = request.url.strip()
        if not url_str:
            raise HTTPException(status_code=422, detail="'url' field must not be empty.")

        try:
            logger.info("Fetching and scraping URL: %s", url_str)
            from scraping import fetch, extract
            html_content = fetch(url_str)
            title, content = extract(html_content, url_str)
        except Exception as e:
            logger.error("Failed to scrape URL %s: %s", url_str, e, exc_info=True)
            raise HTTPException(status_code=400, detail=f"Failed to scrape URL: {str(e)}")

        if not content or not content.strip():
            raise HTTPException(status_code=400, detail="Scraped content is empty.")

        text_to_analyze = content
    else:
        text_to_analyze = request.text
        if not text_to_analyze or not text_to_analyze.strip():
            raise HTTPException(status_code=422, detail="'text' field must not be empty.")

    full_text = f"{title or ''}\n\n{text_to_analyze}".strip()
    source_language = detect_language(full_text)

    # Option 1: translate non-English news to English before sending it to FinBERT.
    # The /analyze contract stays the same for existing clients.
    translation_payload = build_translation_payload(title, text_to_analyze)
    model_text, translated, translation_error = translate_to_english(
        translation_payload,
        source_lang=source_language,
    )
    analysis_language = "en" if translated else source_language

    sentiment_result = analyzer.analyze(
        text=model_text,
        title=title if not translated else None,
        language=analysis_language,
    )

    # Extract macro events from original + translated text.
    # Original catches Indonesian terms; translated catches English macro terms.
    event_text = f"{full_text}\n\n{model_text}" if translated else full_text
    event = extract_event(event_text, language=source_language)
    market = interpret_market(event, sentiment_result)

    result = {
        **sentiment_result,
        "title": title,
        "content": content,
        "language": source_language,
        "analysis_language": analysis_language,
        "translated": translated,
        "translated_text": maybe_trim_translated_text(model_text) if translated else None,
        "translation_error": translation_error,
        "event": event,
        "market_impact": market.get("market_impact"),
        "final_signal": market.get("final_signal"),
        "reason": market.get("reason"),
        "confidence": market.get("confidence"),
    }
    return result


if __name__ == "__main__":
    import uvicorn

    port = int(os.getenv("PORT", 5000))
    workers = int(os.getenv("WORKERS", 1))  # keep 1; model is in-process
    logger.info("Starting server on port %s with %s worker(s)...", port, workers)
    uvicorn.run(
        "main:app",
        host="0.0.0.0",
        port=port,
        reload=False,
        workers=workers,
    )
