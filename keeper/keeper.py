"""
Equilibria Protocol — AI Keeper / Scoring Engine
=================================================
Monitors Portaldot markets, scores opportunities,
posts to on-chain OpportunityRegistry if composite >= threshold.

Usage:
  python3 keeper.py                  # connects to local node
  python3 keeper.py --public         # connects to public dev node
"""

import sys
import time
import random
import argparse
from datetime import datetime
from substrateinterface import SubstrateInterface, Keypair
import asyncio
import threading
import websockets
import json
import queue

# WebSocket broadcast
_loop = None
_clients = set()

def start_ws_server():
    global _loop
    _loop = asyncio.new_event_loop()
    asyncio.set_event_loop(_loop)
    
    async def handler(ws):
        _clients.add(ws)
        try:
            await ws.wait_closed()
        finally:
            _clients.discard(ws)
    
    async def serve():
        async with websockets.serve(handler, "localhost", 8765):
            await asyncio.Future()
    
    _loop.run_until_complete(serve())


_log_queue = queue.Queue()
_clients = set()

def broadcast_log(level, message, score=None):
    _log_queue.put_nowait({
        "time": datetime.now().strftime("%H:%M:%S"),
        "type": level.strip(),
        "message": message,
        "score": score
    })

async def ws_handler(websocket):
    _clients.add(websocket)
    try:
        await websocket.wait_closed()
    finally:
        _clients.discard(websocket)

async def ws_broadcaster():
    while True:
        try:
            msg = _log_queue.get_nowait()
            if _clients:
                data = json.dumps(msg)
                await asyncio.gather(*[c.send(data) for c in list(_clients)], return_exceptions=True)
        except queue.Empty:
            pass
        await asyncio.sleep(0.1)

async def ws_main():
    async with websockets.serve(ws_handler, "localhost", 8765):
        await ws_broadcaster()

def start_ws_server():
    asyncio.run(ws_main())

threading.Thread(target=start_ws_server, daemon=True).start()

#  CONFIG 

LOCAL_NODE      = "ws://127.0.0.1:9944"
PUBLIC_NODE     = "wss://drip-backend-production-8d86.up.railway.app/node"

KEEPER_URI      = "//Bob"    # Authorized keeper wallet
OWNER_URI       = "//Alice"  # Pool owner

SCAN_INTERVAL   = 5           # Seconds between market scans
MIN_SCORE       = 70          #Composite threshold to execute
MAX_RISK        = 60          # Hard risk ceiling — reject above this

# Scoring weights
W_PROFIT        = 0.30
W_RISK          = 0.25           
W_STABILITY     = 0.35
W_CONFIDENCE    = 0.10

# Contract addresses (fill in once deployed)
STABILITY_POOL_ADDRESS    = None   # e.g. "5D..."
OPPORTUNITY_REGISTRY_ADDRESS = None  # e.g. "5D..."

# Trading pairs to monitor
PAIRS = ["POT/rUSD", "POT/USDC", "rUSD/POT"]

#  HELPERS

def ts():
    return datetime.now().strftime("%H:%M:%S")

def log(level, msg, score=None):
    print(f"[{ts()}] [{level:<7}] {msg}")
    broadcast_log(level, msg, score)
    level_colors = {
        "SCAN":    "",
        "DETECT":  "",
        "SCORE":   "",
        "APPROVE": "",
        "REJECT":  "",
        "EXECUTE": "",
        "ERROR":   "",
        "INFO":    "",
    }
    print(f"[{ts()}] [{level:<7}] {msg}")

#  CONNECTION 

def connect(use_public=False):
    url = PUBLIC_NODE if use_public else LOCAL_NODE
    log("INFO", f"Connecting to {'public' if use_public else 'local'} node: {url}")

    try:
        node = SubstrateInterface(url=url, ss58_format=42)
        keeper = Keypair.create_from_uri(KEEPER_URI)
        alice  = Keypair.create_from_uri(OWNER_URI)

        block = node.get_block_number(node.get_chain_head())
        log("INFO", f"Connected to {node.chain} — block #{block}")
        log("INFO", f"Owner  (Alice): {alice.ss58_address}")
        log("INFO", f"Keeper (Bob):   {keeper.ss58_address}")

        if OPPORTUNITY_REGISTRY_ADDRESS:
            log("INFO", f"OpportunityRegistry: {OPPORTUNITY_REGISTRY_ADDRESS}")
        else:
            log("INFO", "OpportunityRegistry: not set — running in simulation mode")

        return node, keeper, alice

    except Exception as e:
        log("ERROR", f"Connection failed: {e}")
        log("INFO", "Tip: make sure your local node is running:")
        log("INFO", "  cd ~/portaldot && ./portaldot_dev --dev --alice --tmp")
        sys.exit(1)

# MARKET DATA 

def fetch_market_data(node):
    """
    Production: query DEX pool storage via node.query()
    Currently: realistic simulation — replace with live DEX reads
    when a DEX contract is deployed on Portaldot.

    To connect to a real DEX pool:
      result = node.query('Contracts', 'ContractInfoOf', [DEX_ADDRESS])
    """
    markets = []
    for pair in PAIRS:
        markets.append({
            "pair":    pair,
            "spread":  round(random.uniform(0.2, 3.8), 2),
            "depth":   random.randint(4000, 90000),
            "vol_30":  round(random.uniform(0.4, 4.5), 2),
            "volume":  random.randint(8000, 600000),
        })
    return markets

