import asyncio
import json
from aiohttp import web
from typing import Optional

class APIGateway:
    def __init__(self, node_id: str, passport: Optional[object] = None):
        self.node_id = node_id
        self.queries_processed = 0
        self.passport = passport

    def handle_ws_message(self, message):
        pass

    async def handle_status(self, request):
        return web.json_response({"status": "ok", "node_id": self.node_id})

    async def handle_oracle_feeds(self, request):
        return web.json_response({"feeds": []})

    async def handle_identity_passport(self, request):
        address = request.query.get("address")
        if not address:
            return web.Response(status=400, text="Missing address")
        if not self.passport:
            return web.Response(status=500, text="Passport gateway not initialized")
        proof = await self.passport.is_human(address)
        return web.json_response({
            "address": proof.address,
            "is_human": proof.is_human,
            "score": proof.score,
            "stamps": proof.stamps,
            "orcid_verified": proof.orcid_verified,
        })

    async def handle_dao_verify_voter(self, request):
        address = request.query.get("address")
        if not address:
            return web.Response(status=400, text="Missing address")
        if not self.passport:
            return web.Response(status=500, text="Passport gateway not initialized")
        can_vote = await self.passport.verify_dao_voter(address)
        return web.json_response({"address": address, "can_vote": can_vote})

    async def start_http_server(self):
        app = web.Application()
        app.add_routes([
            web.get('/v1/status', self.handle_status),
            web.get('/v1/oracle/feeds', self.handle_oracle_feeds),
            web.get('/v1/identity/passport', self.handle_identity_passport),
            web.get('/v1/dao/verify-voter', self.handle_dao_verify_voter),
        ])
        runner = web.AppRunner(app)
        await runner.setup()
        site = web.TCPSite(runner, '0.0.0.0', 8080)
        await site.start()
        print(f"Server started on port 8080")
