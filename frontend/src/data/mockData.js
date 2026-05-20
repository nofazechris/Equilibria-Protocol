export const poolState = {
  totalDeposited: 4280000,
  totalShares: 4194400,
  deployedCapital: 620000,
  yieldAccumulated: 134200,
  paused: false,
  utilizationRate: 14.5,
  apr: 8.7,
  depositors: 142,
}

export const opportunities = [
  { id: 1, pair: 'POT/rUSD', profitScore: 87, riskScore: 23, stabilityScore: 91, confidenceScore: 78, spread: 2.34, status: 'active', time: '2s ago', size: 18600 },
  { id: 2, pair: 'POT/USDC', profitScore: 72, riskScore: 41, stabilityScore: 65, confidenceScore: 83, spread: 1.87, status: 'pending', time: '8s ago', size: 12300 },
  { id: 3, pair: 'rUSD/POT', profitScore: 45, riskScore: 18, stabilityScore: 88, confidenceScore: 91, spread: 0.43, status: 'executed', time: '34s ago', size: 36150 },
  { id: 4, pair: 'POT/rUSD', profitScore: 93, riskScore: 67, stabilityScore: 44, confidenceScore: 61, spread: 3.12, status: 'rejected', time: '1m ago', size: 8400 },
  { id: 5, pair: 'POT/USDC', profitScore: 81, riskScore: 29, stabilityScore: 76, confidenceScore: 88, spread: 1.95, status: 'active', time: '2m ago', size: 28050 },
  { id: 6, pair: 'rUSD/POT', profitScore: 63, riskScore: 35, stabilityScore: 82, confidenceScore: 74, spread: 0.91, status: 'pending', size: 13950 },
]

export const chartData = [
  { time: '00:00', tvl: 3150000, yield: 63000, spread: 2.1 },
  { time: '04:00', tvl: 3375000, yield: 76500, spread: 1.8 },
  { time: '08:00', tvl: 3600000, yield: 87000, spread: 2.4 },
  { time: '12:00', tvl: 3975000, yield: 106500, spread: 1.6 },
  { time: '16:00', tvl: 4125000, yield: 123000, spread: 2.0 },
  { time: '20:00', tvl: 4280000, yield: 134200, spread: 1.9 },
]

export const aiLogs = [
  { time: '14:32:01', type: 'SCAN', message: 'Market scan initiated — monitoring 6 active POT pairs', score: null },
  { time: '14:32:03', type: 'DETECT', message: 'Price divergence detected: POT/rUSD spread 2.34% across Pool A/B', score: 87 },
  { time: '14:32:04', type: 'SCORE', message: 'Profitability: 87 | Risk: 23 | Stability Impact: 91 | Confidence: 78', score: null },
  { time: '14:32:05', type: 'APPROVE', message: 'Composite score 84.2 exceeds threshold 70. Posting to registry.', score: 84 },
  { time: '14:32:08', type: 'EXECUTE', message: 'Capital deployment confirmed: 18,600 POT → Pool arbitrage route', score: null },
  { time: '14:32:41', type: 'RETURN', message: 'Position closed. P&L: +431 POT | Spread normalized to 0.12%', score: null },
  { time: '14:33:01', type: 'SCAN', message: 'Market scan initiated — monitoring 6 active POT pairs', score: null },
  { time: '14:33:06', type: 'DETECT', message: 'Price divergence detected: POT/USDC spread 1.87%', score: 72 },
  { time: '14:33:07', type: 'SCORE', message: 'Profitability: 72 | Risk: 41 | Stability Impact: 65 | Confidence: 83', score: null },
  { time: '14:33:08', type: 'PENDING', message: 'Score 71.8 — borderline. Awaiting next scan confirmation.', score: 72 },
]

export const recentDeployments = [
  { id: 1, pair: 'POT/rUSD', amount: 18600, profit: 431, status: 'success', duration: '40s' },
  { id: 2, pair: 'rUSD/POT', amount: 36150, profit: 782, status: 'success', duration: '1m 12s' },
  { id: 3, pair: 'POT/USDC', amount: 8400, profit: -134, status: 'loss', duration: '28s' },
  { id: 4, pair: 'POT/rUSD', amount: 28050, profit: 618, status: 'success', duration: '55s' },
]