#  SCORING 

def score(market):
    spread = market["spread"]
    depth  = market["depth"]
    vol    = market["vol_30"]
    volume = market["volume"]

    # Profitability: wider spread = more profit potential
    profitability = min(int((spread / 3.8) * 100), 100)

    # Risk: high volatility + low depth = higher risk
    risk = min(int((vol / 4.5) * 60 + (1 - min(depth / 90000, 1)) * 40), 100)

    # Stability impact: closing this spread benefits the market
    stability = min(int((spread / 3.8) * 75 + min(depth / 90000, 1) * 25), 100)

    # Confidence: volume validates the signal
    confidence = min(int((volume / 600000) * 100), 100)

    composite = (
        profitability * W_PROFIT +
        (100 - risk)  * W_RISK +
        stability     * W_STABILITY +
        confidence    * W_CONFIDENCE
    )

    return {
        "profitability": profitability,
        "risk":          risk,
        "stability":     stability,
        "confidence":    confidence,
        "composite":     round(composite, 1),
    }

def recommended_size(market, pool_tvl=4_000_000, max_pct=0.20):
    base   = min(market["depth"] * 0.5, pool_tvl * max_pct)
    factor = min(market["spread"] / 2.0, 1.5)
    return int(base * factor)

#  ON-CHAIN POSTING 

opp_counter = 0

def post_to_registry(node, keeper, opp_id, market, scores):
    """
    Post a qualifying opportunity to the OpportunityRegistry contract.
    Contract address must be set in OPPORTUNITY_REGISTRY_ADDRESS.
    """
    if not OPPORTUNITY_REGISTRY_ADDRESS:
        log("EXECUTE", f"[SIM] Opportunity #{opp_id} would post to registry — set contract address to go live")
        return True

    try:
        # Encode the call data for post_opportunity(id, pair, composite)
        # Full ABI encoding via contract metadata would go here
        call = node.compose_call(
            call_module="Contracts",
            call_function="call",
            call_params={
                "dest":      OPPORTUNITY_REGISTRY_ADDRESS,
                "value":     0,
                "gas_limit": 10_000_000_000,
                "data":      b""   # Replace with encoded post_opportunity call
            }
        )
        extrinsic = node.create_signed_extrinsic(call=call, keypair=keeper)
        result = node.submit_extrinsic(extrinsic, wait_for_inclusion=False)
        log("EXECUTE", f"Opportunity #{opp_id} posted — extrinsic: {result.extrinsic_hash}")
        return True

    except Exception as e:
        log("ERROR", f"Failed to post opportunity #{opp_id}: {e}")
        return False

#  MAIN LOOP 

def run(use_public=False):
    global opp_counter
    node, keeper, alice = connect(use_public)

    print("=" * 65)
    print("  EQUILIBRIA PROTOCOL — AI SCORING ENGINE")
    print(f"  Threshold: {MIN_SCORE} | Max risk: {MAX_RISK} | Interval: {SCAN_INTERVAL}s")
    print(f"  Weights: Profit×{W_PROFIT} Risk×{W_RISK} Stability×{W_STABILITY} Confidence×{W_CONFIDENCE}")
    print("=" * 65)

    scan_number = 0

    while True:
        try:
            scan_number += 1
            block = node.get_block_number(node.get_chain_head())
            log("SCAN", f"Scan #{scan_number} — block #{block} — monitoring {len(PAIRS)} pairs")

            markets = fetch_market_data(node)
            qualified = []

            for market in markets:
                s = score(market)
                log("SCORE", (
                    f"{market['pair']:12} "
                    f"spread={market['spread']}% "
                    f"P={s['profitability']:3} "
                    f"R={s['risk']:3} "
                    f"S={s['stability']:3} "
                    f"C={s['confidence']:3} "
                    f"→ composite={s['composite']}"
                ))

                # Gate 1: risk ceiling
                if s["risk"] > MAX_RISK:
                    log("REJECT", f"{market['pair']} — risk {s['risk']} exceeds ceiling {MAX_RISK}")
                    continue

                # Gate 2: composite threshold
                if s["composite"] < MIN_SCORE:
                    log("REJECT", f"{market['pair']} — composite {s['composite']} below threshold {MIN_SCORE}")
                    continue

                qualified.append((market, s))

            # Process qualified opportunities
            if qualified:
                # Sort by composite score descending — best first
                qualified.sort(key=lambda x: x[1]["composite"], reverse=True)
                for market, s in qualified:
                    opp_counter += 1
                    size = recommended_size(market)
                    log("APPROVE", f"{market['pair']} — composite {s['composite']} | size: {size:,} POT | opp #{opp_counter}", score=s['composite'])
                    post_to_registry(node, keeper, opp_counter, market, s)
            else:
                log("SCAN", "No qualifying opportunities this scan")

            print("-" * 65)
            time.sleep(SCAN_INTERVAL)

        except KeyboardInterrupt:
            print(f"\n[{ts()}] Engine stopped by user.")
            break
        except Exception as e:
            log("ERROR", str(e))
            time.sleep(SCAN_INTERVAL)

#  ENTRY POINT 

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Equilibria AI Keeper")
    parser.add_argument("--public", action="store_true", help="Use public dev node instead of local")
    args = parser.parse_args()
    run(use_public=args.public)