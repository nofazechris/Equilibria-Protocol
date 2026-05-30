import { aiLogs } from '../data/mockData'
import { useState, useEffect, useRef } from 'react'
import '../styles/terminal.css'

const PAIRS = ['POT/rUSD', 'POT/USDC', 'rUSD/POT']

export default function AITerminal() {
  const [logs, setLogs]             = useState(aiLogs)
  const [running, setRunning]       = useState(true)
  const [scanInterval, setScanInterval] = useState(3)
  const bottomRef                   = useRef()

 useEffect(() => {
    const ws = new WebSocket('ws://localhost:8765')
    ws.onmessage = (event) => {
        const log = JSON.parse(event.data)
        setLogs(prev => [...prev.slice(-80), log])
    }
    ws.onerror = () => console.log('Keeper not connected — using simulation')
    return () => ws.close()
}, [])


  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [logs])

  return (
    <div className="terminal-wrapper">

      <div className="terminal-header">
        <div className="terminal-header-left">
          <div className="page-title">AI Intelligence Terminal</div>
          <div className="page-subtitle">Live scoring engine output — every decision logged with full reasoning</div>
        </div>
        <div className="terminal-controls">
          <span className="terminal-scan-label">
            Scan every
            <select
              className="terminal-scan-select"
              value={scanInterval}
              onChange={e => setScanInterval(Number(e.target.value))}
            >
              {[1, 3, 5, 10].map(v => <option key={v} value={v}>{v}s</option>)}
            </select>
            seconds
          </span>
          <button
            onClick={() => setRunning(r => !r)}
            className={`terminal-run-btn ${running ? 'running' : ''}`}
          >
            {running ? '● RUNNING' : '○ PAUSED'}
          </button>
          <button onClick={() => setLogs([])} className="terminal-clear-btn">
            CLEAR
          </button>
        </div>
      </div>

      {/* Weights reference */}
      <div className="weights-bar">
        <span className="weights-label">Scoring weights:</span>
        <span><span style={{ color: 'var(--green)' }}>Profit</span> ×0.30</span>
        <span><span style={{ color: 'var(--red)' }}>Risk (inv)</span> ×0.25</span>
        <span><span style={{ color: 'var(--blue-primary)' }}>Stability</span> ×0.35</span>
        <span><span style={{ color: 'var(--text-secondary)' }}>Confidence</span> ×0.10</span>
        <span className="weights-threshold">Execute threshold: ≥ 70</span>
      </div>

      {/* Terminal output */}
      <div className="terminal-output">
        {logs.map((log, i) => (
          <div key={i} className="log-line">
            <span className="log-time">{log.time}</span>
            <span className={`log-type log-type-${log.type}`}>[{log.type}]</span>
            <span className="log-message">
              {log.message}
              {log.score !== null && log.score !== undefined && (
                <span className={`log-score ${log.score >= 70 ? 'pass' : 'fail'}`}>
                  → {log.score}
                </span>
              )}
            </span>
          </div>
        ))}
        <div className="log-line">
          <span className="log-time" />
          <span className="log-cursor">|</span>
        </div>
        <div ref={bottomRef} />
      </div>
    </div>
  )
}