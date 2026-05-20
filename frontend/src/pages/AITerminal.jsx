import { aiLogs } from '../data/mockData'
import { useState, useEffect, useRef } from 'react'

const typeColors = {
  SCAN:    '#3d4f63',
  DETECT:  '#f4a621',
  SCORE:   '#1a6cf6',
  APPROVE: '#00c896',
  EXECUTE: '#00c896',
  RETURN:  '#00c896',
  PENDING: '#f4a621',
  REJECT:  '#e63946',
}

export default function AITerminal() {
  const [logs, setLogs] = useState(aiLogs)
  const [running, setRunning] = useState(true)
  const [scanInterval, setScanInterval] = useState(3)
  const bottomRef = useRef()

  useEffect(() => {
    if (!running) return
    const interval = setInterval(() => {
      const pairs = ['POT/rUSD', 'POT/USDC', 'rUSD/POT']
      const pair = pairs[Math.floor(Math.random() * pairs.length)]
      const now = new Date().toTimeString().slice(0, 8)
      const spread = (Math.random() * 3 + 0.3).toFixed(2)
      const profit = Math.floor(Math.random() * 40 + 45)
      const risk = Math.floor(Math.random() * 50 + 10)
      const stability = Math.floor(Math.random() * 40 + 50)
      const confidence = Math.floor(Math.random() * 30 + 60)
      const composite = Math.round(profit * 0.3 + (100 - risk) * 0.25 + stability * 0.35 + confidence * 0.1)

      const batch = [
        { time: now, type: 'SCAN', message: `Scan #${Math.floor(Math.random() * 900 + 100)} — monitoring ${pairs.length} active pairs`, score: null },
      ]

      if (Math.random() > 0.3) {
        batch.push({ time: now, type: 'DETECT', message: `Price divergence: ${pair} spread ${spread}% between Pool A and Pool B`, score: null })
        batch.push({ time: now, type: 'SCORE', message: `Profit ${profit} · Risk ${risk} · Stability ${stability} · Confidence ${confidence}`, score: null })
        if (composite >= 70) {
          batch.push({ time: now, type: 'APPROVE', message: `Composite ${composite} ≥ 70 threshold. Posting opportunity to registry.`, score: composite })
        } else {
          batch.push({ time: now, type: 'REJECT', message: `Composite ${composite} < 70 threshold. Opportunity discarded.`, score: composite })
        }
      }

      setLogs(prev => [...prev.slice(-80), ...batch])
    }, scanInterval * 1000)
    return () => clearInterval(interval)
  }, [running, scanInterval])

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [logs])

  return (
    <div style={{ padding: '32px', height: '100%', display: 'flex', flexDirection: 'column', maxWidth: '1100px' }}>

      <div style={{ marginBottom: '24px', borderBottom: '1px solid #1e2530', paddingBottom: '20px', display: 'flex', justifyContent: 'space-between', alignItems: 'flex-end' }}>
        <div>
          <div style={{ fontFamily: 'var(--font-display)', fontSize: '20px', fontWeight: 700, marginBottom: '4px' }}>AI Intelligence Terminal</div>
          <div style={{ color: '#7a8a9e', fontSize: '12px' }}>Live scoring engine output — every decision logged with full reasoning</div>
        </div>
        <div style={{ display: 'flex', gap: '10px', alignItems: 'center' }}>
          <div style={{ fontSize: '11px', color: '#3d4f63' }}>
            Scan every
            <select
              value={scanInterval}
              onChange={e => setScanInterval(Number(e.target.value))}
              style={{ margin: '0 6px', background: '#0f1114', border: '1px solid #1e2530', color: '#e8edf5', padding: '2px 6px', fontFamily: 'var(--font-mono)', fontSize: '11px', borderRadius: '3px' }}
            >
              {[1, 3, 5, 10].map(v => <option key={v} value={v}>{v}s</option>)}
            </select>
            seconds
          </div>
          <button onClick={() => setRunning(r => !r)} style={{
            padding: '6px 14px', borderRadius: '3px', border: '1px solid',
            borderColor: running ? '#00c896' : '#1e2530',
            background: running ? 'rgba(0,200,150,0.08)' : 'transparent',
            color: running ? '#00c896' : '#7a8a9e',
            cursor: 'pointer', fontFamily: 'var(--font-mono)', fontSize: '11px', letterSpacing: '0.06em',
          }}>
            {running ? '● RUNNING' : '○ PAUSED'}
          </button>
          <button onClick={() => setLogs([])} style={{
            padding: '6px 14px', borderRadius: '3px', border: '1px solid #1e2530',
            background: 'transparent', color: '#3d4f63',
            cursor: 'pointer', fontFamily: 'var(--font-mono)', fontSize: '11px', letterSpacing: '0.06em',
          }}>CLEAR</button>
        </div>
      </div>

      {/* Score weights reference */}
      <div style={{ display: 'flex', gap: '24px', marginBottom: '16px', padding: '10px 14px', background: '#13161a', border: '1px solid #1e2530', borderRadius: '4px', fontSize: '11px' }}>
        <span style={{ color: '#3d4f63' }}>Scoring weights:</span>
        <span><span style={{ color: '#00c896' }}>Profit</span> ×0.30</span>
        <span><span style={{ color: '#e63946' }}>Risk (inv)</span> ×0.25</span>
        <span><span style={{ color: '#1a6cf6' }}>Stability</span> ×0.35</span>
        <span><span style={{ color: '#7a8a9e' }}>Confidence</span> ×0.10</span>
        <span style={{ marginLeft: 'auto', color: '#3d4f63' }}>Execute threshold: ≥70</span>
      </div>

      {/* Terminal output */}
      <div style={{
        flex: 1, background: '#080a0c', border: '1px solid #1e2530',
        borderRadius: '6px', padding: '16px 20px', overflow: 'auto',
        fontFamily: 'var(--font-mono)', fontSize: '12px', lineHeight: '1.9',
        minHeight: '400px',
      }}>
        {logs.map((log, i) => (
          <div key={i} style={{ display: 'flex', gap: '14px' }}>
            <span style={{ color: '#3d4f63', minWidth: '72px', userSelect: 'none' }}>{log.time}</span>
            <span style={{
              minWidth: '66px', color: typeColors[log.type] || '#7a8a9e',
              fontSize: '10px', letterSpacing: '0.06em', paddingTop: '2px',
            }}>[{log.type}]</span>
            <span style={{ color: '#e8edf5' }}>
              {log.message}
              {log.score !== null && log.score !== undefined &&
                <span style={{ marginLeft: '8px', color: log.score >= 70 ? '#00c896' : '#e63946' }}>
                  → {log.score}
                </span>
              }
            </span>
          </div>
        ))}
        <div style={{ display: 'flex', gap: '14px' }}>
          <span style={{ minWidth: '72px' }} />
          <span style={{ color: '#1a6cf6', animation: 'none' }}>█</span>
        </div>
        <div ref={bottomRef} />
      </div>
    </div>
  )
}