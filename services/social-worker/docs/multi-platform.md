# Multi-platform social worker

## Konfigurasi akun

Gunakan format `platform:username` dalam satu environment variable:

```env
SOCIAL_ACCOUNTS=twitter:akun_a,truth:realDonaldTrump
```

Platform yang tersedia saat ini:

- `twitter` — memakai `twscrape`; session dapat di-bootstrap dari `X_AUTH_TOKEN` + `X_CT0` saat runtime dan disimpan ke `TWITTER_DB`.

### Cookie X dari environment

Isi **keduanya** di `.env` lokal:

```env
X_AUTH_TOKEN=nilai_auth_token_asli
X_CT0=nilai_ct0_asli
X_ACCOUNT_NAME=worker
```

Nilai tersebut adalah cookie login sensitif dari cookie `x.com` (`auth_token` dan `ct0`). Worker hanya membaca secret dari environment lalu memanggil `twscrape.add_account_cookies()`; nilainya tidak dimasukkan ke image atau log. Jangan commit `.env`. Jika salah satu cookie tidak ada, konfigurasi ditolak.

`TWITTER_DB` tetap hanya state runtime. Compose me-mount `./data:/data` agar session dapat bertahan antar-restart, tetapi `data/accounts.db` tidak boleh di-commit atau dimasukkan ke Docker image.

Jika tidak menggunakan cookie environment, worker masih dapat memakai session `twscrape` yang sudah ada di `TWITTER_DB`.
- `truth` — adapter boundary tersedia, tetapi client/API resmi atau client yang diizinkan harus dikonfigurasi sebelum polling Truth Social aktif.

Username dinormalisasi lowercase dan `@` dihapus. Pasangan platform+username dideduplikasi; `twitter:alice` dan `truth:alice` tetap berbeda.

## Schema event NATS

Subject default: `social.posts`

```json
{
  "event_id": "twitter:123",
  "post_id": "123",
  "platform": "twitter",
  "source_account": "akun_a",
  "author_username": "akun_a",
  "author_display_name": "Account A",
  "text": "contoh postingan",
  "url": "https://x.com/akun_a/status/123",
  "created_at": "2026-01-01T00:00:00Z",
  "fetched_at": "2026-01-01T00:01:00Z",
  "reply_count": 0,
  "retweet_count": 0,
  "like_count": 0,
  "quote_count": 0,
  "language": "id",
  "media_urls": []
}
```

Field yang tidak tersedia pada suatu platform dikirim kosong atau nol; worker tidak mengarang nilai.

## Docker

```powershell
New-Item -ItemType Directory -Force data
# taruh session Twitter di data/accounts.db
docker compose up --build -d
docker compose logs -f worker
```

Compose memberi worker URL internal `nats://nats:4222`. Untuk Truth Social, credential/client harus diberikan melalui runtime secret; jangan memasukkannya ke Dockerfile atau image.

## Menambah adapter platform

1. Tambahkan platform ke `SUPPORTED_PLATFORMS` pada `app/config.py`.
2. Implementasikan `fetch_latest` dan `close` sesuai `app/sources/base.py`.
3. Kembalikan `TweetRecord` dengan `platform` yang tepat.
4. Daftarkan adapter pada `PollingWorker`.
5. Tambahkan fake adapter test sebelum network smoke test.

Adapter harus mematuhi rate limit dan ketentuan layanan platform. Jangan menganggap endpoint atau cookie satu platform dapat dipakai untuk platform lain.

## Truth Social

`truth:realDonaldTrump` sudah valid sebagai konfigurasi target, tetapi source saat ini menghasilkan error terkontrol sampai adapter Truth Social diisi dengan integrasi yang sah. Ini mencegah worker diam-diam mengirim data Twitter dengan label Truth Social.
