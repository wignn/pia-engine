import { useState, useEffect } from 'react'
import { api } from '../api'

export default function ConfigPage() {
  const [limits, setLimits] = useState(null)
  const [tvInput, setTvInput] = useState('')
  const [saving, setSaving] = useState(false)
  const [msg, setMsg] = useState('')

  useEffect(() => {
    api.listConfig().then(d => {
      setLimits(d.plan_limits)
      const tv = d.configs?.tv_symbols
      if (Array.isArray(tv)) setTvInput(tv.join(', '))
    })
  }, [])

  const saveTv = async () => {
    setSaving(true); setMsg('')
    const symbols = tvInput.split(',').map(s => s.trim()).filter(Boolean)
    const res = await api.setConfig('tv_symbols', symbols)
    setMsg(res.error || 'TV symbols saved!')
    setSaving(false)
  }

  return (
    <div className="page">
      <div className="page-header">
        <h2>Configuration</h2>
        <p>Customize your market data feeds</p>
      </div>

      {msg && <div className={`toast ${msg.includes('saved') ? 'success' : 'error'}`}>{msg}</div>}

      <div className="card">
        <h3>TradingView Symbols</h3>
        <p className="card-desc">
          Configure market symbols to stream. Your plan allows up to <strong>{limits?.tv_symbols_max || 3}</strong> symbols.
        </p>
        <div className="form-group">
          <label>Symbols (comma separated)</label>
          <input type="text" placeholder="BINANCE:BTCUSDT, OANDA:XAUUSD"
            value={tvInput} onChange={e => setTvInput(e.target.value)} />
        </div>
        <button className="btn-primary" onClick={saveTv} disabled={saving}>Save TV Symbols</button>
      </div>

      <div className="card info-card">
        <h3>📡 X (Twitter) Feed</h3>
        <p className="card-desc">
          X feed accounts are configured globally by the platform administrator.
          All users receive the same curated feed of market-relevant accounts.
        </p>
      </div>
    </div>
  )
}
