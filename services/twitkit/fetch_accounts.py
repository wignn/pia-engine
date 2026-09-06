import asyncio
import os
from twikit import Client

COOKIES_FILE = 'cookies.json'

SOCIAL_ACCOUNTS = [
    'macropaperr',
    'BullTheoryio',
    'WhaleInsider',
    'coinbureau'
]


async def fetch_account_tweets(client: Client, screen_name: str, tweet_count: int = 5):
    print(f"\n{'=' * 60}")
    print(f" Mengambil postingan dari: @{screen_name}")
    print(f"{'=' * 60}")
    try:
        user = await client.get_user_by_screen_name(screen_name)
        print(f"Nama: {user.name} | Followers: {user.followers_count:,}")

        tweets = await user.get_tweets('Tweets', count=tweet_count)
        if not tweets:
            print("  [!] Tidak ada postingan ditemukan.")
            return

        for idx, tweet in enumerate(tweets, 1):
            print(f"\n--- [{idx}] ID: {tweet.id} ---")
            print(f"Waktu: {tweet.created_at}")
            print(f"Teks : {tweet.text}")
            print(f"Likes: {getattr(tweet, 'favorite_count', 0)} | Retweets: {getattr(tweet, 'retweet_count', 0)}")

    except Exception as e:
        print(f"  [-] Gagal mengambil postingan @{screen_name}: {e}")


async def main():
    client = Client('en-US')

    if not os.path.exists(COOKIES_FILE):
        print(f"File {COOKIES_FILE} belum dibuat! Jalankan py create_cookies.py terlebih dahulu.")
        return

    client.load_cookies(COOKIES_FILE)
    print("Session cookies berhasil dimuat!")

    for account in SOCIAL_ACCOUNTS:
        await fetch_account_tweets(client, account, tweet_count=5)
        await asyncio.sleep(1.5)  # Delay halus agar tidak terkena rate-limit


if __name__ == '__main__':
    asyncio.run(main())
