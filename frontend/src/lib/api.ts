import { writable, type Writable } from 'svelte/store';
import type { User } from '$lib/types';

const API_URL = typeof window !== 'undefined' 
  ? (import.meta.env.VITE_API_URL || 'http://localhost:8080')
  : 'http://backend:8080';

export interface AuthState {
  token: string | null;
  user: User | null;
}

function createAuthStore() {
  const stored = typeof localStorage !== 'undefined' ? localStorage.getItem('auth') : null;
  const initial: AuthState = stored ? JSON.parse(stored) : { token: null, user: null };

  const { subscribe, set, update }: Writable<AuthState> = writable(initial);

  return {
    subscribe,
    login: (token: string, user: User) => {
      const state = { token, user };
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem('auth', JSON.stringify(state));
      }
      set(state);
    },
    logout: () => {
      if (typeof localStorage !== 'undefined') {
        localStorage.removeItem('auth');
      }
      set({ token: null, user: null });
    },
    refresh: async () => {
      const current = stored ? JSON.parse(stored) : { token: null, user: null };
      if (current.token) {
        try {
          const response = await api.get('/api/auth/me');
          if (response.ok) {
            const user = await response.json();
            set({ token: current.token, user });
          }
        } catch (e) {
          set({ token: null, user: null });
        }
      }
    }
  };
}

export const auth = createAuthStore();

export async function request(
  method: 'GET' | 'POST' | 'PUT' | 'DELETE',
  endpoint: string,
  data?: unknown,
  authToken?: string | null
): Promise<Response> {
  const url = `${API_URL}${endpoint}`;
  
  const headers: HeadersInit = {
    'Content-Type': 'application/json'
  };

  if (authToken) {
    headers['Authorization'] = `Bearer ${authToken}`;
  }

  const options: RequestInit = {
    method,
    headers,
    credentials: 'same-origin'
  };

  if (data && (method === 'POST' || method === 'PUT')) {
    options.body = JSON.stringify(data);
  }

  return fetch(url, options);
}

export const api = {
  get: (endpoint: string, token?: string | null) => request('GET', endpoint, undefined, token),
  post: (endpoint: string, data: unknown, token?: string | null) => request('POST', endpoint, data, token),
  put: (endpoint: string, data: unknown, token?: string | null) => request('PUT', endpoint, data, token),
  delete: (endpoint: string, token?: string | null) => request('DELETE', endpoint, undefined, token)
};
