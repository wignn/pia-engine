import { createContext, useContext, useState, useEffect } from 'react'
import { api } from '../api'

const AuthContext = createContext(null)

export function AuthProvider({ children }) {
  const [user, setUser] = useState(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const jwt = localStorage.getItem('wi_jwt')
    const key = localStorage.getItem('wi_api_key')
    if (!jwt && !key) { setLoading(false); return }
    api.me().then(data => {
      if (data.user) setUser(data)
      else {
        localStorage.removeItem('wi_jwt')
        localStorage.removeItem('wi_api_key')
      }
      setLoading(false)
    }).catch(() => {
      setLoading(false)
    })
  }, [])

  const loginWithJwt = (token, userData) => {
    localStorage.setItem('wi_jwt', token)
    setUser(userData)
  }

  const loginWithCredentials = async (email, password) => {
    const data = await api.login(email, password)
    if (data.token && data.user) {
      localStorage.setItem('wi_jwt', data.token)
      // Fetch full user info (plan_limits etc.)
      const me = await api.me()
      if (me.user) setUser(me)
      return { success: true }
    }
    return { success: false, error: data.error || 'Login failed' }
  }

  const loginWithApiKey = (apiKey) => {
    localStorage.setItem('wi_api_key', apiKey)
    return api.me().then(data => {
      if (data.user) setUser(data)
      return data
    })
  }

  const logout = () => {
    localStorage.removeItem('wi_jwt')
    localStorage.removeItem('wi_api_key')
    setUser(null)
  }

  return (
    <AuthContext.Provider value={{ user, setUser, loginWithJwt, loginWithCredentials, loginWithApiKey, logout, loading }}>
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth() {
  return useContext(AuthContext)
}
