# Equilibria Protocol

## What Is Equilibria?

Equilibria is a decentralized market stabilization protocol built natively on the Portaldot blockchain. It pools reserve capital from depositors, uses an AI scoring engine to detect price inefficiencies across DEX trading pairs, and deploys liquidity intelligently to narrow spreads and generate yield.

The core thesis: most emerging blockchain ecosystems suffer from thin liquidity and fragmented markets. Equilibria is the coordinated liquidity layer that Portaldot's ecosystem needs — earning yield by improving markets rather than simply extracting from them.

---

## How It Works

```
User deposits POT
       ↓
Stability Pool (ink! contract)
       ↓
AI Keeper scans DEX markets every N seconds
       ↓
Opportunity scored: Profit × 0.30 + Safety × 0.25 + Stability × 0.35 + Confidence × 0.10
       ↓
If composite score ≥ 70 → post to Opportunity Registry (ink! contract)
       ↓
Capital deployed → spread narrows → profit captured
       ↓
Yield distributed back to depositors
```

---

## Architecture

```
equilibria/
├── contracts/
│   ├── stability_pool/          ← Reserve pool + share token logic
│   │   ├── lib.rs               ← ink! v3 contract
│   │   ├── Cargo.toml
│   │   └── target/ink/          ← Compiled .wasm + .contract + .json
│   └── opportunity_registry/    ← On-chain opportunity log (DEPLOYED)
│       ├── lib.rs
│       └── Cargo.toml
│
├── keeper/                      ← AI scoring engine
│   └── keeper.py                ← Python keeper (runs independently)
│
├── frontend/                    ← React dashboard
│   ├── src/
│   │   ├── App.jsx
│   │   ├── index.css
│   │   ├── styles/
│   │   │   ├── variables.css    ← Design tokens
│   │   │   ├── base.css         ← Reset + typography
│   │   │   ├── components.css   ← Shared component classes
│   │   │   ├── dashboard.css
│   │   │   ├── opportunity.css
│   │   │   ├── pool.css
│   │   │   ├── terminal.css
│   │   │   └── analytics.css
│   │   ├── components/
│   │   │   └── Sidebar.jsx
│   │   ├── pages/
│   │   │   ├── Dashboard.jsx
│   │   │   ├── OpportunityFeed.jsx
│   │   │   ├── StabilityPool.jsx
│   │   │   ├── AITerminal.jsx
│   │   │   └── Analytics.jsx
│   │   └── data/
│   │       └── mockData.js      ← Mock data (live data coming)
│   └── package.json
│
└── scripts/
    └── deploy.py                ← Contract deployment script
```

---

## Smart Contracts

### StabilityPool (`contracts/stability_pool/`)

The reserve pool where users deposit POT and receive EQB shares.

| Function | Access | Description |
|---|---|---|
| `new(keeper, max_deploy_pct)` | Deploy | Initialize pool with keeper address and max deployment % |
| `deposit()` | Public (payable) | Deposit POT, receive EQB shares proportionally |
| `withdraw(share_amount)` | Public | Burn EQB shares, receive POT + yield |
| `record_deployment(id, amount)` | Keeper only | Log capital deployed into an opportunity |
| `record_return(id, profit, loss)` | Keeper only | Log outcome, update pool balance |
| `pause()` / `unpause()` | Owner only | Emergency circuit breaker |
| `set_keeper(address)` | Owner only | Update authorized keeper wallet |
| `get_pool_state()` | Public read | Returns TVL, shares, deployed capital, yield, status |

**Risk controls enforced on-chain:**
- Max 20% of pool deployed at any time
- All arithmetic uses saturating math (no overflow possible)
- Keeper address is a separate wallet from owner
- Emergency pause freezes deposits and withdrawals

### OpportunityRegistry (`contracts/opportunity_registry/`)

On-chain record of every opportunity the AI engine detects and acts on.

| Function | Access | Description |
|---|---|---|
| `post_opportunity(pair, scores, metadata)` | Keeper only | Record a new scored opportunity |
| `execute_opportunity(id)` | Keeper only | Mark opportunity as executed |
| `close_opportunity(id, outcome)` | Keeper only | Record final outcome |
| `get_opportunity(id)` | Public read | Read a specific opportunity |
| `get_active_opportunities()` | Public read | List all open opportunities |

**Status: DEPLOYED on Portaldot local node**

---

## AI Scoring Engine

The keeper (`keeper/keeper.py`) is a Python service that runs independently of the contracts.

