# Equilibria Protocol — Hackathon Submission Form

---

## 1. Basic Info

 **Team Name:** Equilibria Team
 **Project Name:** Equilibria Protocol
 **Track:** DeFi Infrastructure / Ecosystem Stability
 **Main Contact:** Chris (nofazechris)
 **Team Members:**
     Chris - Lead Developer
     Prince - Project Manager 
     Estylsmart - Ux 
     Navaar- Developer 
   

---

## 2. Product Summary

Equilibria is an autonomous market stabilization protocol built natively on Portaldot. It addresses a real problem in emerging blockchain ecosystems: DEX markets on new chains suffer from thin liquidity, wide price spreads, and no coordinated liquidity provision — making the ecosystem unstable and unattractive to serious capital.

Equilibria solves this by pooling reserve capital from depositors into a Stability Pool (ink! smart contract on Portaldot), then running an AI scoring engine that continuously monitors DEX price divergence across trading pairs. When the engine detects a qualifying opportunity — scoring it across profitability, risk, stability impact, and confidence — it posts the opportunity to an on-chain Opportunity Registry contract and deploys capital to narrow the spread. Profits return to the pool, generating yield for depositors.

The key innovation is not just arbitrage — it is ecosystem-aligned arbitrage where profits are socialized back to protocol depositors rather than extracted by a single actor, and where the execution logic is transparent, on-chain, and governed by enforced risk parameters (20% max deployment, circuit breaker, keeper authorization).

The protocol consists of two ink! smart contracts (StabilityPool + OpportunityRegistry), a Python AI scoring keeper, and a React dashboard providing real-time visibility into pool state, live opportunities, scoring engine output, and historical performance.

---

## 3. Current MVP Status

**What already works:**
1. `stability_pool` ink! v3 contract — compiled to WASM, deposit/withdraw/yield logic complete with enforced risk limits
2. `opportunity_registry` ink! v3 contract — deployed on Portaldot public dev node
3. Python AI keeper — scoring engine running, scans markets, scores opportunities with weighted composite formula
4. React frontend — 5 pages fully functional with live mock data (Dashboard, Opportunity Feed, Stability Pool, AI Terminal, Analytics)
5. Keeper→Registry integration — Bob (keeper wallet) authorized to post opportunities on-chain

**What is unfinished:**
1. `stability_pool` contract deployment — version compatibility issue between ink! v4/v5 and Portaldot's contracts pallet (v3 era); contract compiled, deployment in progress
2. Frontend live data connection — currently reading from mockData.js; polkadot.js API integration to read real contract state not yet wired
3. Keeper live execution loop — scoring runs, on-chain posting needs contract address finalized

**What is mocked (declared):**
1. Market price data — simulated DEX price feeds with realistic spread ranges (0.2%–3.5%); no live DEX deployed on Portaldot testnet to read from
2. Pool TVL and yield figures — frontend displays representative mock numbers; real figures will come from contract storage reads once deployed

---

## 4. Local Portaldot Status

| Question | Status 

 Can you run a local Portaldot node? | Yes |
 Is your project connected to local node? | Yes (also connected to public dev node) |
 Can contract / onchain logic be called? | In Progress |

**Public dev node (community-provided):**
```
wss://drip-backend-production-8d86.up.railway.app/node
```

**Evidence:**
- **Local node log:** Node runs with `./portaldot_dev --dev --alice --tmp`, confirmed at block #1000+
- **Contract deploy log:** `opportunity_registry` deployed. `stability_pool` compiled (21.4K WASM), deployment blocked by pallet version mismatch — resolving
- **Code path:** `equilibria/contracts/stability_pool/` and `equilibria/contracts/opportunity_registry/`
- **README:** See `README.md` and `EQUILIBRIA_PROJECT_SUMMARY.md` in repo root
- **Python connection confirmed:**
  ```
  Connected to: Development
  Block: 20255
  Keeper: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
  ```

---

## 5. POT Gas / Fee Status

| Question | Status |
|---|---|
| Can you show POT gas / fee usage? | Yes |

**Which action consumes POT:**
- `contracts.instantiateWithCode` — deploying StabilityPool contract
- `contracts.call` — keeper posting opportunities to OpportunityRegistry
- `contracts.call` — user depositing POT into StabilityPool

**How will you show it:**
- Live transaction on Polkadot.js portal showing Alice's POT balance decreasing after contract call
- Python keeper logs showing extrinsic hash + fee deduction per opportunity post

**Evidence:**
- Alice pre-funded: 4.9994 MUNIT on dev node
- Payment info confirmed via Python SDK:
  ```
  partialFee: 217881986186860 (units)
  weight: 503171301000
  ```

---

## 6. One Core Demo Flow

*Runnable in 60–90 seconds:*

1. **Show the dashboard** — open `http://localhost:5173`, demonstrate live pool metrics (TVL, deployed capital, yield, active opportunities)
2. **Start the AI keeper** — run `python3 keeper/keeper.py` in terminal, show live scoring output across POT pairs
3. **Navigate to AI Terminal** — show the frontend mirroring the keeper's reasoning in real time (scan → detect → score → approve/reject)
4. **Show Opportunity Feed** — demonstrate a qualifying opportunity (composite ≥ 70) appearing with all four scores
5. **Show on-chain proof** — open Polkadot.js portal, navigate to `Developer → Chain State → Contracts → ContractInfoOf`, show OpportunityRegistry contract address is live on chain

**What could fail in this demo:**
1. Public dev node goes down — mitigation: local node as backup (`./portaldot_dev --dev --alice --tmp`)
2. Frontend hot reload delays — mitigation: run `npm run build` beforehand and serve static

---

## 7. Open Source & Reproducibility

- **GitHub:** https://github.com/nofazechris/Equilibria-Protocol
- **Key contract folder:** `contracts/stability_pool/` and `contracts/opportunity_registry/`
- **Does README explain how to run locally:** Yes (see README.md)

**Quick start:**
```bash
# Frontend
cd frontend && npm install && npm run dev

# AI Keeper
cd keeper && pip3 install substrate-interface && python3 keeper.py

# Smart contracts
cd contracts/stability_pool && cargo contract build
```

---

## 8. Demo Video Plan


---

*Submitted by: Chris | github.com/nofazechris*