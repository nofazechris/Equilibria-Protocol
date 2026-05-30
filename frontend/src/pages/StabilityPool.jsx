import { poolState, chartData } from '../data/mockData'
import { AreaChart, Area, XAxis, YAxis, Tooltip, ResponsiveContainer } from 'recharts'
import { useState } from 'react'
import '../styles/pool.css'

function TxRow({ label, value, valueColor }) {
  return (
    <div className="tx-row">
      <span className="tx-row-label">{label}</span>
      <span style={{ color: valueColor || 'var(--text-primary)' }}>{value}</span>
    </div>
  )
}

export default function StabilityPool() {
  const [amount, setAmount] = useState('')
  const [tab, setTab]       = useState('deposit')

  const yieldEstimate = amount ? (parseFloat(amount) * 0.087 / 12).toFixed(2) : '—'
  const potReceived = amount ? (parseFloat(amount) * 1.02).toFixed(2) : '—'

  return (
    <div className="pool-wrapper">

      <div className="pool-header">
        <div className="page-title">Stability Pool</div>
        <div className="page-subtitle">Deposit POT to provide reserve liquidity. Earn yield from protocol stabilization operations.</div>
      </div>

      {/* Metrics */}
      <div className="pool-metric-strip">
        {[
          { label: 'POOL TVL',     value: `${(poolState.totalDeposited / 1000000).toFixed(3)}M POT`, color: 'var(--text-primary)' },
          { label: 'YOUR DEPOSIT', value: '0 POT',    color: 'var(--text-primary)' },
          { label: 'CURRENT APR',  value: `${poolState.apr}%`, color: 'var(--green)' },
          { label: 'YOUR YIELD',   value: '0 POT',    color: 'var(--text-primary)' },
        ].map(({ label, value, color }) => (
          <div key={label} className="pool-metric-cell">
            <div className="pool-metric-label">{label}</div>
            <div className="pool-metric-value" style={{ color }}>{value}</div>
          </div>
        ))}
      </div>

      {/* Yield chart */}
      <div className="yield-chart-panel">
        <div className="yield-chart-label">YIELD ACCUMULATION — 24H (POT)</div>
        <ResponsiveContainer width="100%" height={140}>
          <AreaChart data={chartData} margin={{ top: 0, right: 0, left: 0, bottom: 0 }}>
            <defs>
              <linearGradient id="yieldGrad" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%"  stopColor="#00c896" stopOpacity={0.2} />
                <stop offset="95%" stopColor="#00c896" stopOpacity={0} />
              </linearGradient>
            </defs>
            <XAxis dataKey="time" stroke="#3d4f63" tick={{ fontSize: 10, fill: '#3d4f63' }} axisLine={false} tickLine={false} />
            <YAxis stroke="#3d4f63" tick={{ fontSize: 10, fill: '#3d4f63' }} axisLine={false} tickLine={false} tickFormatter={v => `${(v / 1000).toFixed(0)}K`} />
            <Tooltip formatter={v => [`${v.toLocaleString()} POT`, 'Yield']} />
            <Area type="monotone" dataKey="yield" stroke="#00c896" fill="url(#yieldGrad)" strokeWidth={1.5} dot={false} />
          </AreaChart>
        </ResponsiveContainer>
      </div>

      {/* Deposit / Withdraw */}
      <div className="action-panel">

        <div className="action-tab-bar">
          {['deposit', 'withdraw'].map(t => (
            <button
              key={t}
              onClick={() => setTab(t)}
              className={`action-tab ${tab === t ? 'active' : ''}`}
            >
              {t.toUpperCase()}
            </button>
          ))}
        </div>

        <div style={{ fontSize: '11px', color: '#3d4f63', marginBottom: '8px', letterSpacing: '0.06em' }}>
            {tab === 'deposit' ? 'AMOUNT TO DEPOSIT (POT)' : 'AMOUNT TO WITHDRAW (POT)'}
        </div>

        <div className="action-input-row">
          <input
            className="action-input"
            value={amount}
            onChange={e => setAmount(e.target.value)}
            placeholder="0"
          />
          <button className="action-submit-btn">
            {tab === 'deposit' ? 'DEPOSIT' : 'WITHDRAW'}
          </button>
        </div>

        <div className="tx-preview">
          <div className="tx-preview-label">TRANSACTION PREVIEW</div>
          {tab === 'deposit' ? (
            <>
              <TxRow label="You deposit"   value={amount ? `${parseFloat(amount).toLocaleString()} POT` : '—'} />
              <TxRow label="Pool share"    value={`${((parseFloat(amount || 0) / 4280000) * 100).toFixed(4)}%`} />
              <TxRow label="Yield rate"    value="8.7% APR" />
            </>
          ) : (
            <>
              <TxRow label="You withdraw"  value={amount ? `${parseFloat(amount).toLocaleString()} POT` : '—'} />
              <TxRow label="Yield earned"  value={`+${potReceived} POT`} />
              <TxRow label="Return"        value="~8.7% APR" />
            </>
          )}
          <TxRow label="Pool utilization"    value={`${poolState.utilizationRate}%`} valueColor="var(--yellow)" />
          <TxRow label="Max deployment limit" value="20% of pool" />
        </div>
      </div>
    </div>
  )
}