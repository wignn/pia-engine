import { createContext, useContext, useState, useEffect } from 'react'
import { api } from '../api'

const AuthContext = createContext(null)

export function AuthProvider({ children }) {
  const [user, setUser] = useState(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    // Validate the HttpOnly session cookie before rendering authenticated state.
    api.me().then(data => {
      if (data.user) setUser(data)
      else {
        localStorage.removeItem('wi_api_key')
      }
      setLoading(false)
    }).catch(() => {
      setLoading(false)
    })
  }, [])

  const loginWithJwt = (token, userData) => {
    setUser(userData)
  }

  const loginWithCredentials = async (email, password) => {
    const data = await api.login(email, password)
    if (data.token && data.user) {
      // Refresh the full account profile after the browser stores the cookie.
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
    // Clear client-side credentials; the server session cookie expires separately.
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
