import { BarChart2, Zap, Shield, Terminal, Activity, Hexagon } from 'lucide-react'

const navItems = [
  { id: 'dashboard', label: 'Dashboard', icon: BarChart2 },
  { id: 'opportunities', label: 'Opportunity Feed', icon: Zap },
  { id: 'pool', label: 'Stability Pool', icon: Shield },
  { id: 'terminal', label: 'AI Terminal', icon: Terminal },
  { id: 'analytics', label: 'Analytics', icon: Activity },
]

export default function Sidebar({ active, onNavigate }) {
  return (
    <aside style={{
      width: '220px',
      minWidth: '220px',
      background: '#0c0e12',
      borderRight: '1px solid var(--border)',
      display: 'flex',
      flexDirection: 'column',
      padding: '0',
    }}>
      {/* Logo */}
      <div style={{
        padding: '24px 20px',
        borderBottom: '1px solid var(--border)',
        display: 'flex',
        alignItems: 'center',
        gap: '10px',
      }}>
        <div style={{
          width: '32px', height: '32px',
          background: 'var(--blue-primary)',
          borderRadius: '8px',
          display: 'flex', alignItems: 'center', justifyContent: 'center',
        }}>
          <Hexagon size={16} color="#fff" fill="#fff" />
        </div>
        <div>
          <div style={{ fontFamily: 'var(--font-display)', fontWeight: 800, fontSize: '15px', letterSpacing: '0.02em' }}>EQUILIBRIA</div>
          <div style={{ color: 'var(--text-dim)', fontSize: '10px', letterSpacing: '0.1em' }}>PROTOCOL v1.0</div>
        </div>
      </div>

      {/* Nav */}
      <nav style={{ padding: '12px 8px', flex: 1 }}>
        {navItems.map(({ id, label, icon: Icon }) => {
          const isActive = active === id
          return (
            <button key={id} onClick={() => onNavigate(id)} style={{
              width: '100%',
              display: 'flex',
              alignItems: 'center',
              gap: '10px',
              padding: '10px 12px',
              borderRadius: '6px',
              border: 'none',
              background: isActive ? 'transparent' : 'transparent',
              color: isActive ? 'var(--blue-primary)' : 'var(--text-secondary)',
              cursor: 'pointer',
              textAlign: 'left',
              fontFamily: 'var(--font-mono)',
              fontSize: '12px',
              letterSpacing: '0.03em',
              marginBottom: '2px',
              borderLeft: isActive ? '2px solid var(--blue-primary)' : '2px solid transparent',
              transition: 'all 0.15s ease',
            }}>
              <Icon size={14} />
              {label}
            </button>
          )
        })}
      </nav>

      {/* Status */}
      <div style={{
        padding: '16px 20px',
        borderTop: '1px solid var(--border)',
        fontSize: '11px',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px', color: 'var(--green)', marginBottom: '4px' }}>
          <div style={{ width: '6px', height: '6px', borderRadius: '50%', background: 'var(--green)', boxShadow: '0 0 6px var(--green)' }} />
          NODE CONNECTED
        </div>
        <div style={{ color: 'var(--text-dim)' }}>Portaldot Testnet</div>
        <div style={{ color: 'var(--text-dim)', fontFamily: 'var(--font-mono)' }}>Block #20,255</div>
      </div>
    </aside>
  )
}