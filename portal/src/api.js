const BASE = (import.meta.env.VITE_API_BASE || '/api/v1').replace(/\/$/, '')

function headers() {
  const h = { 'Content-Type': 'application/json' }
  // Prefer JWT over API key
  const jwt = localStorage.getItem('wi_jwt')
  if (jwt) {
    h['Authorization'] = `Bearer ${jwt}`
  } else {
    const key = localStorage.getItem('wi_api_key')
    if (key) h['X-API-Key'] = key
  }
  return h
}

async function request(method, path, body) {
  const res = await fetch(`${BASE}${path}`, {
    method,
    headers: headers(),
    body: body ? JSON.stringify(body) : undefined,
  })
  if (res.status === 401) {
    // Clear invalid tokens
    localStorage.removeItem('wi_jwt')
    localStorage.removeItem('wi_api_key')
    if (window.location.pathname !== '/login' && window.location.pathname !== '/register') {
      window.location.href = '/login'
    }
    return { error: 'Session expired' }
  }
  return res.json()
}

export const api = {
  // Auth
  login: (email, password) => request('POST', '/auth/login', { email, password }),
  register: (email, name, password) => request('POST', '/auth/register', { email, name, password }),
  verify: (token) => request('POST', '/auth/verify', { token }),
  me: () => request('GET', '/auth/me'),

  // OAuth
  getOAuthUrl: (provider) => request('GET', `/auth/oauth/${provider}/url`),
  oauthCallback: (provider, code) => request('POST', `/auth/oauth/${provider}/callback`, { code }),

  // Keys
  listKeys: () => request('GET', '/keys'),
  createKey: (label) => request('POST', '/keys', { label }),
  revokeKey: (id) => request('DELETE', `/keys/${id}`),

  // Config
  listConfig: () => request('GET', '/config'),
  setConfig: (key, value) => request('PUT', `/config/${key}`, { value }),
  deleteConfig: (key) => request('DELETE', `/config/${key}`),

  // Usage
  usage: () => request('GET', '/usage'),
  usageHistory: (days = 30) => request('GET', `/usage/history?days=${days}`),

  // Plans
  plans: () => request('GET', '/plans'),
  upgrade: (planId) => request('POST', '/plans/upgrade', { plan_id: planId }),
}