### Scoring Formula

```
composite = (profitability × 0.30)
          + ((100 - risk) × 0.25)
          + (stability_impact × 0.35)
          + (confidence × 0.10)
```

**Execute if composite ≥ 70 AND risk < 60**

### Signal Inputs

| Signal | Source | Weight in scoring |
|---|---|---|
| Price spread % | DEX pool storage | Drives profitability score |
| Liquidity depth | DEX pool storage | Drives risk score |
| 30-period volatility | Computed from price history | Drives risk score |
| Volume | DEX pool storage | Drives confidence score |
| Spread × depth | Combined | Drives stability impact score |

### Output per opportunity

```json
{
  "pair": "POT/rUSD",
  "spread": 2.34,
  "profitability": 87,
  "risk": 23,
  "stability_impact": 91,
  "confidence": 78,
  "composite": 84.2,
  "action": "EXECUTE",
  "recommended_size": 18600
}
```

---

## Portaldot Node

### Run Local Node

```bash
# Download Portaldot binary
# Place in ~/portaldot/

cd ~/portaldot
./portaldot_dev --dev --alice --tmp
```

Node will start producing blocks. You should see:
```
✨ Imported #1
✨ Imported #2
...
Listening for new connections on 127.0.0.1:9944
```

### Node Details

| Property | Value |
|---|---|
| Local WebSocket | `ws://127.0.0.1:9944` |
| Public dev node | `wss://drip-backend-production-8d86.up.railway.app/node` |
| SS58 format | 42 |
| Native token | POT |
| Token decimals | 14 |
| Chain | Development |

### Pre-funded Dev Accounts

| Account | Address | Role in Equilibria |
|---|---|---|
| Alice | `5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY` | Owner / Deployer |
| Bob | `5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty` | Keeper (AI engine wallet) |

---

## Quick Start

### Prerequisites

```bash
# Rust + ink! toolchain
rustup target add wasm32-unknown-unknown
cargo install --force --locked cargo-contract --version 3.2.0

# Node.js (v20+)
node --version

# Python 3.10+
pip3 install substrate-interface
```

### 1. Start the local Portaldot node

```bash
cd ~/portaldot
./portaldot_dev --dev --alice --tmp
```

### 2. Run the frontend

```bash
cd frontend
npm install
npm run dev
# Open http://localhost:5173
```

### 3. Run the AI keeper

```bash
cd keeper
python3 keeper.py
```

You will see the scoring engine output live:
```
[14:32:01] [SCAN]   Scan #412 — monitoring 3 active pairs
[14:32:03] [DETECT] Price divergence: POT/rUSD spread 2.34%
[14:32:04] [SCORE]  Profit 87 · Risk 23 · Stability 91 · Confidence 78
[14:32:05] [APPROVE] Composite 84.2 ≥ 70 threshold. Posting to registry.
```

### 4. Build smart contracts

```bash
cd contracts/stability_pool
cargo contract build
# Output: target/ink/stability_pool.contract
```

### 5. Test node connection

```bash
python3 -c "
from substrateinterface import SubstrateInterface
node = SubstrateInterface(url='ws://127.0.0.1:9944', ss58_format=42)
print('Connected to:', node.chain)
print('Block:', node.get_block_number(node.get_chain_head()))
"
```

---

## Frontend Pages

| Page | Description |
|---|---|
| Dashboard | Protocol overview — TVL, deployed capital, yield, recent deployments |
| Opportunity Feed | Live table of AI-scored opportunities with four-metric breakdown |
| Stability Pool | Deposit/withdraw interface with transaction preview |
| AI Terminal | Live log of scoring engine reasoning — every decision visible |
| Analytics | Historical performance — yield, spread reduction, deployment outcomes |

---

## Demo Flow (60–90 seconds)

1. Show local Portaldot node running in terminal (blocks producing)
2. Open dashboard at `http://localhost:5173` — show protocol metrics
3. Run `python3 keeper/keeper.py` — show AI engine scanning live
4. Navigate to AI Terminal — show frontend mirroring engine output
5. Show Opportunity Feed — qualifying opportunity appears with composite score ≥ 70
6. Open Polkadot.js portal → Developer → Chain State → show OpportunityRegistry on-chain

---

## Team

| Name | Role |
|---|---|
| Chris | Lead Developer — Contracts, Backend, Frontend |
| [Teammate] | [Role] |

---

## License

MIT — see [LICENSE](LICENSE)

---

*Built for the Portaldot Hackathon 2026*