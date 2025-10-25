import logging
import asyncio
import httpx

logging.basicConfig(
    format="%(levelname)s [%(asctime)s] %(name)s - %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S",
    level=logging.INFO
)

async def fetch(client: httpx.AsyncClient, i):
    logging.info(f'req {i} sent')
    response = await client.post('http://127.0.0.1:8000', json={'a': i, 'b': 'hello',}, timeout=None)
    logging.info(f'req {i} done')
    return response

async def foo():
    async with httpx.AsyncClient(http1=False, http2=True) as client:
        for i in range(10):
            tasks = [fetch(client, i) for i in range(4)]
            results = await asyncio.gather(*tasks)
            # logging.info(response.elapsed)
            logging.info(results)

if __name__ == '__main__':
    asyncio.run(foo())
