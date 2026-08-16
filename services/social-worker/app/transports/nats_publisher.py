from __future__ import annotations

import json

from ..config import Config
from ..poller import PollingWorker


async def publish_worker(config: Config, worker: PollingWorker) -> None:
    try:
        import nats
    except ImportError as exc:
        raise RuntimeError("mode NATS membutuhkan dependency nats-py") from exc

    options = {"servers": [config.nats_url]}
    if config.nats_creds:
        options["user_credentials"] = config.nats_creds
    nc = await nats.connect(**options)
    try:
        async for record in worker.subscribe():
            await nc.publish(config.nats_subject, json.dumps(record.as_dict(), ensure_ascii=False).encode())
    finally:
        await nc.drain()
