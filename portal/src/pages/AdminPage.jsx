import { useState, useEffect, useRef, useCallback } from 'react'
import { api, CORE_WS_BASE } from '../api'

const PLANS = ['free', 'basic', 'pro', 'enterprise']

function useLiveWS(endpoint) {
  const [messages, setMessages] = useState([])
  const [status, setStatus] = useState('disconnected')
  const wsRef = useRef(null)
  const reconnectTimer = useRef(null)
  const apiKey = localStorage.getItem('wi_api_key') || ''

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return
    const url = `${CORE_WS_BASE}/api/v1/ws/${endpoint}?api_key=${apiKey}&bot_id=admin-${endpoint}`
    setStatus('connecting')
    const ws = new WebSocket(url)
    wsRef.current = ws

    ws.onopen = () => setStatus('live')
    ws.onclose = () => {
      setStatus('disconnected')
      reconnectTimer.current = setTimeout(connect, 4000)
    }
    ws.onerror = () => setStatus('error')
    ws.onmessage = (e) => {
      try {
        const data = JSON.parse(e.data)
        if (data.event === 'connected') return
        setMessages(prev => [{ ...data, _ts: Date.now() }, ...prev].slice(0, 80))
      } catch {}
    }
  }, [endpoint, apiKey])

  useEffect(() => {
    connect()
    return () => {
      wsRef.current?.close()
      clearTimeout(reconnectTimer.current)
    }
  }, [connect])

  const clear = () => setMessages([])
  return { messages, status, clear }
}

function StatusDot({ status }) {
  const colors = { live: '#00d2a0', connecting: '#ffa502', error: '#ff4757', disconnected: '#555570' }
  return (
    <span style={{ display: 'inline-flex', alignItems: 'center', gap: 6, fontSize: 12, color: colors[status] }}>
      <span style={{
        width: 7, height: 7, borderRadius: '50%', background: colors[status],
        boxShadow: status === 'live' ? `0 0 8px ${colors[status]}` : 'none',
        animation: status === 'live' ? 'pulse 2s infinite' : 'none'
      }} />
      {status.toUpperCase()}
    </span>
  )
}

function PriceFeed() {
  const { messages, status, clear } = useLiveWS('forex')
  const prices = {}
  messages.forEach(m => {
    const tick = m.data?.tick
    if (tick?.symbol) prices[tick.symbol] = tick
  })
  const latestPrices = Object.values(prices)

  return (
    <div className="admin-panel">
      <div className="panel-header">
        <div>
          <h3>📈 Price Feed</h3>
          <StatusDot status={status} />
        </div>
        <button className="btn-sm" onClick={clear}>Clear</button>
      </div>
      <div className="price-grid">
        {latestPrices.length === 0 && messages.length === 0 && (
          <div className="feed-empty">Waiting for price ticks…</div>
        )}
        {latestPrices.map(tick => (
          <div key={tick.symbol} className="price-card">
            <div className="price-symbol">{tick.symbol}</div>
            <div className="price-value">{tick.price?.toFixed(tick.price > 100 ? 2 : 5)}</div>
            <div className={`price-change ${(tick.change_percent || 0) >= 0 ? 'pos' : 'neg'}`}>
              {(tick.change_percent || 0) >= 0 ? '▲' : '▼'} {Math.abs(tick.change_percent || 0).toFixed(3)}%
            </div>
            {tick.bid && <div className="price-spread">B: {tick.bid?.toFixed(5)} | A: {tick.ask?.toFixed(5)}</div>}
          </div>
        ))}
      </div>
      <div className="feed-log">
        {messages.slice(0, 20).map((m, i) => {
          const tick = m.data?.tick
          if (!tick) return null
          return (
            <div key={i} className="feed-log-row">
              <span className="log-time">{new Date(m._ts).toLocaleTimeString()}</span>
              <span className="log-sym">{tick.symbol}</span>
              <span className="log-price">{tick.price?.toFixed(5)}</span>
            </div>
          )
        })}
      </div>
    </div>
  )
}

