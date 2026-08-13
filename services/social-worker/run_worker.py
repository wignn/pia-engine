from __future__ import annotations

import asyncio
import logging

from app.config import Config
from app.poller import PollingWorker
from app.transports.nats_publisher import publish_worker


async def main() -> None:
    config = Config.from_env()
    logging.basicConfig(level="INFO")
    worker = PollingWorker(config)
    await worker.start()
    try:
        await publish_worker(config, worker)
    finally:
        await worker.stop()


if __name__ == "__main__":
    asyncio.run(main())
