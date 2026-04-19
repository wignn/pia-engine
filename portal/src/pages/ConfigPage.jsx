import { useState, useEffect } from 'react'
import { api } from '../api'

export default function ConfigPage() {
  const [config, setConfig] = useState({})
  const [limits, setLimits] = useState(null)
  const [xInput, setXInput] = useState('')
  const [tvInput, setTvInput] = useState('')
  const [saving, setSaving] = useState(false)
  const [msg, setMsg] = useState('')

  useEffect(() => {
    api.listConfig().then(d => {
      setConfig(d.configs || {})
      setLimits(d.plan_limits)
      const xu = d.configs?.x_usernames
      if (Array.isArray(xu)) setXInput(xu.join(', '))
      const tv = d.configs?.tv_symbols
      if (Array.isArray(tv)) setTvInput(tv.join(', '))
    })
  }, [])

  const saveX = async () => {
    setSaving(true); setMsg('')
    const usernames = xInput.split(',').map(s => s.trim()).filter(Boolean)
    const res = await api.setConfig('x_usernames', usernames)
    setMsg(res.error || 'X usernames saved!')
    setSaving(false)
  }

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
        <p>Customize your data feeds and sources</p>
      </div>

      {msg && <div className={`toast ${msg.includes('saved') ? 'success' : 'error'}`}>{msg}</div>}

      <div className="card">
        <h3>X (Twitter) Feed</h3>
        <p className="card-desc">
          Configure which X accounts to monitor. Your plan allows up to <strong>{limits?.x_usernames_max || 1}</strong> usernames.
        </p>
        <div className="form-group">
          <label>Usernames (comma separated)</label>
          <input type="text" placeholder="elonmusk, naval, VitalikButerin"
            value={xInput} onChange={e => setXInput(e.target.value)} />
        </div>
        <button className="btn-primary" onClick={saveX} disabled={saving}>Save X Usernames</button>
      </div>

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
    </div>
  )
}
