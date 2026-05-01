import axios, { type AxiosError, type InternalAxiosRequestConfig } from 'axios'

import { useAuthStore } from '@/stores/auth'

const api = axios.create({
  baseURL: '/api/v1',
  headers: { 'Content-Type': 'application/json' },
})

api.interceptors.request.use((config: InternalAxiosRequestConfig) => {
  const token = useAuthStore.getState().accessToken
  if (token) {
    config.headers.set('Authorization', `Bearer ${token}`)
  }
  return config
})

type RetryConfig = InternalAxiosRequestConfig & { _retry?: boolean }

api.interceptors.response.use(
  (response) => response,
  async (error: AxiosError) => {
    const originalConfig = error.config as RetryConfig | undefined
    if (!originalConfig || error.response?.status !== 401 || originalConfig._retry) {
      if (error.response?.status === 401) {
        useAuthStore.getState().logout()
        window.location.href = '/login'
      }
      return Promise.reject(error)
    }

    originalConfig._retry = true
    const refreshed = await useAuthStore.getState().refreshSession()
    if (!refreshed) {
      useAuthStore.getState().logout()
      window.location.href = '/login'
      return Promise.reject(error)
    }

    const token = useAuthStore.getState().accessToken
    if (token) {
      originalConfig.headers.set('Authorization', `Bearer ${token}`)
    }

    return api(originalConfig)
  }
)

export default api
