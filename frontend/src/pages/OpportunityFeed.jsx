import { opportunities } from '../data/mockData'
import { useState } from 'react'
import '../styles/opportunity.css'

const statusColors = {
  active:   'var(--blue-primary)',
  pending:  'var(--yellow)',
  executed: 'var(--green)',
  rejected: 'var(--red)',
}

const statusBadge = {
  active:   'badge-info',
  pending:  'badge-warning',
  executed: 'badge-success',
  rejected: 'badge-error',
}

function ScoreCell({ value, color }) {
  return (
    <div className="score-cell">
      <div className="score-bar-track">
        <div className="score-bar-fill" style={{ width: `${value}%`, background: color }} />
      </div>
      <span className="score-value" style={{ color }}>{value}</span>
    </div>
  )
}

export default function OpportunityFeed() {
  const [filter, setFilter] = useState('all')
  const filtered = filter === 'all' ? opportunities : opportunities.filter(o => o.status === filter)

  return (
    <div className="opportunity-wrapper">

      <div className="opportunity-header">
        <div className="page-title">Opportunity Feed</div>
        <div className="page-subtitle">AI-scored market opportunities — ranked by composite stability-adjusted return</div>
      </div>

      {/* Filters */}
      <div className="filter-bar">
        {['all', 'active', 'pending', 'executed', 'rejected'].map(f => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={`filter-btn ${filter === f ? 'active' : ''}`}
          >
            {f.toUpperCase()}
          </button>
        ))}
        <span className="filter-count">{filtered.length} opportunities</span>
      </div>

      {/* Table */}
      <div className="opp-table-panel">
        <table className="opp-table">
          <thead>
            <tr>
              {['PAIR', 'SPREAD', 'SIZE (POT)', 'PROFIT', 'RISK', 'STABILITY', 'CONFIDENCE', 'COMPOSITE', 'STATUS', 'AGE'].map(h => (
                <th key={h}>{h}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {filtered.map(opp => {
              const composite = Math.round(
                opp.profitScore    * 0.30 +
                (100 - opp.riskScore) * 0.25 +
                opp.stabilityScore * 0.35 +
                opp.confidenceScore * 0.10
              )
              return (
                <tr key={opp.id} className={opp.status === 'active' ? 'row-active' : ''}>
                  <td><span className="opp-pair">{opp.pair}</span></td>
                  <td><span className="opp-spread">{opp.spread}%</span></td>
                  <td>{opp.size.toLocaleString()}</td>
                  <td><ScoreCell value={opp.profitScore}    color="var(--green)" /></td>
                  <td><ScoreCell value={opp.riskScore}      color="var(--red)" /></td>
                  <td><ScoreCell value={opp.stabilityScore} color="var(--blue-primary)" /></td>
                  <td><ScoreCell value={opp.confidenceScore} color="var(--text-secondary)" /></td>
                  <td>
                    <span className="opp-composite" style={{
                      color: composite >= 75 ? 'var(--green)' : composite >= 55 ? 'var(--yellow)' : 'var(--red)'
                    }}>{composite}</span>
                  </td>
                  <td>
                    <span className={`badge ${statusBadge[opp.status]}`}>
                      {opp.status.toUpperCase()}
                    </span>
                  </td>
                  <td style={{ color: 'var(--text-dim)', fontSize: '11px' }}>{opp.time}</td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>

      {/* Scoring legend */}
      <div className="scoring-legend">
        <div className="scoring-legend-label">COMPOSITE SCORE FORMULA</div>
        <div className="scoring-legend-row">
          <span><span style={{ color: 'var(--green)' }}>Profitability</span> × 0.30</span>
          <span><span style={{ color: 'var(--red)' }}>Risk (inverted)</span> × 0.25</span>
          <span><span style={{ color: 'var(--blue-primary)' }}>Stability Impact</span> × 0.35</span>
          <span><span style={{ color: 'var(--text-secondary)' }}>Confidence</span> × 0.10</span>
          <span className="scoring-legend-threshold">Execute threshold: ≥ 70</span>
        </div>
      </div>
    </div>
  )
}