import { opportunities } from '../data/mockData'
import { useState } from 'react'

const statusColors = {
  active: '#1a6cf6',
  pending: '#f4a621',
  executed: '#00c896',
  rejected: '#e63946',
}

export default function OpportunityFeed() {
  const [filter, setFilter] = useState('all')
  const filtered = filter === 'all' ? opportunities : opportunities.filter(o => o.status === filter)

  return (
    <div style={{ padding: '32px', maxWidth: '1300px' }}>

      <div style={{ marginBottom: '32px', borderBottom: '1px solid #1e2530', paddingBottom: '20px' }}>
        <div style={{ fontFamily: 'var(--font-display)', fontSize: '20px', fontWeight: 700, marginBottom: '4px' }}>Opportunity Feed</div>
        <div style={{ color: '#7a8a9e', fontSize: '12px' }}>AI-scored market opportunities — ranked by composite stability-adjusted return</div>
      </div>

      {/* Filter row */}
      <div style={{ display: 'flex', gap: '6px', marginBottom: '20px' }}>
        {['all', 'active', 'pending', 'executed', 'rejected'].map(f => (
          <button key={f} onClick={() => setFilter(f)} style={{
            padding: '5px 12px', borderRadius: '3px', border: '1px solid',
            borderColor: filter === f ? '#1a6cf6' : '#1e2530',
            background: filter === f ? '#0d2d6e' : 'transparent',
            color: filter === f ? '#1a6cf6' : '#7a8a9e',
            cursor: 'pointer', fontSize: '11px', fontFamily: 'var(--font-mono)',
            letterSpacing: '0.06em',
          }}>{f.toUpperCase()}</button>
        ))}
        <span style={{ marginLeft: 'auto', color: '#3d4f63', fontSize: '11px', alignSelf: 'center' }}>
          {filtered.length} opportunities
        </span>
      </div>

      {/* Table view — more useful than cards for comparison */}
      <div style={{ background: '#13161a', border: '1px solid #1e2530', borderRadius: '6px', overflow: 'hidden', marginBottom: '24px' }}>
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ borderBottom: '1px solid #1e2530' }}>
              {['PAIR', 'SPREAD', 'SIZE (POT)', 'PROFIT', 'RISK', 'STABILITY', 'CONFIDENCE', 'COMPOSITE', 'STATUS', 'AGE'].map(h => (
                <th key={h} style={{ textAlign: 'left', padding: '10px 14px', color: '#3d4f63', fontSize: '10px', letterSpacing: '0.08em', fontWeight: 400 }}>{h}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {filtered.map(opp => {
              const composite = Math.round(opp.profitScore * 0.3 + (100 - opp.riskScore) * 0.25 + opp.stabilityScore * 0.35 + opp.confidenceScore * 0.1)
              return (
                <tr key={opp.id} style={{ borderBottom: '1px solid #1e2530', background: opp.status === 'active' ? 'rgba(26,108,246,0.03)' : 'transparent' }}>
                  <td style={{ padding: '12px 14px', fontFamily: 'var(--font-display)', fontWeight: 600, fontSize: '13px' }}>{opp.pair}</td>
                  <td style={{ padding: '12px 14px', color: '#f4a621', fontSize: '12px' }}>{opp.spread}%</td>
                  <td style={{ padding: '12px 14px', fontSize: '12px' }}>{opp.size.toLocaleString()}</td>
                  <td style={{ padding: '12px 14px' }}>
                    <ScoreCell value={opp.profitScore} color="#00c896" />
                  </td>
                  <td style={{ padding: '12px 14px' }}>
                    <ScoreCell value={opp.riskScore} color="#e63946" invert />
                  </td>
                  <td style={{ padding: '12px 14px' }}>
                    <ScoreCell value={opp.stabilityScore} color="#1a6cf6" />
                  </td>
                  <td style={{ padding: '12px 14px' }}>
                    <ScoreCell value={opp.confidenceScore} color="#7a8a9e" />
                  </td>
                  <td style={{ padding: '12px 14px' }}>
                    <span style={{
                      fontFamily: 'var(--font-display)', fontWeight: 700, fontSize: '15px',
                      color: composite >= 75 ? '#00c896' : composite >= 55 ? '#f4a621' : '#e63946'
                    }}>{composite}</span>
                  </td>
                  <td style={{ padding: '12px 14px' }}>
                    <span style={{
                      fontSize: '10px', padding: '2px 8px', borderRadius: '3px', letterSpacing: '0.05em',
                      background: `${statusColors[opp.status]}15`,
                      color: statusColors[opp.status],
                    }}>{opp.status.toUpperCase()}</span>
                  </td>
                  <td style={{ padding: '12px 14px', color: '#3d4f63', fontSize: '11px' }}>{opp.time}</td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>

      {/* Scoring legend */}
      <div style={{ background: '#13161a', border: '1px solid #1e2530', borderRadius: '6px', padding: '16px 20px' }}>
        <div style={{ fontSize: '10px', color: '#3d4f63', letterSpacing: '0.08em', marginBottom: '10px' }}>COMPOSITE SCORE FORMULA</div>
        <div style={{ display: 'flex', gap: '32px', fontSize: '11px', color: '#7a8a9e' }}>
          <span><span style={{ color: '#00c896' }}>Profitability</span> × 0.30</span>
          <span><span style={{ color: '#e63946' }}>Risk (inverted)</span> × 0.25</span>
          <span><span style={{ color: '#1a6cf6' }}>Stability Impact</span> × 0.35</span>
          <span><span style={{ color: '#7a8a9e' }}>Confidence</span> × 0.10</span>
          <span style={{ marginLeft: 'auto', color: '#3d4f63' }}>Threshold to execute: 70</span>
        </div>
      </div>
    </div>
  )
}

function ScoreCell({ value, color, invert }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
      <div style={{ width: '40px', height: '3px', background: '#1e2530', borderRadius: '2px' }}>
        <div style={{ width: `${value}%`, height: '100%', background: color, borderRadius: '2px' }} />
      </div>
      <span style={{ fontSize: '11px', color: invert ? (value > 50 ? '#e63946' : '#7a8a9e') : color }}>{value}</span>
    </div>
  )
}