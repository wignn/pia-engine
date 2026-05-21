import sys
import re
import httpx
from urllib.parse import urlparse
from bs4 import BeautifulSoup


HEADERS = {
    "User-Agent": (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
        "AppleWebKit/537.36 (KHTML, like Gecko) "
        "Chrome/124.0.0.0 Safari/537.36"
    ),
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
    "Accept-Language": "en-US,en;q=0.9,id;q=0.8",
}

NOISE_TAGS = [
    "script", "style", "nav", "header", "footer", "aside",
    "iframe", "noscript", "form", "button", "svg", "img",
    "figure", "figcaption", "picture", "video", "audio",
    "ins", "ads", "advertisement",
]

DOMAIN_RULES: dict[str, list[str]] = {
    "fxstreet-id.com": [
        "div.fxs_article_content",
        "div[class*='fxs_article']",
        "article",
    ],
    "fxstreet.com": [
        "div.fxs_article_content",
        "article",
    ],
    "investing.com": [
        "div.articlePage",
        "div[data-test='article-content']",
        "div[class*='articleText']",
        "article",
    ],
    "investinglive.com": [
        "div.entry-content",
        "div.post-content",
        "article",
    ],
    "thomsonreuters.com": [
        "div.content__body",
        "div[class*='body']",
        "article",
    ],
    # Federal Reserve
    "federalreserve.gov": [
        "div#article",
        "div.col-xs-12.col-sm-8",
        "div[class*='article']",
        "section",
    ],
    # ECB
    "ecb.europa.eu": [
        "div.ecb-pressContent",
        "section.ecb-pressContent",
        "div[class*='press']",
        "article",
    ],
    "_generic": [
        "article",
        "div[class*='article-body']",
        "div[class*='article-content']",
        "div[class*='articleBody']",
        "div[class*='post-content']",
        "div[class*='entry-content']",
        "div[class*='story-body']",
        "div[class*='content-body']",
        "div[class*='newsBody']",
        "div[class*='news-content']",
        "main",
    ],
}


def get_domain(url: str) -> str:
    try:
        host = urlparse(url).hostname or ""
        return host.removeprefix("www.")
    except Exception:
        return ""


def clean(node: BeautifulSoup) -> str:
    for tag in node.find_all(NOISE_TAGS):
        tag.decompose()
    for tag in node.find_all(True):
        attrs = tag.attrs if tag.attrs is not None else {}
        classes = " ".join(attrs.get("class", []) or [])
        ids = attrs.get("id", "") or ""
        if any(k in classes.lower() or k in ids.lower()
               for k in ("related", "recommend", "promo", "banner", "subscribe",
                         "newsletter", "cookie", "social", "share", "comment",
                         "sidebar", "widget", "popup", "modal", "advert")):
            tag.decompose()

    text = node.get_text(separator="\n")
    lines = [ln.strip() for ln in text.splitlines()]
    lines = [ln for ln in lines if len(ln) > 3]
    result = re.sub(r"\n{3,}", "\n\n", "\n".join(lines))
    return result.strip()


def extract(html: str, url: str) -> tuple[str, str]:
    """Returns (title, content)."""
    soup = BeautifulSoup(html, "lxml")

    # Title
    title = ""
    og_title = soup.find("meta", property="og:title")
    if og_title and og_title.get("content"):
        title = og_title["content"].strip()
    elif soup.title:
        title = soup.title.get_text().strip()

    domain = get_domain(url)

    selectors = DOMAIN_RULES.get("_generic")
    for d, sels in DOMAIN_RULES.items():
        if d != "_generic" and d in domain:
            selectors = sels
            break

    for sel in selectors:
        node = soup.select_one(sel)
        if node:
            text = clean(node)
            if len(text) > 150:
                return title, text

    body = soup.find("body")
    return title, clean(body) if body else ""


def fetch(url: str) -> str:
    with httpx.Client(
        headers=HEADERS,
        follow_redirects=True,
        verify=False,
        timeout=20,
    ) as client:
        resp = client.get(url)
        resp.raise_for_status()
        return resp.text


def scrape(url: str) -> None:
    print(f"\n{'─'*60}")
    print(f"URL   : {url}")
    print(f"{'─'*60}")

    try:
        html = fetch(url)
    except Exception as e:
        print(f"[ERROR] Gagal fetch: {e}")
        return

    title, content = extract(html, url)

    print(f"TITLE : {title}")
    print(f"WORDS : {len(content.split())}")
    print(f"{'─'*60}")
    print(content)
    print(f"{'─'*60}\n")

