import { chartData, recentDeployments, opportunities } from '../data/mockData'
import { LineChart, Line, BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid } from 'recharts'

export default function Analytics() {
  const totalProfit = recentDeployments.reduce((a, d) => a + d.profit, 0)
  const successCount = recentDeployments.filter(d => d.status === 'success').length
  const successRate = ((successCount / recentDeployments.length) * 100).toFixed(0)
  const totalDeployed = recentDeployments.reduce((a, d) => a + d.amount, 0)
  const executedOpps = opportunities.filter(o => o.status === 'executed').length

  return (
    <div style={{ padding: '32px', maxWidth: '1200px' }}>

      <div style={{ marginBottom: '32px', borderBottom: '1px solid #1e2530', paddingBottom: '20px' }}>
        <div style={{ fontFamily: 'var(--font-display)', fontSize: '20px', fontWeight: 700, marginBottom: '4px' }}>Protocol Analytics</div>
        <div style={{ color: '#7a8a9e', fontSize: '12px' }}>Historical performance — deployment outcomes, yield generation, market impact</div>
      </div>

      {/* KPI row */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(5, 1fr)', gap: '1px', background: '#1e2530', border: '1px solid #1e2530', borderRadius: '6px', overflow: 'hidden', marginBottom: '24px' }}>
        {[
          { label: 'TOTAL DEPLOYMENTS', value: recentDeployments.length.toString() },
          { label: 'SUCCESS RATE', value: `${successRate}%`, color: '#00c896' },
          { label: 'TOTAL P&L (POT)', value: `${totalProfit > 0 ? '+' : ''}${totalProfit.toLocaleString()}`, color: totalProfit > 0 ? '#00c896' : '#e63946' },
          { label: 'CAPITAL DEPLOYED (POT)', value: totalDeployed.toLocaleString() },
          { label: 'AVG SPREAD REDUCTION', value: '1.8%', color: '#1a6cf6' },
        ].map(({ label, value, color }) => (
          <div key={label} style={{ background: '#13161a', padding: '18px 20px' }}>
            <div style={{ fontSize: '10px', color: '#3d4f63', letterSpacing: '0.1em', marginBottom: '8px' }}>{label}</div>
            <div style={{ fontSize: '18px', fontFamily: 'var(--font-display)', fontWeight: 700, color: color || '#e8edf5' }}>{value}</div>
          </div>
        ))}
      </div>

      {/* Charts */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '16px', marginBottom: '16px' }}>

        <div style={{ background: '#13161a', border: '1px solid #1e2530', borderRadius: '6px', padding: '20px' }}>
          <div style={{ fontSize: '10px', color: '#3d4f63', letterSpacing: '0.1em', marginBottom: '16px' }}>SPREAD REDUCTION OVER TIME (%)</div>
          <ResponsiveContainer width="100%" height={180}>
            <LineChart data={chartData} margin={{ top: 0, right: 0, left: 0, bottom: 0 }}>
              <CartesianGrid strokeDasharray="2 4" stroke="#1e2530" />
              <XAxis dataKey="time" stroke="#3d4f63" tick={{ fontSize: 10, fill: '#3d4f63' }} axisLine={false} tickLine={false} />
              <YAxis stroke="#3d4f63" tick={{ fontSize: 10, fill: '#3d4f63' }} axisLine={false} tickLine={false} domain={[0, 3]} />
              <Tooltip formatter={(v) => [`${v}%`, 'Spread']} contentStyle={{ background: '#0f1114', border: '1px solid #1e2530', fontSize: '11px' }} />
              <Line type="monotone" dataKey="spread" name="Spread" stroke="#1a6cf6" strokeWidth={1.5} dot={false} />
            </LineChart>
          </ResponsiveContainer>
        </div>

        <div style={{ background: '#13161a', border: '1px solid #1e2530', borderRadius: '6px', padding: '20px'  }}>
          <div style={{ fontSize: '10px', color: '#3d4f63', letterSpacing: '0.1em', marginBottom: '16px' }}>YIELD GENERATED PER PERIOD (POT)</div>
          <ResponsiveContainer width="100%" height={180}>
            <BarChart data={chartData} margin={{ top: 0, right: 0, left: 0, bottom: 0 }}>
              <CartesianGrid strokeDasharray="2 4" stroke="#1e2530" />
              <XAxis dataKey="time" stroke="#3d4f63" tick={{ fontSize: 10, fill: '#3d4f63' }} axisLine={false} tickLine={false} />
              <YAxis stroke="#3d4f63" tick={{ fontSize: 10, fill: '#3d4f63' }} axisLine={false} tickLine={false} tickFormatter={v => `${(v/1000).toFixed(0)}K`} />
              <Tooltip formatter={(v) => [`${v.toLocaleString()} POT`, 'Yield']} contentStyle={{ background: '#0f1114', border: '1px solid #1e2530', fontSize: '11px' }} />
              <Bar dataKey="yield" name="Yield" fill="#1a6cf6" radius={[2, 2, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Deployment breakdown table */}
      <div style={{ background: '#13161a', border: '1px solid #1e2530', borderRadius: '6px', padding: '20px' }}>
        <div style={{ fontSize: '10px', color: '#3d4f63', letterSpacing: '0.1em', marginBottom: '16px' }}>DEPLOYMENT BREAKDOWN</div>
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ borderBottom: '1px solid #1e2530' }}>
              {['ID', 'PAIR', 'DEPLOYED (POT)', 'P&L (POT)', 'RETURN %', 'DURATION', 'OUTCOME'].map(h => (
                <th key={h} style={{ textAlign: 'left', padding: '6px 12px', color: '#3d4f63', fontSize: '10px', letterSpacing: '0.08em', fontWeight: 400 }}>{h}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {recentDeployments.map(d => {
              const returnPct = ((d.profit / d.amount) * 100).toFixed(2)
              return (
                <tr key={d.id} style={{ borderBottom: '1px solid #1e2530' }}>
                  <td style={{ padding: '10px 12px', color: '#3d4f63', fontSize: '12px' }}>#{d.id}</td>
                  <td style={{ padding: '10px 12px', fontSize: '12px' }}>{d.pair}</td>
                  <td style={{ padding: '10px 12px', fontSize: '12px' }}>{d.amount.toLocaleString()}</td>
                  <td style={{ padding: '10px 12px', fontSize: '12px', color: d.profit > 0 ? '#00c896' : '#e63946' }}>
                    {d.profit > 0 ? '+' : ''}{d.profit}
                  </td>
                  <td style={{ padding: '10px 12px', fontSize: '12px', color: d.profit > 0 ? '#00c896' : '#e63946' }}>
                    {d.profit > 0 ? '+' : ''}{returnPct}%
                  </td>
                  <td style={{ padding: '10px 12px', color: '#7a8a9e', fontSize: '12px' }}>{d.duration}</td>
                  <td style={{ padding: '10px 12px' }}>
                    <span style={{
                      fontSize: '10px', padding: '2px 8px', borderRadius: '3px', letterSpacing: '0.05em',
                      background: d.status === 'success' ? 'rgba(0,200,150,0.08)' : 'rgba(230,57,70,0.08)',
                      color: d.status === 'success' ? '#00c896' : '#e63946',
                    }}>{d.status.toUpperCase()}</span>
                  </td>
                </tr>
              )
            })}
          </tbody>
          <tfoot>
            <tr style={{ borderTop: '1px solid #2a3444' }}>
              <td colSpan={2} style={{ padding: '10px 12px', color: '#3d4f63', fontSize: '11px' }}>TOTAL</td>
              <td style={{ padding: '10px 12px', fontSize: '12px' }}>{totalDeployed.toLocaleString()}</td>
              <td style={{ padding: '10px 12px', fontSize: '12px', color: totalProfit > 0 ? '#00c896' : '#e63946' }}>
                {totalProfit > 0 ? '+' : ''}{totalProfit}
              </td>
              <td style={{ padding: '10px 12px', fontSize: '12px', color: totalProfit > 0 ? '#00c896' : '#e63946' }}>
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