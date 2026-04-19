import { useEffect, useState } from 'react'
import { useNavigate, useParams, useSearchParams } from 'react-router-dom'
import { api } from '../api'
import { useAuth } from '../context/AuthContext'

export default function OAuthCallbackPage() {
  const { provider } = useParams()
  const [searchParams] = useSearchParams()
  const [error, setError] = useState('')
  const [status, setStatus] = useState('Authenticating...')
  const { loginWithJwt } = useAuth()
  const nav = useNavigate()

  useEffect(() => {
    const code = searchParams.get('code')
    const errorParam = searchParams.get('error')

    if (errorParam) {
      setError(`OAuth error: ${errorParam}`)
      setStatus('')
      return
    }

    if (!code) {
      setError('No authorization code received')
      setStatus('')
      return
    }

    setStatus(`Completing ${provider} login...`)

    api.oauthCallback(provider, code)
      .then(data => {
        if (data.error) {
          setError(data.error)
          setStatus('')
        } else if (data.token && data.user) {
          setStatus('Success! Redirecting...')
          loginWithJwt(data.token, { user: data.user })
          setTimeout(() => nav('/'), 500)
        } else {
          setError('Unexpected response from server')
          setStatus('')
        }
      })
      .catch(() => {
        setError('Connection failed')
        setStatus('')
      })
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  return (
    <div className="auth-page">
      <div className="auth-card" style={{ textAlign: 'center' }}>
        <div className="auth-header">
          <div className="brand-icon large">◈</div>
          <h1>
            {error ? 'Authentication Failed' : 'Signing You In'}
          </h1>
        </div>

        {status && !error && (
          <div className="oauth-loading">
            <div className="spinner" />
            <p>{status}</p>
          </div>
        )}

        {error && (
          <>
            <div className="error-msg">{error}</div>
            <button
              className="btn-primary full"
              onClick={() => nav('/login')}
              style={{ marginTop: '16px' }}
            >
              Back to Login
            </button>
          </>
        )}
      </div>
    </div>
  )
}
