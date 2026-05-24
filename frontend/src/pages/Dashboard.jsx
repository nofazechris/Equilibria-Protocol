import { poolState, chartData, recentDeployments, opportunities } from '../data/mockData'
import { AreaChart, Area, XAxis, YAxis, Tooltip, ResponsiveContainer } from 'recharts'
import '../styles/dashboard.css'

const CustomTooltip = ({ active, payload, label }) => {
  if (!active || !payload?.length) return null
  return (
    <div className="card" style={{ padding: '8px 12px', fontSize: '11px' }}>
      <div style={{ color: 'var(--text-dim)', marginBottom: '3px' }}>{label}</div>
      {payload.map((p, i) => (
        <div key={i} style={{ color: 'var(--text-primary)' }}>
          {p.name}: {p.value > 1000 ? `${(p.value / 1000000).toFixed(2)}M POT` : `${p.value} POT`}
        </div>
      ))}
    </div>
  )
}

export default function Dashboard() {
  const activeOpps = opportunities.filter(o => o.status === 'active').length

  return (
    <div className="dashboard-wrapper">

      {/* Header */}
      <div className="dashboard-header">
        <div className="dashboard-header-row">
          <span className="page-title">Protocol Overview</span>
          <span className="badge badge-live">LIVE</span>
        </div>
        <div className="page-subtitle">Equilibria — autonomous market stabilization on Portaldot</div>
      </div>

      {/* Metric strip */}
      <div className="metric-strip">
        {[
          { label: 'TOTAL VALUE LOCKED',   value: `${(poolState.totalDeposited / 1000000).toFixed(2)}M POT`, sub: `${poolState.depositors} depositors` },
          { label: 'DEPLOYED CAPITAL',     value: `${(poolState.deployedCapital / 1000).toFixed(0)}K POT`,  sub: `${poolState.utilizationRate}% utilization` },
          { label: 'YIELD ACCUMULATED',    value: `${(poolState.yieldAccumulated / 1000).toFixed(1)}K POT`, sub: `${poolState.apr}% APR` },
          { label: 'ACTIVE OPPORTUNITIES', value: activeOpps, sub: 'of 6 scanned pairs' },
        ].map(({ label, value, sub }) => (
          <div key={label} className="metric-cell">
            <div className="metric-cell-label">{label}</div>
            <div className="metric-cell-value">{value}</div>
            <div className="metric-cell-sub">{sub}</div>
          </div>
        ))}
      </div>

      {/* Charts row */}
      <div className="charts-row">

        {/* TVL chart */}
        <div className="chart-panel">
          <div className="chart-panel-label">TOTAL VALUE LOCKED — 24H</div>
          <ResponsiveContainer width="100%" height={180}>
            <AreaChart data={chartData} margin={{ top: 0, right: 0, left: 0, bottom: 0 }}>
              <defs>
                <linearGradient id="tvlGrad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%"  stopColor="#1a6cf6" stopOpacity={0.2} />
                  <stop offset="95%" stopColor="#1a6cf6" stopOpacity={0} />
                </linearGradient>
              </defs>
              <XAxis dataKey="time" stroke="#3d4f63" tick={{ fontSize: 10, fill: '#3d4f63' }} axisLine={false} tickLine={false} />
              <YAxis stroke="#3d4f63" tick={{ fontSize: 10, fill: '#3d4f63' }} axisLine={false} tickLine={false} tickFormatter={v => `${(v / 1000000).toFixed(1)}M`} />
              <Tooltip content={<CustomTooltip />} />
              <Area type="monotone" dataKey="tvl" name="TVL" stroke="#1a6cf6" fill="url(#tvlGrad)" strokeWidth={1.5} dot={false} />
            </AreaChart>
          </ResponsiveContainer>
        </div>

        {/* Pool status */}
        <div className="pool-status-panel">
          <div className="pool-status-label">POOL STATUS</div>

          {[
            { label: 'Idle Reserve', value: 85.5, color: 'var(--blue-primary)' },
            { label: 'Deployed',     value: 14.5, color: 'var(--yellow)' },
          ].map(({ label, value, color }) => (
            <div key={label} className="allocation-row">
              <div className="allocation-row-header">
                <span className="allocation-row-label">{label}</span>
                <span style={{ color }}>{value}%</span>
              </div>
              <div className="progress-track">
                <div className="progress-fill" style={{ width: `${value}%`, background: color }} />
              </div>
            </div>
          ))}

          <div className="risk-controls">
            <div className="risk-controls-label">RISK CONTROLS</div>
            {[
              { label: 'Max deployment', value: '20% of pool',   color: null },
              { label: 'Circuit breaker', value: '● ARMED',      color: 'var(--green)' },
              { label: 'Pool status',    value: '● ACTIVE',      color: 'var(--green)' },
            ].map(({ label, value, color }) => (
              <div key={label} className="risk-row">
                <span className="risk-row-label">{label}</span>
                <span style={{ color: color || 'var(--text-primary)' }}>{value}</span>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Recent deployments */}
      <div className="deployments-panel">
        <div className="deployments-panel-label">RECENT DEPLOYMENTS</div>
        <table className="deploy-table">
          <thead>
            <tr>
              {['ID', 'PAIR', 'DEPLOYED (POT)', 'P&L (POT)', 'DURATION', 'STATUS'].map(h => (
                <th key={h}>{h}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {recentDeployments.map(d => (
              <tr key={d.id}>
                <td className="id-col">#{d.id}</td>
                <td className="pair-col">{d.pair}</td>
                <td>{d.amount.toLocaleString()}</td>
                <td style={{ color: d.profit > 0 ? 'var(--green)' : 'var(--red)' }}>
                  {d.profit > 0 ? '+' : ''}{d.profit}
                </td>
                <td style={{ color: 'var(--text-secondary)' }}>{d.duration}</td>
                <td>
                  <span className={`badge ${d.status === 'success' ? 'badge-success' : 'badge-error'}`}>
                    {d.status.toUpperCase()}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}