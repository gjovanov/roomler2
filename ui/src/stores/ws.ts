import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useMessageStore } from './messages'
import { useTaskStore } from './tasks'

type WsStatus = 'disconnected' | 'connecting' | 'connected'

export const useWsStore = defineStore('ws', () => {
  const status = ref<WsStatus>('disconnected')
  let socket: WebSocket | null = null
  let pingInterval: ReturnType<typeof setInterval> | null = null
  let reconnectTimeout: ReturnType<typeof setTimeout> | null = null

  function connect(token: string) {
    if (socket && socket.readyState <= WebSocket.OPEN) return

    status.value = 'connecting'
    const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:'
    socket = new WebSocket(`${protocol}//${location.host}/ws?token=${token}`)

    socket.onopen = () => {
      status.value = 'connected'
      pingInterval = setInterval(() => {
        if (socket?.readyState === WebSocket.OPEN) {
          socket.send(JSON.stringify({ type: 'ping' }))
        }
      }, 30_000)
    }

    socket.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data)
        handleMessage(msg)
      } catch {
        // ignore malformed messages
      }
    }

    socket.onclose = () => {
      cleanup()
      status.value = 'disconnected'
      reconnectTimeout = setTimeout(() => connect(token), 3000)
    }

    socket.onerror = () => {
      socket?.close()
    }
  }

  function handleMessage(msg: { type: string; data?: unknown }) {
    const messageStore = useMessageStore()
    const taskStore = useTaskStore()

    switch (msg.type) {
      case 'message:create':
        messageStore.addMessageFromWs(msg.data as never)
        break
      case 'task:update':
        taskStore.updateFromWs(msg.data as never)
        break
    }
  }

  function send(type: string, data: unknown) {
    if (socket?.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify({ type, data }))
    }
  }

  function sendTyping(channelId: string) {
    send('typing:start', { channel_id: channelId })
  }

  function cleanup() {
    if (pingInterval) {
      clearInterval(pingInterval)
      pingInterval = null
    }
    if (reconnectTimeout) {
      clearTimeout(reconnectTimeout)
      reconnectTimeout = null
    }
  }

  function disconnect() {
    cleanup()
    socket?.close()
    socket = null
    status.value = 'disconnected'
  }

  return { status, connect, disconnect, send, sendTyping }
})
