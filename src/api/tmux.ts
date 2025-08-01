import axios from 'axios'
import type { 
  TmuxSession, 
  TmuxWindow, 
  SessionCreateResponse, 
  SessionActionResponse, 
  WindowsListResponse,
  WindowCreateResponse 
} from '@/types'

// Always use relative path - let Vite proxy handle everything
const API_BASE = '/api'

export const tmuxApi = {
  async getSessions(): Promise<TmuxSession[]> {
    const { data } = await axios.get<{ sessions: TmuxSession[] }>(`${API_BASE}/sessions`)
    return data.sessions
  },

  async createSession(name: string): Promise<SessionCreateResponse> {
    try {
      console.log('Current location:', window.location.href)
      console.log('API_BASE:', API_BASE)
      const fullUrl = `${window.location.origin}${API_BASE}/sessions`
      console.log('Full URL will be:', fullUrl)
      console.log('Making POST request to:', `${API_BASE}/sessions`, 'with data:', { name })
      
      const response = await axios.post<SessionCreateResponse>(`${API_BASE}/sessions`, { name })
      console.log('Response received:', response.data)
      return response.data
    } catch (error) {
      console.error('API request failed:', error)
      if (axios.isAxiosError(error)) {
        console.error('Response status:', error.response?.status)
        console.error('Response data:', error.response?.data)
        console.error('Request URL:', error.config?.url)
        console.error('Request baseURL:', error.config?.baseURL)
        console.error('Full failed URL:', error.config?.baseURL ? error.config.baseURL + error.config.url : error.config?.url)
      }
      throw error
    }
  },

  async killSession(sessionName: string): Promise<SessionActionResponse> {
    const { data } = await axios.post<SessionActionResponse>(`${API_BASE}/sessions/${encodeURIComponent(sessionName)}/kill`)
    return data
  },

  async renameSession(sessionName: string, newName: string): Promise<SessionActionResponse> {
    const { data } = await axios.post<SessionActionResponse>(`${API_BASE}/sessions/${encodeURIComponent(sessionName)}/rename`, { newName })
    return data
  },

  // Window management
  async getWindows(sessionName: string): Promise<TmuxWindow[]> {
    const { data } = await axios.get<WindowsListResponse>(`${API_BASE}/sessions/${encodeURIComponent(sessionName)}/windows`)
    return data.windows
  },

  async createWindow(sessionName: string, windowName?: string): Promise<WindowCreateResponse> {
    const { data } = await axios.post<WindowCreateResponse>(`${API_BASE}/sessions/${encodeURIComponent(sessionName)}/windows`, { windowName })
    return data
  },

  async killWindow(sessionName: string, windowIndex: number): Promise<SessionActionResponse> {
    const { data } = await axios.delete<SessionActionResponse>(`${API_BASE}/sessions/${encodeURIComponent(sessionName)}/windows/${windowIndex}`)
    return data
  },

  async renameWindow(sessionName: string, windowIndex: number, newName: string): Promise<SessionActionResponse> {
    const { data } = await axios.post<SessionActionResponse>(`${API_BASE}/sessions/${encodeURIComponent(sessionName)}/windows/${windowIndex}/rename`, { newName })
    return data
  }
}