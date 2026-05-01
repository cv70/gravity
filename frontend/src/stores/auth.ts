import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { MeResponse } from '@/types'

interface User {
  id: string
  email: string
  name: string
  role: string
}

interface AuthState {
  user: User | null
  organizationName: string | null
  accessToken: string | null
  refreshToken: string | null
  isAuthenticated: boolean
  login: (email: string, password: string, organizationName?: string) => Promise<void>
  register: (name: string, email: string, password: string, organizationName: string) => Promise<void>
  refreshSession: () => Promise<boolean>
  hydrateFromProfile: () => Promise<void>
  logout: () => void
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      organizationName: null,
      accessToken: null,
      refreshToken: null,
      isAuthenticated: false,

      login: async (email: string, password: string, organizationName?: string) => {
        const response = await fetch('/api/v1/auth/login', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ email, password, organization_name: organizationName }),
        })

        if (!response.ok) throw new Error('Login failed')

        const data = await response.json()
        set({
          user: data.data.user,
          organizationName: organizationName || null,
          accessToken: data.data.access_token,
          refreshToken: data.data.refresh_token,
          isAuthenticated: true,
        })
      },

      register: async (name: string, email: string, password: string, organizationName: string) => {
        const response = await fetch('/api/v1/auth/register', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ name, email, password, organization_name: organizationName }),
        })

        if (!response.ok) throw new Error('Register failed')

        const data = await response.json()
        set({
          user: data.data.user,
          organizationName,
          accessToken: data.data.access_token,
          refreshToken: data.data.refresh_token,
          isAuthenticated: true,
        })
      },

      refreshSession: async () => {
        const { refreshToken } = get()
        if (!refreshToken) return false

        try {
          const response = await fetch('/api/v1/auth/refresh', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ refresh_token: refreshToken }),
          })

          if (!response.ok) return false

          const data = await response.json()
          set({ accessToken: data.data.access_token, isAuthenticated: true })
          return true
        } catch {
          return false
        }
      },

      hydrateFromProfile: async () => {
        const { accessToken, isAuthenticated } = get()
        if (!accessToken || !isAuthenticated) return

        try {
          let response = await fetch('/api/v1/auth/me', {
            headers: { Authorization: `Bearer ${accessToken}` },
          })
          if (response.status === 401 && (await get().refreshSession())) {
            const refreshedToken = get().accessToken
            if (refreshedToken) {
              response = await fetch('/api/v1/auth/me', {
                headers: { Authorization: `Bearer ${refreshedToken}` },
              })
            }
          }
          if (!response.ok) return
          const data = await response.json()
          const profile = data.data as MeResponse
          set({
            user: profile.user,
            organizationName: profile.organization.name,
            isAuthenticated: true,
          })
        } catch {
          // Let the API interceptor handle invalid sessions on the next request.
        }
      },

      logout: () => {
        set({
          user: null,
          organizationName: null,
          accessToken: null,
          refreshToken: null,
          isAuthenticated: false,
        })
      },
    }),
    {
      name: 'gravity-auth',
    }
  )
)
