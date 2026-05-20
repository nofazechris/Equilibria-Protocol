# Equilibria Protocol

Autonomous market stabilization infrastructure for the Portaldot ecosystem.

## What It Does

Equilibria pools reserve capital, uses an AI scoring engine to detect price inefficiencies across Portaldot DEX markets, and deploys liquidity to narrow spreads and generate yield for depositors.

## Stack

| Layer | Technology |
| Blockchain | Portaldot (Substrate) |
| Smart Contracts | Rust + ink! v3 |
| AI Engine | Python + substrate-interface |
| Frontend | React + Vite + Recharts |

## Quick Start

```bash
# Frontend
cd frontend && npm install && npm run dev

# Smart contracts
cd contracts/stability_pool && cargo contract build

# AI Keeper
cd keeper && pip install substrate-interface && python3 keeper.py
```

## Node

Public testnet: `wss://drip-backend-production-8d86.up.railway.app/node`

## Docs

See `EQUILIBRIA_PROJECT_SUMMARY.md` for full architecture, contract interface, and onboarding guide.
