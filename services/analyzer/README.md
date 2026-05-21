# Tuned Financial Sentiment Service

Compatible replacement for the old `/analyze` API.

This version implements **Option 1: translate non-English text to English before sending it to FinBERT / the configured English financial model**.

## Run

```bash
pip install -r requirements.txt
python main.py
```

## Optional env

```bash
export PORT=5000
export WORKERS=1
export SENTIMENT_MODEL=yiyanghkust/finbert-tone
export MAX_TOKENS=512
export NEUTRAL_THRESHOLD=0.55
export MARGIN_THRESHOLD=0.10

# Translation-first mode
export ENABLE_TRANSLATION=true
export RETURN_TRANSLATED_TEXT=true
export TRANSLATION_MAX_CHARS=4500
export TRANSLATION_CHUNK_CHARS=1400
```

For development/testing, translation uses `deep-translator` / `GoogleTranslator`.
For production/high volume, replace `translator.translate_to_english()` with a stable paid translation API.

## Analyze text

```bash
curl -X POST http://localhost:5000/analyze \
  -H 'Content-Type: application/json' \
  -d '{"text":"AS: Klaim Tunjangan Pengangguran Awal naik menjadi 209 ribu dari 212 ribu minggu sebelumnya."}'
```

## Analyze URL

```bash
curl -X POST http://localhost:5000/analyze \
  -H 'Content-Type: application/json' \
  -d '{"url":"https://example.com/news"}'
```

## Old request is unchanged

```json
{"text":"..."}
```

or:

```json
{"url":"https://..."}
```

## New optional response fields

```json
{
  "language": "id",
  "analysis_language": "en",
  "translated": true,
  "translated_text": "US Initial Jobless Claims rose to 209 thousand...",
  "translation_error": null
}
```

The old fields are still returned:

```json
{
  "sentiment": "neutral",
  "score": 0.72,
  "distribution": {},
  "highlights": [],
  "entities": {}
}
```
