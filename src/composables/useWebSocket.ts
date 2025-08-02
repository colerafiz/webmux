import { onMounted, onUnmounted, computed, ComputedRef } from 'vue'
import { wsManager } from '@/services/websocket'
import type { WsMessage } from '@/types'

type MessageHandler<T extends WsMessage = WsMessage> = (data: T) => void

export interface UseWebSocketReturn {
  isConnected: ComputedRef<boolean>
  send: (data: WsMessage) => void
  onMessage: <T extends WsMessage = WsMessage>(type: string, handler: MessageHandler<T>) => void
  offMessage: (type: string) => void
  ensureConnected: () => Promise<void>
}

export function useWebSocket(): UseWebSocketReturn {
  const isConnected = computed(() => wsManager.isConnected)
  const messageHandlers = new Map<string, MessageHandler>()

  const send = (data: WsMessage): void => {
    wsManager.send(data)
  }

  const onMessage = <T extends WsMessage = WsMessage>(type: string, handler: MessageHandler<T>): void => {
    messageHandlers.set(type, handler as MessageHandler)
    wsManager.onMessage(type, handler as MessageHandler)
  }

  const offMessage = (type: string): void => {
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