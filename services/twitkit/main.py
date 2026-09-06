import asyncio
import os
import json
from twikit import Client

COOKIES_FILE = 'cookies.json'


async def main():
    client = Client('en-US')

    if not os.path.exists(COOKIES_FILE):
        print(f"[!] File {COOKIES_FILE} belum ditemukan.")
        print("[!] Twitter memblokir login username/password langsung via Cloudflare (403).")
        print("[!] Silakan buat cookies terlebih dahulu dengan menjalankan:")
        print("    py create_cookies.py\n")
        return

    try:
        client.load_cookies(COOKIES_FILE)
        print("[+] Berhasil memuat session cookies!")

        # Test koneksi & ambil informasi profil akun
        user = await client.user()
        print(f"[+] Login Sukses sebagai: @{user.screen_name} ({user.name})")
        print(f"[+] Jumlah Followers: {user.followers_count}")
    except Exception as e:
        print(f"[-] Terjadi kesalahan saat menggunakan cookie: {e}")
        print("[-] Pastikan nilai auth_token dan ct0 di cookies.json masih valid dan belum expired.")


if __name__ == '__main__':
    asyncio.run(main())
