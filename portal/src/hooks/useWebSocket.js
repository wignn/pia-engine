import { useState, useEffect, useCallback, useRef } from 'react'
import { CORE_WS_BASE } from '../api'

export function useWebSocket(path, options = {}) {
  const { onMessage, autoConnect = true } = options
  const [status, setStatus] = useState('disconnected') // disconnected, connecting, connected, error
  const [error, setError] = useState(null)
  const ws = useRef(null)
  const reconnectCount = useRef(0)
  const maxReconnectDelay = 30000
  const baseDelay = 1000

  const connect = useCallback(() => {
    if (ws.current?.readyState === WebSocket.OPEN) return

    setStatus('connecting')
    
    // Auth: try API key from localStorage first, then fallback
    const apiKey = localStorage.getItem('wi_api_key') || ''
    const url = new URL(`${CORE_WS_BASE}${path}`)
    if (apiKey) {
      url.searchParams.set('api_key', apiKey)
    }
    // Add bot_id for backend tracking
    url.searchParams.set('bot_id', 'portal-web')

    const socket = new WebSocket(url.toString())
    ws.current = socket

    socket.onopen = () => {
      console.log(`WS connected: ${path}`)
      setStatus('connected')
      reconnectCount.current = 0
      setError(null)
    }

    socket.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data)
        if (onMessage) onMessage(data)
      } catch (e) {
        console.error('WS parse error:', e)
      }
    }

    socket.onclose = (event) => {
      if (event.wasClean) {
        setStatus('disconnected')
      } else {
        setStatus('error')
        // Attempt reconnect
        const delay = Math.min(baseDelay * Math.pow(2, reconnectCount.current), maxReconnectDelay)
        reconnectCount.current++
        console.log(`WS connection lost: ${path}. Retrying in ${delay}ms...`)
        setTimeout(connect, delay)
      }
    }

    socket.onerror = (err) => {
      setError(err)
      socket.close()
    }
  }, [path, onMessage])

  useEffect(() => {
    if (autoConnect) {
      connect()
    }
    return () => {
      if (ws.current) {
        ws.current.close(1000, 'Component unmounted')
      }
    }
  }, [connect, autoConnect])

  const sendMessage = useCallback((msg) => {
    if (ws.current?.readyState === WebSocket.OPEN) {
      ws.current.send(typeof msg === 'string' ? msg : JSON.stringify(msg))
    }
  }, [])

  return { status, error, sendMessage, reconnect: connect }
}
