import { useState, useEffect } from 'react'
import { api } from '../api'

export default function KeysPage() {
  const [keys, setKeys] = useState([])
  const [newKey, setNewKey] = useState(null)
  const [label, setLabel] = useState('')
  const [loading, setLoading] = useState(false)

  const load = () => api.listKeys().then(d => setKeys(d.keys || []))
  useEffect(() => { load() }, [])

  const handleCreate = async () => {
    setLoading(true)
    const data = await api.createKey(label || 'default')
    if (data.api_key) { setNewKey(data.api_key); setLabel(''); load() }
    setLoading(false)
  }

  const handleRevoke = async (id) => {
    if (!confirm('Revoke this key? This action cannot be undone.')) return
    await api.revokeKey(id)
    load()
  }

  return (
    <div className="page">
      <div className="page-header">
        <h2>API Keys</h2>
        <p>Manage your API keys for authentication</p>
      </div>

      <div className="card">
        <h3>Generate New Key</h3>
        <div className="form-row">
          <input type="text" placeholder="Label (optional)" value={label}
            onChange={e => setLabel(e.target.value)} className="input-inline" />
          <button className="btn-primary" onClick={handleCreate} disabled={loading}>
            {loading ? 'Generating...' : 'Generate Key'}
          </button>
        </div>
        {newKey && (
          <div className="new-key-banner">
            <p>⚠ Save this key now — it won't be shown again:</p>
            <div className="key-display">
              <code>{newKey}</code>
              <button className="btn-copy" onClick={() => { navigator.clipboard.writeText(newKey); }}>Copy</button>
            </div>
            <button className="btn-dismiss" onClick={() => setNewKey(null)}>I've saved it</button>
          </div>
        )}
      </div>

      <div className="card">
        <h3>Active Keys ({keys.filter(k => k.is_active).length})</h3>
        <div className="keys-list">
          {keys.length === 0 && <p className="empty">No API keys yet</p>}
          {keys.map(k => (
            <div key={k.id} className={`key-row ${k.is_active ? '' : 'revoked'}`}>
              <div className="key-info">
                <code className="key-prefix">{k.key_prefix}</code>
                <span className="key-label">{k.label}</span>
                <span className={`key-status ${k.is_active ? 'active' : 'inactive'}`}>
                  {k.is_active ? 'Active' : 'Revoked'}
                </span>
              </div>
              <div className="key-meta">
                <span>Created: {new Date(k.created_at).toLocaleDateString()}</span>
                {k.last_used_at && <span>Last used: {new Date(k.last_used_at).toLocaleDateString()}</span>}
              </div>
              {k.is_active && (
                <button className="btn-danger-sm" onClick={() => handleRevoke(k.id)}>Revoke</button>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
