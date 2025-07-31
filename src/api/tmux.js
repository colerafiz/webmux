import axios from 'axios'

const API_BASE = '/api'

export const tmuxApi = {
  getSessions: async () => {
    const { data } = await axios.get(`${API_BASE}/sessions`)
    return data.sessions
  },

  createSession: async (name) => {
    const { data } = await axios.post(`${API_BASE}/sessions`, { name })
    return data
  },

  killSession: async (sessionName) => {
    const { data } = await axios.post(`${API_BASE}/sessions/${sessionName}/kill`)
    return data
  },

  renameSession: async (sessionName, newName) => {
    const { data } = await axios.post(`${API_BASE}/sessions/${sessionName}/rename`, { newName })
    return data
  },

  // Window management
  getWindows: async (sessionName) => {
    const { data } = await axios.get(`${API_BASE}/sessions/${sessionName}/windows`)
    return data.windows
  },

  createWindow: async (sessionName, windowName) => {
    const { data } = await axios.post(`${API_BASE}/sessions/${sessionName}/windows`, { windowName })
    return data
  },

  killWindow: async (sessionName, windowIndex) => {
    const { data } = await axios.delete(`${API_BASE}/sessions/${sessionName}/windows/${windowIndex}`)
    return data
  },

  renameWindow: async (sessionName, windowIndex, newName) => {
    const { data } = await axios.post(`${API_BASE}/sessions/${sessionName}/windows/${windowIndex}/rename`, { newName })
    return data
  }
}