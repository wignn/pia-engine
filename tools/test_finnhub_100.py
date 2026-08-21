#!/usr/bin/env python3
"""Secret-safe Finnhub websocket diagnostic for the ingestion configuration."""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import re
import socket
import ssl
import sys
import threading
import time
import unittest
from dataclasses import dataclass, field
from pathlib import Path
from urllib.parse import urlsplit

MAX_NON_CRYPTO = 100
MAX_PER_KEY = 50
SUBSCRIBE_DELAY = 0.05
DEFAULT_ENV = Path("infra/env/.env.ingestion")


@dataclass(frozen=True)
class Mapping:
    provider: str
    public: str
    asset_type: str
    group: str


@dataclass(frozen=True)
class Candidate:
    mapping: Mapping
    symbol: str | None
    reason: str | None = None


@dataclass
class SymbolResult:
    public: str
    provider: str
    status: str = "not_sent"
    trade: bool = False
    messages: int = 0
    error: str | None = None


@dataclass
class SessionResult:
    label: str
    allocated: list[Candidate]
    handshake: str = "not_started"
    handshake_error: str | None = None
    messages: int = 0
    trades: int = 0
    provider_errors: list[str] = field(default_factory=list)
    close_reason: str = "not_started"
    subscriptions: int = 0
    acknowledgements: int = 0
    symbols: dict[str, SymbolResult] = field(default_factory=dict)


class HandshakeError(Exception):
    def __init__(self, category: str):
        super().__init__(category)
        self.category = category


class WebSocket:
    def __init__(self, sock: socket.socket):
        self.sock = sock
        self.buffer = bytearray()

    @classmethod
    def connect(cls, url: str, timeout: float = 10.0) -> "WebSocket":
        parts = urlsplit(url)
        if parts.scheme != "wss" or not parts.hostname:
            raise HandshakeError("invalid_websocket_url")
        port = parts.port or 443
        path = parts.path or "/"
        if parts.query:
            path += "?" + parts.query
        raw: socket.socket | None = None
        sock: socket.socket | None = None
        try:
            raw = socket.create_connection((parts.hostname, port), timeout=timeout)
            context = ssl.create_default_context()
            sock = context.wrap_socket(raw, server_hostname=parts.hostname)
            raw = None
            sock.settimeout(timeout)
            key = base64.b64encode(hashlib.sha256(f"{time.time_ns()}".encode()).digest()[:16]).decode()
            host = parts.hostname if port == 443 else f"{parts.hostname}:{port}"
            request = (
                f"GET {path} HTTP/1.1\r\n"
                f"Host: {host}\r\n"
                "Upgrade: websocket\r\n"
                "Connection: Upgrade\r\n"
                f"Sec-WebSocket-Key: {key}\r\n"
                "Sec-WebSocket-Version: 13\r\n"
                "User-Agent: atlsd-finnhub-diagnostic/1\r\n\r\n"
            ).encode("ascii")
            sock.sendall(request)
            response = bytearray()
            while b"\r\n\r\n" not in response and len(response) <= 65536:
                chunk = sock.recv(4096)
                if not chunk:
                    break
                response.extend(chunk)
            marker = b"\r\n\r\n"
            header_end = bytes(response).find(marker)
            if header_end < 0:
                raise HandshakeError("invalid_handshake_response")
            header = bytes(response[:header_end])
            first = header.split(b"\r\n", 1)[0].decode("latin1", "replace")
            match = re.match(r"HTTP/\d(?:\.\d)?\s+(\d{3})", first)
            if not match:
                raise HandshakeError("invalid_handshake_response")
            status = int(match.group(1))
            if status != 101:
                raise HandshakeError(http_category(status))
            if b"upgrade: websocket" not in header.lower():
                raise HandshakeError("missing_websocket_upgrade")
            ws = cls(sock)
            ws.buffer.extend(bytes(response[header_end + len(marker) :]))
            sock = None
            return ws
        except HandshakeError:
            raise
        except socket.timeout as exc:
            raise HandshakeError("network_timeout") from exc
        except (OSError, ssl.SSLError) as exc:
            raise HandshakeError("network_error") from exc
        finally:
            if sock is not None:
                sock.close()
            if raw is not None:
                raw.close()


    def send_text(self, text: str) -> None:
        self._send_frame(0x1, text.encode("utf-8"))

    def send_pong(self, payload: bytes = b"") -> None:
        self._send_frame(0xA, payload)

    def send_close(self) -> None:
        try:
            self._send_frame(0x8, b"\x03\xe8")
        except OSError:
            pass

    def close(self) -> None:
        try:
            self.sock.close()
        except OSError:
            pass

    def recv(self) -> tuple[int, bytes]:
        first, second = self._read_exact(2)
        opcode = first & 0x0F
        masked = bool(second & 0x80)
        length = second & 0x7F
        if length == 126:
            length = int.from_bytes(self._read_exact(2), "big")
        elif length == 127:
            length = int.from_bytes(self._read_exact(8), "big")
        if length > 4 * 1024 * 1024:
            raise ValueError("frame_too_large")
        mask = self._read_exact(4) if masked else b""
        payload = bytearray(self._read_exact(length))
        if masked:
            for i in range(length):
                payload[i] ^= mask[i % 4]
        return opcode, bytes(payload)

    def _read_exact(self, size: int) -> bytes:
        while len(self.buffer) < size:
            chunk = self.sock.recv(max(4096, size - len(self.buffer)))
            if not chunk:
                raise ConnectionError("socket_closed")
            self.buffer.extend(chunk)
        result = bytes(self.buffer[:size])
        del self.buffer[:size]
        return result

    def _send_frame(self, opcode: int, payload: bytes) -> None:
        length = len(payload)
        if length < 126:
            header = bytes([0x80 | opcode, 0x80 | length])
        elif length < 65536:
            header = bytes([0x80 | opcode, 0x80 | 126]) + length.to_bytes(2, "big")
        else:
            header = bytes([0x80 | opcode, 0x80 | 127]) + length.to_bytes(8, "big")
        mask = hashlib.sha256(f"{time.time_ns()}".encode()).digest()[:4]
        body = bytes(value ^ mask[i % 4] for i, value in enumerate(payload))
        self.sock.sendall(header + mask + body)


