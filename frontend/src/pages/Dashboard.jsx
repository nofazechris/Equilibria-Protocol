import { poolState, chartData, recentDeployments, opportunities } from '../data/mockData'
import { AreaChart, Area, XAxis, YAxis, Tooltip, ResponsiveContainer } from 'recharts'

const CustomTooltip = ({ active, payload, label }) => {
  if (!active || !payload?.length) return null
  return (
    <div style={{ background: '#0f1114', border: '1px solid #1e2530', padding: '8px 12px', borderRadius: '4px', fontSize: '11px' }}>
      <div style={{ color: '#3d4f63', marginBottom: '3px' }}>{label}</div>
      {payload.map((p, i) => (
        <div key={i} style={{ color: '#e8edf5' }}>
          {p.name}: {p.value > 1000 ? `${(p.value/1000000).toFixed(2)}M POT` : `${p.value} POT`}
        </div>
      ))}
    </div>
  )
}

export default function Dashboard() {
  const activeOpps = opportunities.filter(o => o.status === 'active').length

  return (
    <div style={{ padding: '32px', maxWidth: '1300px' }}>

      {/* Page title */}
      <div style={{ marginBottom: '32px', borderBottom: '1px solid #1e2530', paddingBottom: '20px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '4px' }}>
          <span style={{ fontFamily: 'var(--font-display)', fontSize: '20px', fontWeight: 700 }}>Protocol Overview</span>
          <span style={{ background: '#0d2d6e', color: '#1a6cf6', fontSize: '10px', padding: '2px 7px', borderRadius: '3px', letterSpacing: '0.08em' }}>LIVE</span>
        </div>
        <div style={{ color: '#7a8a9e', fontSize: '12px' }}>Equilibria — autonomous market stabilization on Portaldot</div>
      </div>

      {/* Four key metrics — plain row, no cards with gradients */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '1px', background: '#1e2530', border: '1px solid #1e2530', borderRadius: '6px', overflow: 'hidden', marginBottom: '28px' }}>
        {[
          { label: 'TOTAL VALUE LOCKED', value: `${(poolState.totalDeposited/1000000).toFixed(2)}M POT`, sub: `${poolState.depositors} depositors` },
          { label: 'DEPLOYED CAPITAL', value: `${(poolState.deployedCapital/1000).toFixed(0)}K POT`, sub: `${poolState.utilizationRate}% utilization` },
          { label: 'YIELD ACCUMULATED', value: `${(poolState.yieldAccumulated/1000).toFixed(1)}K POT`, sub: `${poolState.apr}% APR` },
          { label: 'ACTIVE OPPORTUNITIES', value: activeOpps, sub: 'of 6 scanned pairs' },
        ].map(({ label, value, sub }, i) => (
          <div key={i} style={{ background: '#13161a', padding: '20px 24px' }}>
            <div style={{ fontSize: '10px', color: '#3d4f63', letterSpacing: '0.1em', marginBottom: '10px' }}>{label}</div>
            <div style={{ fontSize: '22px', fontFamily: 'var(--font-display)', fontWeight: 700, color: '#e8edf5', marginBottom: '4px' }}>{value}</div>
            <div style={{ fontSize: '11px', color: '#7a8a9e' }}>{sub}</div>
          </div>
        ))}
      </div>

      {/* TVL Chart + Pool Status side by side */}
      <div style={{ display: 'grid', gridTemplateColumns: '2fr 1fr', gap: '16px', marginBottom: '16px' }}>

        {/* TVL chart */}
        <div style={{ background: '#13161a', border: '1px solid #1e2530', borderRadius: '6px', padding: '20px' }}>
          <div style={{ fontSize: '10px', color: '#3d4f63', letterSpacing: '0.1em', marginBottom: '20px' }}>TOTAL VALUE LOCKED — 24H</div>
          <ResponsiveContainer width="100%" height={180}>
            <AreaChart data={chartData} margin={{ top: 0, right: 0, left: 0, bottom: 0 }}>
              <defs>
                <linearGradient id="tvlGrad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#1a6cf6" stopOpacity={0.2} />
                  <stop offset="95%" stopColor="#1a6cf6" stopOpacity={0} />
                </linearGradient>
              </defs>
              <XAxis dataKey="time" stroke="#3d4f63" tick={{ fontSize: 10, fill: '#3d4f63' }} axisLine={false} tickLine={false} />
              <YAxis stroke="#3d4f63" tick={{ fontSize: 10, fill: '#3d4f63' }} axisLine={false} tickLine={false} tickFormatter={v => `${(v/1000000).toFixed(1)}M`} />
              <Tooltip content={<CustomTooltip />} />
              <Area type="monotone" dataKey="tvl" name="TVL" stroke="#1a6cf6" fill="url(#tvlGrad)" strokeWidth={1.5} dot={false} />
            </AreaChart>
          </ResponsiveContainer>
        </div>

        {/* Pool status */}
        <div style={{ background: '#13161a', border: '1px solid #1e2530', borderRadius: '6px', padding: '20px' }}>
          <div style={{ fontSize: '10px', color: '#3d4f63', letterSpacing: '0.1em', marginBottom: '20px' }}>POOL STATUS</div>

          <div style={{ marginBottom: '20px' }}>
            {[
              { label: 'Idle Reserve', value: 85.5, color: '#1a6cf6' },
              { label: 'Deployed', value: 14.5, color: '#f4a621' },
            ].map(({ label, value, color }) => (
              <div key={label} style={{ marginBottom: '14px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '5px', fontSize: '11px' }}>
                  <span style={{ color: '#7a8a9e' }}>{label}</span>
                  <span style={{ color }}>{value}%</span>
                </div>
                <div style={{ height: '3px', background: '#1e2530', borderRadius: '2px' }}>
                  <div style={{ width: `${value}%`, height: '100%', background: color, borderRadius: '2px' }} />
                </div>
              </div>
            ))}
          </div>

          <div style={{ borderTop: '1px solid #1e2530', paddingTop: '16px' }}>
            <div style={{ fontSize: '10px', color: '#3d4f63', letterSpacing: '0.08em', marginBottom: '10px' }}>RISK CONTROLS</div>
            {[
              { label: 'Max deployment', value: '20% of pool' },
              { label: 'Circuit breaker', value: '● ARMED', valueColor: '#00c896' },
              { label: 'Pool status', value: '● ACTIVE', valueColor: '#00c896' },
            ].map(({ label, value, valueColor }) => (
              <div key={label} style={{ display: 'flex', justifyContent: 'space-between', fontSize: '11px', marginBottom: '6px' }}>
                <span style={{ color: '#7a8a9e' }}>{label}</span>
                <span style={{ color: valueColor || '#e8edf5' }}>{value}</span>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Recent deployments table */}
      <div style={{ background: '#13161a', border: '1px solid #1e2530', borderRadius: '6px', padding: '20px' }}>
        <div style={{ fontSize: '10px', color: '#3d4f63', letterSpacing: '0.1em', marginBottom: '16px' }}>RECENT DEPLOYMENTS</div>
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ borderBottom: '1px solid #1e2530' }}>
              {['ID', 'PAIR', 'DEPLOYED (POT)', 'P&L (POT)', 'DURATION', 'STATUS'].map(h => (
                <th key={h} style={{ textAlign: 'left', padding: '6px 12px', color: '#3d4f63', fontSize: '10px', letterSpacing: '0.08em', fontWeight: 400 }}>{h}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {recentDeployments.map(d => (
              <tr key={d.id} style={{ borderBottom: '1px solid #1e2530' }}>
                <td style={{ padding: '10px 12px', color: '#3d4f63', fontSize: '12px' }}>#{d.id}</td>
                <td style={{ padding: '10px 12px', color: '#e8edf5', fontSize: '12px' }}>{d.pair}</td>
                <td style={{ padding: '10px 12px', fontSize: '12px' }}>{d.amount.toLocaleString()} POT</td>
                <td style={{ padding: '10px 12px', fontSize: '12px', color: d.profit > 0 ? '#00c896' : '#e63946' }}>
                  {d.profit > 0 ? '+' : ''}{d.profit} POT
                </td>
                <td style={{ padding: '10px 12px', color: '#7a8a9e', fontSize: '12px' }}>{d.duration}</td>
                <td style={{ padding: '10px 12px' }}>
                  <span style={{
                    fontSize: '10px', padding: '2px 8px', borderRadius: '3px',
                    background: d.status === 'success' ? 'rgba(0,200,150,0.08)' : 'rgba(230,57,70,0.08)',
                    color: d.status === 'success' ? '#00c896' : '#e63946',
                    letterSpacing: '0.05em',
                  }}>{d.status.toUpperCase()}</span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}