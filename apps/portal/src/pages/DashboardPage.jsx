import { useState, useEffect } from 'react'
import { api } from '../api'
import { useAuth } from '../context/AuthContext'

export default function DashboardPage() {
  const { user } = useAuth()
  const [usage, setUsage] = useState(null)
  const [history, setHistory] = useState([])

  useEffect(() => {
    api.usage().then(setUsage).catch(() => {})
    api.usageHistory(14).then(d => setHistory(d.history || [])).catch(() => {})
  }, [])

  const plan = user?.plan_limits
  const u = user?.user

  return (
    <div className="page">
      <div className="page-header">
        <h2>Dashboard</h2>
        <p>Welcome back, {u?.name || 'Developer'}</p>
      </div>

      <div className="stats-grid">
        <div className="stat-card">
          <div className="stat-label">Today's Requests</div>
          <div className="stat-value">{usage?.today ?? '—'}</div>
          <div className="stat-sub">/ {usage?.daily_limit ?? '—'} limit</div>
          {usage && <div className="stat-bar">
            <div className="stat-bar-fill" style={{width: `${Math.min(100, (usage.today / usage.daily_limit) * 100)}%`}} />
          </div>}
        </div>
        <div className="stat-card">
          <div className="stat-label">This Week</div>
          <div className="stat-value">{usage?.this_week ?? '—'}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">This Month</div>
          <div className="stat-value">{usage?.this_month ?? '—'}</div>
        </div>
        <div className="stat-card accent">
          <div className="stat-label">Current Plan</div>
          <div className="stat-value">{u?.plan?.toUpperCase() || 'FREE'}</div>
          <div className="stat-sub">{user?.active_keys || 0} active keys</div>
        </div>
      </div>

      {plan && (
        <div className="card">
          <h3>Plan Limits</h3>
          <div className="limits-grid">
            <div className="limit-item"><span>Requests/day</span><strong>{plan.requests_per_day?.toLocaleString()}</strong></div>
            <div className="limit-item"><span>WS Connections</span><strong>{plan.ws_connections}</strong></div>
            <div className="limit-item"><span>X Usernames</span><strong>{plan.x_usernames_max}</strong></div>
            <div className="limit-item"><span>TV Symbols</span><strong>{plan.tv_symbols_max}</strong></div>
            <div className="limit-item"><span>News History</span><strong>{plan.news_history_days} days</strong></div>
            <div className="limit-item"><span>Rate Limit</span><strong>{plan.rate_limit_per_min}/min</strong></div>
          </div>
        </div>
      )}

      {history.length > 0 && (
        <div className="card">
          <h3>Recent Usage (14 days)</h3>
          <div className="usage-chart">
            {history.map(d => {
              const max = Math.max(...history.map(h => h.count), 1)
              return (
                <div key={d.day} className="chart-bar-wrap">
                  <div className="chart-bar" style={{height: `${(d.count / max) * 100}%`}} />
                  <span className="chart-label">{d.day.slice(5)}</span>
                </div>
              )
            })}
          </div>
        </div>
      )}
    </div>
  )
}
