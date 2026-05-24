import { chartData, recentDeployments } from '../data/mockData'
import { LineChart, Line, BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid } from 'recharts'
import '../styles/analytics.css'

export default function Analytics() {
  const totalProfit   = recentDeployments.reduce((a, d) => a + d.profit, 0)
  const successCount  = recentDeployments.filter(d => d.status === 'success').length
  const successRate   = ((successCount / recentDeployments.length) * 100).toFixed(0)
  const totalDeployed = recentDeployments.reduce((a, d) => a + d.amount, 0)

  const tooltipStyle = { background: 'var(--bg-secondary)', border: '1px solid var(--border)', fontSize: '11px' }

  return (
    <div className="analytics-wrapper">

      <div className="analytics-header">
        <div className="page-title">Protocol Analytics</div>
        <div className="page-subtitle">Historical performance — deployment outcomes, yield generation, market impact</div>
      </div>

      {/* KPI strip */}
      <div className="kpi-strip">
        {[
          { label: 'TOTAL DEPLOYMENTS',    value: recentDeployments.length.toString(), color: null },
          { label: 'SUCCESS RATE',         value: `${successRate}%`, color: 'var(--green)' },
          { label: 'TOTAL P&L (POT)',      value: `${totalProfit > 0 ? '+' : ''}${totalProfit.toLocaleString()}`, color: totalProfit > 0 ? 'var(--green)' : 'var(--red)' },
          { label: 'CAPITAL DEPLOYED (POT)', value: totalDeployed.toLocaleString(), color: null },
          { label: 'AVG SPREAD REDUCTION', value: '1.8%', color: 'var(--blue-primary)' },
        ].map(({ label, value, color }) => (
          <div key={label} className="kpi-cell">
            <div className="kpi-cell-label">{label}</div>
            <div className="kpi-cell-value" style={{ color: color || 'var(--text-primary)' }}>{value}</div>
          </div>
        ))}
      </div>

      {/* Charts */}
      <div className="charts-grid">
        <div className="analytics-chart-panel">
          <div className="analytics-chart-label">SPREAD REDUCTION OVER TIME (%)</div>
          <ResponsiveContainer width="100%" height={180}>
            <LineChart data={chartData} margin={{ top: 0, right: 0, left: 0, bottom: 0 }}>
              <CartesianGrid strokeDasharray="2 4" stroke="var(--border)" />
              <XAxis dataKey="time" stroke="#3d4f63" tick={{ fontSize: 10, fill: '#3d4f63' }} axisLine={false} tickLine={false} />
              <YAxis stroke="#3d4f63" tick={{ fontSize: 10, fill: '#3d4f63' }} axisLine={false} tickLine={false} domain={[0, 3]} />
              <Tooltip formatter={v => [`${v}%`, 'Spread']} contentStyle={tooltipStyle} />
              <Line type="monotone" dataKey="spread" stroke="var(--blue-primary)" strokeWidth={1.5} dot={false} />
            </LineChart>
          </ResponsiveContainer>
        </div>

        <div className="analytics-chart-panel">
          <div className="analytics-chart-label">YIELD GENERATED PER PERIOD (POT)</div>
          <ResponsiveContainer width="100%" height={180}>
            <BarChart data={chartData} margin={{ top: 0, right: 0, left: 0, bottom: 0 }}>
              <CartesianGrid strokeDasharray="2 4" stroke="var(--border)" />
              <XAxis dataKey="time" stroke="#3d4f63" tick={{ fontSize: 10, fill: '#3d4f63' }} axisLine={false} tickLine={false} />
              <YAxis stroke="#3d4f63" tick={{ fontSize: 10, fill: '#3d4f63' }} axisLine={false} tickLine={false} tickFormatter={v => `${(v / 1000).toFixed(0)}K`} />
              <Tooltip formatter={v => [`${v.toLocaleString()} POT`, 'Yield']} contentStyle={tooltipStyle} />
              <Bar dataKey="yield" fill="var(--blue-primary)" radius={[2, 2, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Breakdown table */}
      <div className="breakdown-panel">
        <div className="breakdown-label">DEPLOYMENT BREAKDOWN</div>
        <table className="breakdown-table">
          <thead>
            <tr>
              {['ID', 'PAIR', 'DEPLOYED (POT)', 'P&L (POT)', 'RETURN %', 'DURATION', 'OUTCOME'].map(h => (
                <th key={h}>{h}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {recentDeployments.map(d => {
              const ret = ((d.profit / d.amount) * 100).toFixed(2)
              return (
                <tr key={d.id}>
                  <td className="breakdown-id">#{d.id}</td>
                  <td>{d.pair}</td>
                  <td>{d.amount.toLocaleString()}</td>
                  <td style={{ color: d.profit > 0 ? 'var(--green)' : 'var(--red)' }}>
                    {d.profit > 0 ? '+' : ''}{d.profit}
                  </td>
                  <td style={{ color: d.profit > 0 ? 'var(--green)' : 'var(--red)' }}>
                    {d.profit > 0 ? '+' : ''}{ret}%
                  </td>
                  <td style={{ color: 'var(--text-secondary)' }}>{d.duration}</td>
                  <td>
                    <span className={`badge ${d.status === 'success' ? 'badge-success' : 'badge-error'}`}>
                      {d.status.toUpperCase()}
                    </span>
                  </td>
                </tr>
              )
            })}
          </tbody>
          <tfoot>
            <tr>
              <td colSpan={2} className="breakdown-total">TOTAL</td>
              <td>{totalDeployed.toLocaleString()}</td>
              <td style={{ color: totalProfit > 0 ? 'var(--green)' : 'var(--red)' }}>
                {totalProfit > 0 ? '+' : ''}{totalProfit}
              </td>
              <td style={{ color: totalProfit > 0 ? 'var(--green)' : 'var(--red)' }}>
                {((totalProfit / totalDeployed) * 100).toFixed(2)}%
              </td>
              <td colSpan={2} />
            </tr>
          </tfoot>
        </table>
      </div>
    </div>
  )
}