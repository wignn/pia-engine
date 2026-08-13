# Arsitektur social worker

```text
Twitter/twscrape ─┐
                  ├─> PollingWorker ─> TweetRecord ─> NATS subject ─> consumers
Truth Social ─────┘
```

`PollingWorker` mengatur jadwal, cursor, deduplikasi, dan status. Adapter di `app/sources/` menangani perbedaan API setiap platform. Publisher NATS tidak perlu tahu cara post diambil; semua source menghasilkan schema `TweetRecord` yang sama.

Event dibedakan oleh `event_id` berbentuk `platform:post_id`, sehingga post ID yang sama pada Twitter dan Truth Social tetap menjadi dua event berbeda.

Truth Social adapter sengaja gagal jelas sampai client/API yang sah dikonfigurasi. Jangan memakai cookie Twitter atau endpoint privat Truth Social sebagai fallback.
