import json
import os

COOKIES_FILE = 'cookies.json'

print("=" * 50)
print("  PENGATURAN COOKIE TWITTER / X (BYPASS CLOUDFLARE)")
print("=" * 50)
print("\nCara ambil cookie dari Chrome / Edge:")
print("1. Buka https://x.com di browser (pastikan sudah login).")
print("2. Tekan F12 di keyboard -> Pilih tab 'Application' (atau 'Storage').")
print("3. Di menu sebelah kiri: Cookies -> https://x.com")
print("4. Salin nilai (Value) dari dua cookie berikut:")
print("   - auth_token")
print("   - ct0\n")

auth_token = input("Masukkan auth_token: ").strip().strip('"').strip("'")
ct0 = input("Masukkan ct0: ").strip().strip('"').strip("'")

if not auth_token or not ct0:
    print("\n[ERROR] auth_token dan ct0 tidak boleh kosong!")
    exit(1)

cookies = {
    "auth_token": auth_token,
    "ct0": ct0
}

with open(COOKIES_FILE, "w", encoding="utf-8") as f:
    json.dump(cookies, f, indent=4)

print(f"\n[SUKSES] File '{COOKIES_FILE}' berhasil dibuat!")
print("Sekarang jalankan: py main.py\n")