def http_category(status: int) -> str:
    if status in (401, 403):
        return "auth_rejected"
    if status == 400:
        return "bad_handshake_request"
    if status == 429:
        return "rate_limited"
    if 500 <= status <= 599:
        return "provider_server_error"
    return f"http_{status}"


def load_env(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("export "):
            line = line[7:].lstrip()
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip()
        if len(value) >= 2 and value[0] == value[-1] and value[0] in "\"'":
            value = value[1:-1]
        values[key] = value
    return values


def parse_mappings(raw: str, group: str, limit: int) -> list[Mapping]:
    result: list[Mapping] = []
    for item in raw.split(","):
        parts = [part.strip() for part in item.split("|")]
        if len(parts) != 3 or not all(parts):
            continue
        result.append(Mapping(parts[0], parts[1].upper(), parts[2].lower(), group))
        if len(result) == limit:
            break
    return result


def configured_symbols(env: dict[str, str]) -> tuple[list[Mapping], list[Mapping], list[Mapping], list[Mapping]]:
    primary = parse_mappings(env.get("PRIMARY_FX_SYMBOLS", ""), "primary_fx", MAX_PER_KEY)
    secondary_raw = env.get("SECONDARY_FX_SYMBOLS", "")
    if not secondary_raw.strip():
        secondary_raw = env.get("SECONDARY_SYMBOLS", "")
    secondary = parse_mappings(secondary_raw, "secondary_fx", MAX_PER_KEY)
    indices = parse_mappings(env.get("INDEX_FEED_SYMBOLS", ""), "index", MAX_NON_CRYPTO)
    stocks = parse_mappings(env.get("STOCK_FEED_SYMBOLS", ""), "stock", MAX_NON_CRYPTO)
    return primary, secondary, indices, stocks


def dedupe_and_cap(groups: tuple[list[Mapping], ...]) -> list[Mapping]:
    seen: set[str] = set()
    result: list[Mapping] = []
    for group in groups:
        for mapping in group:
            if mapping.public not in seen and len(result) < MAX_NON_CRYPTO:
                seen.add(mapping.public)
                result.append(mapping)
    return result


def candidate(mapping: Mapping) -> Candidate:
    provider = mapping.provider.strip().upper()
    if mapping.asset_type == "forex":
        symbol = provider.replace(":", "")
        return Candidate(mapping, symbol)
    if mapping.asset_type == "stock":
        exchange = provider.split(":", 1)[0] if ":" in provider else ""
        if exchange in {"NASDAQ", "NYSE", "NYSEARCA", "AMEX", "LSE"}:
            return Candidate(mapping, mapping.public)
        return Candidate(mapping, None, "exchange_not_confident_for_finnhub")
    return Candidate(mapping, None, "tradingview_mapping_not_finnhub")


def canonical(symbol: str) -> str:
    return "".join(char for char in symbol.upper() if char.isalnum())


def provider_error_category(message: str) -> str:
    text = message.lower()
    if any(word in text for word in ("auth", "token", "api key", "permission")):
        return "auth_rejected"
    if "limit" in text or "maximum" in text:
        return "subscription_limit"
    if "symbol" in text or "subscribe" in text:
        return "symbol_rejected"
    return "provider_error"


def run_session(label: str, key: str, allocated: list[Candidate], url_template: str, duration: float) -> SessionResult:
    result = SessionResult(label, allocated)
    result.symbols = {
        item.mapping.public: SymbolResult(item.mapping.public, item.mapping.provider)
        for item in allocated
    }
    if not key.strip():
        result.handshake = "failed"
        result.handshake_error = "missing_api_key"
        return result
    url = url_template.strip().replace("{token}", key.strip()).replace("***", key.strip())
    if "{token}" in url or "***" in url:
        result.handshake = "failed"
        result.handshake_error = "missing_token_placeholder"
        return result

    ws: WebSocket | None = None
    aliases: dict[str, str] = {}
    try:
        ws = WebSocket.connect(url)
        result.handshake = "ok"
        supported = [item for item in allocated if item.symbol]
        for item in supported:
            assert item.symbol is not None
            aliases[canonical(item.symbol)] = item.mapping.public
            try:
                ws.send_text(json.dumps({"type": "subscribe", "symbol": item.symbol}, separators=(",", ":")))
                result.subscriptions += 1
                result.symbols[item.mapping.public].status = "sent"
                time.sleep(SUBSCRIBE_DELAY)
            except (OSError, ConnectionError):
                result.symbols[item.mapping.public].status = "send_failed"
                result.symbols[item.mapping.public].error = "network_error"
                break

        deadline = time.monotonic() + max(0.1, duration)
        while time.monotonic() < deadline:
            remaining = max(0.1, deadline - time.monotonic())
            ws.sock.settimeout(min(1.0, remaining))
            try:
                opcode, payload = ws.recv()
            except socket.timeout:
                result.close_reason = "read_timeout"
                continue
            except ssl.SSLWantReadError:
                continue
            except ssl.SSLError:
                result.close_reason = "tls_error"
                break
            except ConnectionError as exc:
                result.close_reason = str(exc)
                break
            except (OSError, ValueError):
                result.close_reason = "socket_error"
                break
            result.messages += 1
            if opcode == 0x8:
                result.close_reason = "server_close"
                break
            if opcode == 0x9:
                try:
                    ws.send_pong(payload)
                except OSError:
                    break
                continue
            if opcode != 0x1:
                continue
            try:
                message = json.loads(payload.decode("utf-8"))
            except (UnicodeDecodeError, json.JSONDecodeError):
                continue
            message_type = str(message.get("type", ""))
            if message_type == "subscribed":
                result.acknowledgements += 1
                public = aliases.get(canonical(str(message.get("symbol", ""))))
                if public and public in result.symbols:
                    result.symbols[public].status = "accepted"
                elif result.acknowledgements <= result.subscriptions:
                    pending = [
                        item for item in result.symbols.values()
                        if item.status == "sent"
                    ]
                    if len(pending) == 1:
                        pending[0].status = "accepted"
            elif message_type == "error":
                category = provider_error_category(str(message.get("msg", "")))
                if category not in result.provider_errors:
                    result.provider_errors.append(category)
                for item in result.symbols.values():
                    if item.status == "sent":
                        item.status = "rejected"
                        item.error = category
            elif message_type == "trade":
                for trade in message.get("data", []):
                    if not isinstance(trade, dict):
                        continue
                    public = aliases.get(canonical(str(trade.get("s", ""))))
                    if public and public in result.symbols:
                        item = result.symbols[public]
                        if item.status in {"sent", "not_sent"}:
                            item.status = "accepted"
                        if not item.trade:
                            result.trades += 1
                        item.trade = True
                        item.messages += 1
            elif message_type == "ping":
                try:
                    ws.send_pong()
                except OSError:
                    result.close_reason = "pong_failed"
                    break

        if result.close_reason == "not_started":
            result.close_reason = "duration_elapsed"
        if result.handshake == "ok" and result.subscriptions and result.acknowledgements == 0:
            result.close_reason = "no_subscription_ack"
            for item in result.symbols.values():
                if item.status == "sent":
                    item.status = "no_ack"
        elif result.handshake == "ok" and result.subscriptions == 0:
            result.close_reason = "no_supported_subscriptions"
        elif result.handshake == "ok" and result.acknowledgements > 0 and result.trades == 0:
            result.close_reason = "no_trade_during_window"
        elif result.handshake == "ok" and result.acknowledgements > 0:
            result.close_reason = "activity_received"
    except HandshakeError as exc:
        result.handshake = "failed"
        result.handshake_error = exc.category
    finally:
        if ws is not None:
            ws.send_close()
            ws.close()

    for item in allocated:
        symbol_result = result.symbols[item.mapping.public]
        if not item.symbol:
            symbol_result.status = "unsupported"
            symbol_result.error = item.reason
        elif symbol_result.status == "sent":
            symbol_result.status = "no_ack"
    return result


def print_result(result: SessionResult) -> None:
    print(
        f"{result.label}: allocation={len(result.allocated)} "
        f"handshake={result.handshake} subscriptions={result.subscriptions} "
        f"acks={result.acknowledgements} messages={result.messages} trades={result.trades} "
        f"close={result.close_reason}"
    )
    if result.handshake_error:
        print(f"{result.label}: handshake_error={result.handshake_error}")
    if result.provider_errors:
        print(f"{result.label}: provider_errors={','.join(result.provider_errors)}")
    for item in result.allocated:
        symbol = result.symbols[item.mapping.public]
        extra = f" error={symbol.error}" if symbol.error else ""
        print(
            f"{result.label}: {symbol.public} provider={symbol.provider} "
            f"status={symbol.status} trade={'yes' if symbol.trade else 'no'}{extra}"
        )


def offline_check(env: dict[str, str]) -> int:
    primary, secondary, indices, stocks = configured_symbols(env)
    ordered = dedupe_and_cap((primary, secondary, indices, stocks))
    checks = {
        "primary_fx": len(primary) == 6,
        "secondary_fx": len(secondary) == 6,
        "non_crypto": len(ordered) == 100,
        "unique_public": len({item.public for item in ordered}) == len(ordered),
        "primary_key": bool(env.get("PRIMARY_FX_API_KEY", "").strip()),
        "secondary_key": bool((env.get("SECONDARY_FX_API_KEY") or env.get("SECONDRY_FX_API_KEY", "")).strip()),
    }
    print("offline: " + " ".join(f"{name}={'ok' if value else 'failed'}" for name, value in checks.items()))
    return 0 if all(checks.values()) else 1


def live_check(env: dict[str, str], mode: str, max_symbols: int, duration: float) -> int:
    primary, secondary, indices, stocks = configured_symbols(env)
    ordered = dedupe_and_cap((primary, secondary, indices, stocks))[:max_symbols]
    if mode == "configured":
        allocations = (list(map(candidate, primary)), list(map(candidate, secondary)))
    else:
        allocations = (
            [candidate(item) for item in ordered[:MAX_PER_KEY]],
            [candidate(item) for item in ordered[MAX_PER_KEY : MAX_PER_KEY * 2]],
        )
    supported = sum(1 for group in allocations for item in group if item.symbol)
    unsupported = sum(1 for group in allocations for item in group if not item.symbol)
    print(
        f"mode={mode} configured_non_crypto={len(ordered)} "
        f"finnhub_candidates={supported} unsupported={unsupported}"
    )
    print(f"primary_fx_configured={len(primary)} secondary_fx_configured={len(secondary)}")

    results: list[SessionResult | None] = [None, None]
    threads: list[threading.Thread] = []
    keys = (
        env.get("PRIMARY_FX_API_KEY", ""),
        env.get("SECONDARY_FX_API_KEY") or env.get("SECONDRY_FX_API_KEY", ""),
    )
    for index, (label, key, group) in enumerate(
        zip(("primary", "secondary"), keys, allocations)
    ):
        thread = threading.Thread(
            target=lambda i=index, l=label, k=key, g=group: results.__setitem__(
                i, run_session(l, k, g, env.get("PRIMARY_FX_WS_URL", ""), duration)
            ),
            daemon=True,
        )
        threads.append(thread)
        thread.start()
    for thread in threads:
        thread.join()
    completed = [result for result in results if result is not None]
    for result in completed:
        print_result(result)

    failed = any(result.handshake != "ok" for result in completed)
    for result in completed:
        for item in result.allocated:
            symbol = result.symbols[item.mapping.public]
            if item.symbol and symbol.status in {"rejected", "send_failed", "no_ack"}:
                failed = True
            if item.symbol and not symbol.trade:
                failed = True
    return 1 if failed else 0


class DiagnosticTests(unittest.TestCase):
    def test_mapping_parser_rejects_malformed_and_caps(self) -> None:
        raw = ",".join(f"FX:S{i}|S{i}|forex" for i in range(60))
        parsed = parse_mappings("FX:BAD|BAD,missing," + raw, "primary_fx", MAX_PER_KEY)
        self.assertEqual(len(parsed), MAX_PER_KEY)
        self.assertEqual(parsed[0].public, "S0")

    def test_dedupe_preserves_priority_and_cap(self) -> None:
        primary = [Mapping("FX:A", "A", "forex", "primary_fx")]
        secondary = [Mapping("FX:A", "A", "forex", "secondary_fx"), Mapping("FX:B", "B", "forex", "secondary_fx")]
        result = dedupe_and_cap((primary, secondary, [], []))
        self.assertEqual([item.public for item in result], ["A", "B"])

    def test_candidate_only_marks_supported_finnhub_assets(self) -> None:
        self.assertEqual(candidate(Mapping("EURUSD", "EURUSD", "forex", "primary_fx")).symbol, "EURUSD")
        self.assertEqual(candidate(Mapping("NASDAQ:AAPL", "AAPL", "stock", "stock")).symbol, "AAPL")
        self.assertIsNone(candidate(Mapping("TVC:DXY", "DXY", "index", "index")).symbol)



def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--env-file", type=Path, default=DEFAULT_ENV)
    parser.add_argument("--mode", choices=("configured", "all"), default="configured")
    parser.add_argument("--duration", type=float, default=10.0)
    parser.add_argument("--max-symbols", type=int, default=MAX_NON_CRYPTO)
    parser.add_argument("--offline", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.self_test:
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(DiagnosticTests)
        return 0 if unittest.TextTestRunner(verbosity=1).run(suite).wasSuccessful() else 1
    if args.max_symbols < 1 or args.max_symbols > MAX_NON_CRYPTO:
        print("error: --max-symbols must be between 1 and 100", file=sys.stderr)
        return 2
    try:
        env = load_env(args.env_file)
        if args.offline:
            return offline_check(env)
        return live_check(env, args.mode, args.max_symbols, args.duration)
    except FileNotFoundError:
        print(f"error: env file not found: {args.env_file}", file=sys.stderr)
        return 2
    except OSError:
        print("error: could not read env file", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
