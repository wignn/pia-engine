import { useState, useEffect } from 'react'
import { api } from '../api'
import { useAuth } from '../context/AuthContext'

export default function PlansPage() {
  const { user } = useAuth()
  const [plans, setPlans] = useState([])

  useEffect(() => {
    api.plans().then(d => setPlans(d.plans || []))
  }, [])

  const currentPlan = user?.user?.plan || 'free'

  const formatPrice = (idr) => {
    if (idr === 0) return 'Free'
    return `Rp${(idr).toLocaleString('id-ID')}/mo`
  }

  const handleUpgrade = async (planId) => {
    const res = await api.upgrade(planId)
    alert(res.message || 'Please contact admin')
  }

  return (
    <div className="page">
      <div className="page-header">
        <h2>Plans & Pricing</h2>
        <p>Choose the right plan for your needs</p>
      </div>

      <div className="plans-grid">
        {plans.map(plan => (
          <div key={plan.id} className={`plan-card ${plan.id === currentPlan ? 'current' : ''} ${plan.id === 'pro' ? 'featured' : ''}`}>
            {plan.id === 'pro' && <div className="plan-badge">Most Popular</div>}
            {plan.id === currentPlan && <div className="plan-badge current-badge">Current</div>}
            <h3>{plan.name}</h3>
            <div className="plan-price">{formatPrice(plan.price_idr)}</div>
            <ul className="plan-features">
              <li>{plan.requests_per_day.toLocaleString()} requests/day</li>
              <li>{plan.ws_connections} WebSocket connections</li>
              <li>{plan.x_usernames_max} X usernames</li>
              <li>{plan.tv_symbols_max} TV symbols</li>
              <li>{plan.news_history_days} days news history</li>
              <li>{plan.rate_limit_per_min}/min rate limit</li>
              <li>{plan.can_scrape ? '✓' : '✗'} Article scraping</li>
              <li>{plan.can_custom_rss ? '✓' : '✗'} Custom RSS feeds</li>
            </ul>
            {plan.id !== currentPlan && plan.id !== 'enterprise' && (
              <button className="btn-primary full" onClick={() => handleUpgrade(plan.id)}>
                Upgrade to {plan.name}
              </button>
            )}
            {plan.id === 'enterprise' && plan.id !== currentPlan && (
              <button className="btn-outline full" onClick={() => handleUpgrade(plan.id)}>
                Contact Sales
              </button>
            )}
          </div>
        ))}
      </div>
    </div>
  )
}
