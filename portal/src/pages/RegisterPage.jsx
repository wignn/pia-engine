import { useState } from 'react'
import { useNavigate, Link } from 'react-router-dom'
import { api } from '../api'
import { useAuth } from '../context/AuthContext'

export default function RegisterPage() {
  const [email, setEmail] = useState('')
  const [name, setName] = useState('')
  const [password, setPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [result, setResult] = useState(null)
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)
  const { loginWithJwt } = useAuth()
  const nav = useNavigate()

  const passwordStrength = (pw) => {
    if (!pw) return { level: 0, label: '', cls: '' }
    let score = 0
    if (pw.length >= 6) score++
    if (pw.length >= 10) score++
    if (/[A-Z]/.test(pw)) score++
    if (/[0-9]/.test(pw)) score++
    if (/[^A-Za-z0-9]/.test(pw)) score++
    if (score <= 1) return { level: 1, label: 'Weak', cls: 'weak' }
    if (score <= 3) return { level: 2, label: 'Medium', cls: 'medium' }
    return { level: 3, label: 'Strong', cls: 'strong' }
  }

  const strength = passwordStrength(password)

  const handleSubmit = async (e) => {
    e.preventDefault()
    if (!email.trim() || !name.trim() || !password) return
    if (password.length < 6) {
      setError('Password must be at least 6 characters')
      return
    }
    if (password !== confirmPassword) {
      setError('Passwords do not match')
      return
    }
    setLoading(true); setError(''); setResult(null)
    try {
      const data = await api.register(email.trim(), name.trim(), password)
      if (data.error) { setError(data.error) }
      else { setResult(data) }
    } catch { setError('Connection failed') }
    setLoading(false)
  }

  const handleContinue = async () => {
    if (result?.token) {
      loginWithJwt(result.token, { user: result.user })
      nav('/')
    }
  }

  const handleOAuth = async (provider) => {
    try {
      const data = await api.getOAuthUrl(provider)
      if (data.url) {
        window.location.href = data.url
      } else {
        setError(data.error || `${provider} signup not available`)
      }
    } catch {
      setError('Failed to start OAuth flow')
    }
  }

  return (
    <div className="auth-page">
      <div className="auth-card">
        <div className="auth-header">
          <div className="brand-icon large">◈</div>
          <h1>Create Account</h1>
          <p>Start building with World Info API</p>
        </div>

        {!result ? (
          <>
            {/* OAuth Buttons */}
            <div className="oauth-buttons">
              <button
                type="button"
                className="btn-oauth btn-google"
                onClick={() => handleOAuth('google')}
              >
                <svg className="oauth-icon" viewBox="0 0 24 24" width="18" height="18">
                  <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 01-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z"/>
                  <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                  <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
                  <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
                </svg>
                Sign up with Google
              </button>
              <button
                type="button"
                className="btn-oauth btn-github"
                onClick={() => handleOAuth('github')}
              >
                <svg className="oauth-icon" viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                  <path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"/>
                </svg>
                Sign up with GitHub
              </button>
            </div>

            <div className="auth-divider">
              <span>or register with email</span>
            </div>

            <form onSubmit={handleSubmit}>
              <div className="form-group">
                <label htmlFor="reg-name">Name</label>
                <input
                  id="reg-name"
                  type="text"
                  placeholder="Your name"
                  value={name}
                  onChange={e => setName(e.target.value)}
                  autoFocus
                  autoComplete="name"
                />
              </div>
              <div className="form-group">
                <label htmlFor="reg-email">Email</label>
                <input
                  id="reg-email"
                  type="email"
                  placeholder="you@email.com"
                  value={email}
                  onChange={e => setEmail(e.target.value)}
                  autoComplete="email"
                />
              </div>
              <div className="form-group">
                <label htmlFor="reg-password">Password</label>
                <input
                  id="reg-password"
                  type="password"
                  placeholder="Min. 6 characters"
                  value={password}
                  onChange={e => setPassword(e.target.value)}
                  autoComplete="new-password"
                />
                {password && (
                  <div className="password-strength">
                    <div className={`strength-bar ${strength.cls}`}>
                      <div className="strength-fill" style={{ width: `${(strength.level / 3) * 100}%` }} />
                    </div>
                    <span className={`strength-label ${strength.cls}`}>{strength.label}</span>
                  </div>
                )}
              </div>
              <div className="form-group">
                <label htmlFor="reg-confirm">Confirm Password</label>
                <input
                  id="reg-confirm"
                  type="password"
                  placeholder="Repeat your password"
                  value={confirmPassword}
                  onChange={e => setConfirmPassword(e.target.value)}
                  autoComplete="new-password"
                />
              </div>
              {error && <div className="error-msg">{error}</div>}
              <button type="submit" className="btn-primary full" disabled={loading}>
                {loading ? 'Creating...' : 'Create Account'}
              </button>
            </form>
          </>
        ) : (
          <div className="register-success">
            <div className="success-icon">✓</div>
            <h3>Registration Successful!</h3>
            {result.api_key && (
              <>
                <p className="key-warning">Save your API key — it will only be shown once:</p>
                <div className="key-display">
                  <code>{result.api_key}</code>
                  <button className="btn-copy" onClick={() => navigator.clipboard.writeText(result.api_key)}>
                    Copy
                  </button>
                </div>
              </>
            )}
            {result.verify_token && (
              <p className="verify-note">
                Verification token: <code>{result.verify_token}</code>
              </p>
            )}
            <button className="btn-primary full" onClick={handleContinue}>
              Continue to Dashboard
            </button>
          </div>
        )}
        <p className="auth-footer">
          Already have an account? <Link to="/login">Sign In</Link>
        </p>
      </div>
    </div>
  )
}
