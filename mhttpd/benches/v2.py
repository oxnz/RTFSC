import logging
import asyncio
import httpx

logging.basicConfig(
    format="%(levelname)s [%(asctime)s] %(name)s - %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S",
    level=logging.INFO
)

async def foo():
    async with httpx.AsyncClient(http1=False, http2=True) as client:
        for i in range(10):
            response = await client.post('http://127.0.0.1:8000', json={
                'a': 1,
                'b': 'hello',
            })
            # logging.info(response.elapsed)

if __name__ == '__main__':
    asyncio.run(foo())