function XFeed() {
  const { messages, status, clear } = useLiveWS('x')
  const posts = messages.filter(m => m.event === 'x.new')

  return (
    <div className="admin-panel">
      <div className="panel-header">
        <div>
          <h3>𝕏 Live Feed</h3>
          <StatusDot status={status} />
        </div>
        <button className="btn-sm" onClick={clear}>Clear</button>
      </div>
      {posts.length === 0 && <div className="feed-empty">Waiting for tweets…</div>}
      <div className="feed-list">
        {posts.map((m, i) => {
          const post = m.data?.post
          return (
            <div key={i} className="x-post">
              <div className="x-meta">
                <span className="x-author">@{post?.author_username}</span>
                <span className="x-time">{post?.created_at ? new Date(post.created_at).toLocaleTimeString() : ''}</span>
              </div>
              <div className="x-text">{post?.text}</div>
              {post?.url && <a href={post.url} target="_blank" rel="noreferrer" className="x-link">View →</a>}
            </div>
          )
        })}
      </div>
    </div>
  )
}

function NewsFeed() {
  const [news, setNews] = useState([])
  const [tab, setTab] = useState('forex')

  useEffect(() => {
    const load = async () => {
      const data = tab === 'forex' ? await api.coreForexNews(30) : await api.coreEquityNews(30)
      setNews(data?.news || data?.items || [])
    }
    load()
    const t = setInterval(load, 30000)
    return () => clearInterval(t)
  }, [tab])

  return (
    <div className="admin-panel">
      <div className="panel-header">
        <div>
          <h3>📰 News Feed</h3>
          <div style={{ display: 'flex', gap: 8, marginTop: 4 }}>
            {['forex', 'equity'].map(t => (
              <button key={t} className={`tab-btn ${tab === t ? 'active' : ''}`} onClick={() => setTab(t)}>
                {t.charAt(0).toUpperCase() + t.slice(1)}
              </button>
            ))}
          </div>
        </div>
        <span style={{ fontSize: 11, color: '#555570' }}>{news.length} items</span>
      </div>
      <div className="feed-list news-list">
        {news.length === 0 && <div className="feed-empty">No news loaded</div>}
        {news.slice(0, 25).map((item, i) => (
          <div key={i} className="news-item">
            <div className="news-meta">
              <span className="news-source">{item.source || item.feed_name}</span>
              <span className="news-time">{item.published_at ? new Date(item.published_at).toLocaleString() : ''}</span>
            </div>
            <div className="news-title">
              {item.url ? <a href={item.url} target="_blank" rel="noreferrer">{item.title}</a> : item.title}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

function UsersTable({ users, onPlanChange, onToggle, loading }) {
  const [search, setSearch] = useState('')
  const [planFilter, setPlanFilter] = useState('all')

  const filtered = users.filter(u => {
    const matchSearch = u.email.includes(search) || u.name.includes(search)
    const matchPlan = planFilter === 'all' || u.plan === planFilter
    return matchSearch && matchPlan
  })

  return (
    <div className="admin-panel full-width">
      <div className="panel-header">
        <h3>👥 User Management</h3>
        <div style={{ display: 'flex', gap: 8 }}>
          <input
            type="text"
            placeholder="Search email or name…"
            value={search}
            onChange={e => setSearch(e.target.value)}
            style={{ width: 220, padding: '6px 12px', fontSize: 13 }}
          />
          <select
            value={planFilter}
            onChange={e => setPlanFilter(e.target.value)}
            style={{ padding: '6px 12px', background: 'var(--bg-input)', border: '1px solid var(--border)', color: 'var(--text-primary)', borderRadius: 8, fontSize: 13 }}
          >
            <option value="all">All Plans</option>
            {PLANS.map(p => <option key={p} value={p}>{p}</option>)}
          </select>
        </div>
      </div>
      <div style={{ overflowX: 'auto' }}>
        <table className="admin-table">
          <thead>
            <tr>
              <th>User</th>
              <th>Plan</th>
              <th>Status</th>
              <th>Verified</th>
              <th>Keys</th>
              <th>Joined</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map(u => (
              <tr key={u.id}>
                <td>
                  <div className="user-cell">
                    <div className="user-avatar-sm">{u.name?.[0]?.toUpperCase() || '?'}</div>
                    <div>
                      <div style={{ fontSize: 13, fontWeight: 600 }}>{u.name}</div>
                      <div style={{ fontSize: 11, color: 'var(--text-muted)' }}>{u.email}</div>
                    </div>
                  </div>
                </td>
                <td>
                  <select
                    value={u.plan}
                    onChange={e => onPlanChange(u.id, e.target.value)}
                    className="plan-select"
                  >
                    {PLANS.map(p => <option key={p} value={p}>{p}</option>)}
                  </select>
                </td>
                <td>
                  <span className={`status-badge ${u.is_active ? 'active' : 'inactive'}`}>
                    {u.is_active ? '● Active' : '○ Inactive'}
                  </span>
                </td>
                <td>
                  <span style={{ color: u.email_verified ? 'var(--success)' : 'var(--warning)', fontSize: 12 }}>
                    {u.email_verified ? '✓ Yes' : '✗ No'}
                  </span>
                </td>
                <td style={{ textAlign: 'center' }}>{u.active_keys}</td>
                <td style={{ fontSize: 11, color: 'var(--text-muted)' }}>
                  {new Date(u.created_at).toLocaleDateString()}
                </td>
                <td>
                  <button
                    className={`btn-sm ${u.is_active ? 'btn-sm-danger' : 'btn-sm-success'}`}
                    onClick={() => onToggle(u.id)}
                    disabled={loading}
                  >
                    {u.is_active ? 'Deactivate' : 'Activate'}
                  </button>
                </td>
              </tr>
            ))}
            {filtered.length === 0 && (
              <tr><td colSpan={7} style={{ textAlign: 'center', color: 'var(--text-muted)', padding: 24 }}>No users found</td></tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  )
}

export default function AdminPage() {
  const [stats, setStats] = useState(null)
  const [users, setUsers] = useState([])
  const [actionLoading, setActionLoading] = useState(false)
  const [toast, setToast] = useState('')
  const [activeTab, setActiveTab] = useState('live')

  const showToast = (msg) => { setToast(msg); setTimeout(() => setToast(''), 3000) }

  const loadData = async () => {
    const [s, u] = await Promise.all([api.adminStats(), api.adminUsers()])
    if (s.total_users !== undefined) setStats(s)
    if (u.users) setUsers(u.users)
  }

  useEffect(() => { loadData() }, [])

  const handlePlanChange = async (userId, plan) => {
    setActionLoading(true)
    const res = await api.adminSetPlan(userId, plan)
    if (!res.error) {
      setUsers(prev => prev.map(u => u.id === userId ? { ...u, plan } : u))
      showToast(`Plan updated to ${plan}`)
    } else {
      showToast(res.error)
    }
    setActionLoading(false)
  }

  const handleToggle = async (userId) => {
    setActionLoading(true)
    const res = await api.adminToggleUser(userId)
    if (!res.error) {
      setUsers(prev => prev.map(u => u.id === userId ? { ...u, is_active: res.is_active } : u))
      showToast(res.message)
    }
    setActionLoading(false)
  }

  return (
    <div className="page admin-page">
      <div className="page-header" style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div>
          <h2>⚡ Admin Dashboard</h2>
          <p>Manage users · Monitor live data feeds</p>
        </div>
        <button className="btn-outline" onClick={loadData} style={{ fontSize: 13 }}>↻ Refresh</button>
      </div>

      {toast && <div className="toast success">{toast}</div>}

      {/* Stats Strip */}
      {stats && (
        <div className="admin-stats-strip">
          {[
            { label: 'Total Users', value: stats.total_users, icon: '👥' },
            { label: 'Active Users', value: stats.active_users, icon: '✅' },
            { label: 'API Keys', value: stats.total_api_keys, icon: '🔑' },
            ...Object.entries(stats.users_by_plan || {}).map(([plan, count]) => ({
              label: plan.charAt(0).toUpperCase() + plan.slice(1),
              value: count,
              icon: plan === 'enterprise' ? '💎' : plan === 'pro' ? '⭐' : plan === 'basic' ? '🔹' : '○'
            }))
          ].map((s, i) => (
            <div key={i} className="admin-stat">
              <span className="admin-stat-icon">{s.icon}</span>
              <div>
                <div className="admin-stat-value">{s.value}</div>
                <div className="admin-stat-label">{s.label}</div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Tab Navigation */}
      <div className="admin-tabs">
        <button className={`admin-tab ${activeTab === 'live' ? 'active' : ''}`} onClick={() => setActiveTab('live')}>
          📡 Live Feeds
        </button>
        <button className={`admin-tab ${activeTab === 'users' ? 'active' : ''}`} onClick={() => setActiveTab('users')}>
          👥 Users ({users.length})
        </button>
      </div>

      {activeTab === 'live' && (
        <div className="admin-feeds-grid">
          <PriceFeed />
          <XFeed />
          <NewsFeed />
        </div>
      )}

      {activeTab === 'users' && (
        <UsersTable
          users={users}
          onPlanChange={handlePlanChange}
          onToggle={handleToggle}
          loading={actionLoading}
        />
      )}
    </div>
  )
}
