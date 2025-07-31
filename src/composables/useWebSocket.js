import { ref, onMounted, onUnmounted, computed } from 'vue'
import { wsManager } from '../services/websocket'

export function useWebSocket() {
  const isConnected = computed(() => wsManager.isConnected)
  const messageHandlers = new Map()

  const send = (data) => {
    wsManager.send(data)
  }

  const onMessage = (type, handler) => {
    messageHandlers.set(type, handler)
    wsManager.onMessage(type, handler)
  }

  const offMessage = (type) => {
    const handler = messageHandlers.get(type)
    if (handler) {
      wsManager.offMessage(type, handler)
      messageHandlers.delete(type)
    }
  }

  onMounted(() => {
    wsManager.connect()
  })

  onUnmounted(() => {
    // Remove all handlers for this component
    messageHandlers.forEach((handler, type) => {
      wsManager.offMessage(type, handler)
    })
    messageHandlers.clear()
  })

  return {
    isConnected,
    send,
    onMessage,
    offMessage,
    ensureConnected: () => wsManager.ensureConnected()
  }
}