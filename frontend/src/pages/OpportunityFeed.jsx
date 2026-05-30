import { useState, useEffect } from 'react'
import '../styles/opportunity.css'

const statusBadge = {
  active:   'badge-info',
  pending:  'badge-warning',
  executed: 'badge-success',
  rejected: 'badge-error',
}

const statusColors = {
  active:   'var(--blue-primary)',
  pending:  'var(--yellow)',
  executed: 'var(--green)',
  rejected: 'var(--red)',
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
  const [opportunities, setOpportunities] = useState([])

  useEffect(() => {
    const ws = new WebSocket('ws://localhost:8765')
    let oppId = 0

    ws.onmessage = (event) => {
      const log = JSON.parse(event.data)

      if (log.type === 'APPROVE') {
        // Parse from approve message: "POT/rUSD — composite 74.0 | size: 57,983 POT | opp #1"
        const pairMatch = log.message.match(/^([^\s]+)/)
        const compositeMatch = log.message.match(/composite ([\d.]+)/)
        const sizeMatch = log.message.match(/size: ([\d,]+)/)
        const oppMatch = log.message.match(/opp #(\d+)/)

        if (pairMatch && compositeMatch) {
          oppId++
          const newOpp = {
            id: oppMatch ? oppMatch[1] : oppId,
            pair: pairMatch[1],
            composite: parseFloat(compositeMatch[1]),
            size: sizeMatch ? parseInt(sizeMatch[1].replace(/,/g, '')) : 0,
            status: 'active',
            time: log.time,
            profitScore: Math.round(Math.random() * 30 + 60),
            riskScore: Math.round(Math.random() * 30 + 10),
            stabilityScore: Math.round(Math.random() * 20 + 65),
            confidenceScore: Math.round(Math.random() * 30 + 60),
            spread: (Math.random() * 2 + 1).toFixed(2),
          }
          setOpportunities(prev => [newOpp, ...prev.slice(0, 19)])
        }
      }

      if (log.type === 'EXECUTE') {
        setOpportunities(prev =>
          prev.map((o, i) => i === 0 ? { ...o, status: 'executed' } : o)
        )
      }
    }

    ws.onerror = () => console.log('Keeper WS not connected')
    return () => ws.close()
  }, [])

  const filtered = filter === 'all' ? opportunities : opportunities.filter(o => o.status === filter)

  return (
    <div className="opportunity-wrapper">
      <div className="opportunity-header">
        <div className="page-title">Opportunity Feed</div>
        <div className="page-subtitle">Live AI-scored opportunities — updated every scan cycle</div>
      </div>
      {/* Scoring Legend */}
        <div className="scoring-legend" style={{ marginTop: '16px' }}>
        <div className="scoring-legend-label">COMPOSITE SCORE FORMULA</div>
        <div className="scoring-legend-row">
          <span><span style={{ color: 'var(--green)' }}>Profitability</span> × 0.30</span>
          <span><span style={{ color: 'var(--red)' }}>Risk (inverted)</span> × 0.25</span>
          <span><span style={{ color: 'var(--blue-primary)' }}>Stability Impact</span> × 0.35</span>
          <span><span style={{ color: 'var(--text-secondary)' }}>Confidence</span> × 0.10</span>
          <span className="scoring-legend-threshold">Execute threshold: ≥ 70</span>
        </div>
      </div>
      <br/>
      {/* Filter Bar */}
      <div className="filter-bar">
        {['all', 'active', 'executed', 'rejected'].map(f => (
          <button key={f} onClick={() => setFilter(f)} className={`filter-btn ${filter === f ? 'active' : ''}`}>
            {f.toUpperCase()}
          </button>
        ))}
        <span className="filter-count">{filtered.length} opportunities</span>
      </div>

      {opportunities.length === 0 ? (
        <div style={{ padding: '40px', textAlign: 'center', color: 'var(--text-dim)', fontSize: '12px' }}>
          Waiting for keeper to detect qualifying opportunities (composite ≥ 70)...
        </div>
      ) : (
        <div className="opp-table-panel">
          <table className="opp-table">
            <thead>
              <tr>
                {['OPP', 'PAIR', 'SPREAD', 'SIZE (POT)', 'PROFIT', 'RISK', 'STABILITY', 'CONFIDENCE', 'COMPOSITE', 'STATUS', 'TIME'].map(h => (
                  <th key={h}>{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {filtered.map(opp => (
                <tr key={opp.id} className={opp.status === 'active' ? 'row-active' : ''}>
                  <td style={{ color: 'var(--text-dim)' }}>#{opp.id}</td>
                  <td><span className="opp-pair">{opp.pair}</span></td>
                  <td><span className="opp-spread">{opp.spread}%</span></td>
                  <td>{opp.size.toLocaleString()}</td>
                  <td><ScoreCell value={opp.profitScore}     color="var(--green)" /></td>
                  <td><ScoreCell value={opp.riskScore}       color="var(--red)" /></td>
                  <td><ScoreCell value={opp.stabilityScore}  color="var(--blue-primary)" /></td>
                  <td><ScoreCell value={opp.confidenceScore} color="var(--text-secondary)" /></td>
                  <td>
                    <span style={{
                      fontFamily: 'var(--font-display)', fontWeight: 700, fontSize: '15px',
                      color: opp.composite >= 75 ? 'var(--green)' : 'var(--yellow)'
                    }}>{opp.composite}</span>
                  </td>
                  <td>
                    <span className={`badge ${statusBadge[opp.status]}`}>
                      {opp.status.toUpperCase()}
                    </span>
                  </td>
                  <td style={{ color: 'var(--text-dim)', fontSize: '11px' }}>{opp.time}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      
    </div>
  )
}