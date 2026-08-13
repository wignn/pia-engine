# NATS worker

## Jalur data

```text
X/twscrape -> PollingWorker -> NATS subject -> subscriber
```

Worker mengambil postingan terbaru dari username di `TWITTER_ACCOUNTS`, lalu mempublish hanya tweet baru ke subject `NATS_SUBJECT`.

## Jalankan lokal

```powershell
pip install -r requirements.txt
$env:TWITTER_ACCOUNTS = "akun_a,akun_b"
$env:TWITTER_DB = "accounts.db"
$env:NATS_URL = "nats://127.0.0.1:4222"
$env:NATS_SUBJECT = "twitter.tweets"
python run_worker.py
```

Jalankan NATS:

```powershell
docker run --rm -p 4222:4222 nats:2.10-alpine
```

## Jalankan dengan Compose

```powershell
New-Item -ItemType Directory -Force data
# Letakkan session twscrape di data/accounts.db
docker compose up --build -d
docker compose logs -f worker
```

Compose memakai service name `nats`, jadi URL internal worker adalah `nats://nats:4222`.

## Payload

```json
{
  "tweet_id": "123",
  "source_account": "akun_a",
  "author_username": "akun_a",
  "text": "contoh postingan",
  "created_at": "2026-01-01T00:00:00Z",
  "fetched_at": "2026-01-01T00:01:00Z",
  "url": "https://x.com/akun_a/status/123",
  "reply_count": 0,
  "retweet_count": 0,
  "like_count": 0,
  "quote_count": 0,
  "language": "id",
  "media_urls": []
}
```

## Subscriber Python

```python
import asyncio
import json
import nats

async def main():
    nc = await nats.connect("nats://127.0.0.1:4222")

    async def handle(message):
        tweet = json.loads(message.data)
        print(tweet["source_account"], tweet["text"])

    await nc.subscribe("twitter.tweets", cb=handle)
    try:
        await asyncio.Event().wait()
    finally:
        await nc.drain()

asyncio.run(main())
```

Install subscriber dependency:

```powershell
pip install nats-py
```

## Environment

| Variable | Default | Fungsi |
|---|---|---|
| `TWITTER_ACCOUNTS` | wajib | Username dipisahkan koma |
| `TWITTER_POLL_SECONDS` | `60` | Interval polling |
| `TWITTER_TWEET_LIMIT` | `20` | Limit per akun per siklus |
| `TWITTER_DB` | `accounts.db` | Database session twscrape |
| `NATS_URL` | `nats://127.0.0.1:4222` | Alamat NATS |
| `NATS_SUBJECT` | `twitter.tweets` | Subject publish |
| `NATS_CREDS` | kosong | Credential file opsional |

## Reliability

Core NATS bersifat at-most-once untuk subscriber aktif. Publisher tidak menyimpan event yang terlewat ketika subscriber mati. Compose mengaktifkan JetStream dengan `-js`, tetapi publisher saat ini belum memakai stream/ack JetStream. Tambahkan JetStream hanya jika replay, durable consumer, atau acknowledgement diperlukan.

Jika koneksi NATS gagal, container akan gagal dan `restart: unless-stopped` pada Compose akan mencoba menjalankannya kembali.
