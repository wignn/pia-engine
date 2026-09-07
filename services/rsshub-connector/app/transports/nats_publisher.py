from __future__ import annotations

import json
import logging

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
    log = logging.getLogger('social_worker')
    log.info('NATS publisher connected subject=%s', config.nats_subject)
    stream = worker.subscribe()
    await worker.start()
    try:
        async for record in stream:
            payload = json.dumps(record.as_dict(), ensure_ascii=False).encode()
            await nc.publish(config.nats_subject, payload)
            await nc.flush()
            log.info('published subject=%s event_id=%s', config.nats_subject, record.event_id)
    finally:
        await nc.drain()
