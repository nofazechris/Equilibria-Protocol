import { poolState, chartData } from '../data/mockData'
import { AreaChart, Area, XAxis, YAxis, Tooltip, ResponsiveContainer } from 'recharts'
import { useState } from 'react'

export default function StabilityPool() {
  const [amount, setAmount] = useState('')
  const [tab, setTab] = useState('deposit')

  const sharesReceived = amount ? (parseFloat(amount) * 0.98).toFixed(2) : '—'
  const potReceived = amount ? (parseFloat(amount) * 1.02).toFixed(2) : '—'

  return (
    <div style={{ padding: '32px', maxWidth: '900px' }}>

      <div style={{ marginBottom: '32px', borderBottom: '1px solid #1e2530', paddingBottom: '20px' }}>
        <div style={{ fontFamily: 'var(--font-display)', fontSize: '20px', fontWeight: 700, marginBottom: '4px' }}>Stability Pool</div>
        <div style={{ color: '#7a8a9e', fontSize: '12px' }}>Deposit POT to provide reserve liquidity. Earn yield from protocol stabilization operations.</div>
      </div>

      {/* Pool metrics — plain grid */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '1px', background: '#1e2530', border: '1px solid #1e2530', borderRadius: '6px', overflow: 'hidden', marginBottom: '24px' }}>
        {[
          { label: 'POOL TVL', value: `${(poolState.totalDeposited/1000000).toFixed(3)}M POT` },
          { label: 'YOUR SHARES', value: '0 EQB' },
          { label: 'CURRENT APR', value: `${poolState.apr}%`, color: '#00c896' },
          { label: 'YOUR YIELD', value: '0 POT' },
        ].map(({ label, value, color }) => (
          <div key={label} style={{ background: '#13161a', padding: '18px 20px' }}>
            <div style={{ fontSize: '10px', color: '#3d4f63', letterSpacing: '0.1em', marginBottom: '8px' }}>{label}</div>
            <div style={{ fontSize: '18px', fontFamily: 'var(--font-display)', fontWeight: 700, color: color || '#e8edf5' }}>{value}</div>
          </div>
        ))}
      </div>

      {/* Yield chart */}
      <div style={{ background: '#13161a', border: '1px solid #1e2530', borderRadius: '6px', padding: '20px', marginBottom: '20px' }}>
        <div style={{ fontSize: '10px', color: '#3d4f63', letterSpacing: '0.1em', marginBottom: '16px' }}>YIELD ACCUMULATION — 24H (POT)</div>
        <ResponsiveContainer width="100%" height={140}>
          <AreaChart data={chartData} margin={{ top: 0, right: 0, left: 0, bottom: 0 }}>
            <defs>
              <linearGradient id="yieldGrad" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="#00c896" stopOpacity={0.2} />
                <stop offset="95%" stopColor="#00c896" stopOpacity={0} />
              </linearGradient>
            </defs>
            <XAxis dataKey="time" stroke="#3d4f63" tick={{ fontSize: 10, fill: '#3d4f63' }} axisLine={false} tickLine={false} />
            <YAxis stroke="#3d4f63" tick={{ fontSize: 10, fill: '#3d4f63' }} axisLine={false} tickLine={false} tickFormatter={v => `${(v/1000).toFixed(0)}K`} />
            <Tooltip formatter={(v) => [`${v.toLocaleString()} POT`, 'Yield']} />
            <Area type="monotone" dataKey="yield" stroke="#00c896" fill="url(#yieldGrad)" strokeWidth={1.5} dot={false} />
          </AreaChart>
        </ResponsiveContainer>
      </div>

      {/* Deposit / Withdraw */}
      <div style={{ background: '#13161a', border: '1px solid #1e2530', borderRadius: '6px', padding: '24px' }}>
        {/* Tabs */}
        <div style={{ display: 'flex', borderBottom: '1px solid #1e2530', marginBottom: '20px' }}>
          {['deposit', 'withdraw'].map(t => (
            <button key={t} onClick={() => setTab(t)} style={{
              padding: '8px 20px', border: 'none', background: 'transparent',
              color: tab === t ? '#1a6cf6' : '#7a8a9e',
              cursor: 'pointer', fontFamily: 'var(--font-mono)', fontSize: '11px', letterSpacing: '0.06em',
              borderBottom: tab === t ? '1px solid #1a6cf6' : '1px solid transparent',
              marginBottom: '-1px',
            }}>{t.toUpperCase()}</button>
          ))}
        </div>

        {/* Input */}
        <div style={{ marginBottom: '16px' }}>
          <div style={{ fontSize: '11px', color: '#3d4f63', marginBottom: '8px', letterSpacing: '0.06em' }}>
            {tab === 'deposit' ? 'AMOUNT TO DEPOSIT (POT)' : 'SHARES TO REDEEM (EQB)'}
          </div>
          <div style={{ display: 'flex', gap: '10px' }}>
            <input
              value={amount}
              onChange={e => setAmount(e.target.value)}
              placeholder="0"
              style={{
                flex: 1, background: '#0f1114', border: '1px solid #1e2530',
                borderRadius: '4px', padding: '11px 14px', color: '#e8edf5',
                fontFamily: 'var(--font-mono)', fontSize: '15px', outline: 'none',
              }}
            />
            <button style={{
              padding: '11px 22px', background: '#1a6cf6', border: 'none',
              borderRadius: '4px', color: '#fff', cursor: 'pointer',
              fontFamily: 'var(--font-mono)', fontSize: '11px', letterSpacing: '0.06em',
            }}>
              {tab === 'deposit' ? 'DEPOSIT' : 'WITHDRAW'}
            </button>
          </div>
        </div>

        {/* Transaction preview */}
        <div style={{ padding: '12px 14px', background: '#0f1114', borderRadius: '4px', border: '1px solid #1e2530' }}>
          <div style={{ fontSize: '10px', color: '#3d4f63', letterSpacing: '0.08em', marginBottom: '8px' }}>TRANSACTION PREVIEW</div>
          {tab === 'deposit' ? (
            <>
              <Row label="You deposit" value={amount ? `${parseFloat(amount).toLocaleString()} POT` : '—'} />
              <Row label="You receive" value={`${sharesReceived} EQB`} />
              <Row label="Exchange rate" value="1 POT = 0.980 EQB" />
            </>
          ) : (
            <>
              <Row label="You redeem" value={amount ? `${parseFloat(amount).toLocaleString()} EQB` : '—'} />
              <Row label="You receive" value={`${potReceived} POT`} />
              <Row label="Exchange rate" value="1 EQB = 1.020 POT" />
            </>
          )}
          <Row label="Pool utilization" value={`${poolState.utilizationRate}%`} valueColor="#f4a621" />
          <Row label="Max deployment limit" value="20% of pool" />
        </div>
      </div>
    </div>
  )
}

function Row({ label, value, valueColor }) {
  return (
    <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '11px', marginBottom: '5px' }}>
      <span style={{ color: '#7a8a9e' }}>{label}</span>
      <span style={{ color: valueColor || '#e8edf5' }}>{value}</span>
    </div>
  )
}