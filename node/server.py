import asyncio
from api_gateway import APIGateway
from passport_gateway import PassportGateway

class ArkheNode:
    def __init__(self, config_path: str = "config.yaml"):
        self.node_id = "demo-node"
        self.config = {"passport_enabled": True}
        self.passport = PassportGateway()
        self.api = APIGateway(node_id=self.node_id, passport=self.passport)

    async def start(self):
        if self.config.get("passport_enabled", True):
            await self.passport.start()
        await self.api.start_http_server()

if __name__ == "__main__":
    node = ArkheNode()
    loop = asyncio.get_event_loop()
    try:
        loop.run_until_complete(node.start())
        loop.run_forever()
    except KeyboardInterrupt:
        pass
    finally:
        loop.run_until_complete(node.passport.stop())
        loop.close()
