import { useState } from 'react'
import Sidebar from './components/Sidebar'
import Dashboard from './pages/Dashboard'
import OpportunityFeed from './pages/OpportunityFeed'
import StabilityPool from './pages/StabilityPool'
import AITerminal from './pages/AITerminal'
import Analytics from './pages/Analytics'
import './index.css'

export default function App() {
  const [page, setPage] = useState('dashboard')

  const pages = {
    dashboard: <Dashboard />,
    opportunities: <OpportunityFeed />,
    pool: <StabilityPool />,
    terminal: <AITerminal />,
    analytics: <Analytics />,
  }

  return (
    <div style={{ display: 'flex', height: '100vh', overflow: 'hidden' }}>
      <Sidebar active={page} onNavigate={setPage} />
      <main style={{
        flex: 1,
        overflow: 'auto',
        background: 'var(--bg-primary)',
      }}>
        {pages[page]}
      </main>
    </div>
  )
}