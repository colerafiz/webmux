import axios from 'axios'
import type { 
  TmuxSession, 
  TmuxWindow, 
  SessionCreateResponse, 
  SessionActionResponse, 
  WindowsListResponse,
  WindowCreateResponse 
} from '@/types'

const API_BASE = '/api'

export const tmuxApi = {
  async getSessions(): Promise<TmuxSession[]> {
    const { data } = await axios.get<{ sessions: TmuxSession[] }>(`${API_BASE}/sessions`)
    return data.sessions
  },

  async createSession(name: string): Promise<SessionCreateResponse> {
    const { data } = await axios.post<SessionCreateResponse>(`${API_BASE}/sessions`, { name })
    return data
  },

  async killSession(sessionName: string): Promise<SessionActionResponse> {
    const { data } = await axios.post<SessionActionResponse>(`${API_BASE}/sessions/${sessionName}/kill`)
    return data
  },

  async renameSession(sessionName: string, newName: string): Promise<SessionActionResponse> {
    const { data } = await axios.post<SessionActionResponse>(`${API_BASE}/sessions/${sessionName}/rename`, { newName })
    return data
  },

  // Window management
  async getWindows(sessionName: string): Promise<TmuxWindow[]> {
    const { data } = await axios.get<WindowsListResponse>(`${API_BASE}/sessions/${sessionName}/windows`)
    return data.windows
  },

  async createWindow(sessionName: string, windowName?: string): Promise<WindowCreateResponse> {
    const { data } = await axios.post<WindowCreateResponse>(`${API_BASE}/sessions/${sessionName}/windows`, { windowName })
    return data
  },

  async killWindow(sessionName: string, windowIndex: number): Promise<SessionActionResponse> {
    const { data } = await axios.delete<SessionActionResponse>(`${API_BASE}/sessions/${sessionName}/windows/${windowIndex}`)
    return data
  },

  async renameWindow(sessionName: string, windowIndex: number, newName: string): Promise<SessionActionResponse> {
    const { data } = await axios.post<SessionActionResponse>(`${API_BASE}/sessions/${sessionName}/windows/${windowIndex}/rename`, { newName })
    return data
  }
}