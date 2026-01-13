import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import api from '../lib/api';

interface Zone {
  id: string;
  name: string;
  status: string;
  plan?: {
    name: string;
  };
}

interface CloudflareState {
  isConnected: boolean;
  isLoading: boolean;
  isUnderAttack: boolean;
  zone: Zone | null;
  error: string | null;

  // Actions
  checkConnection: () => Promise<void>;
  setUnderAttack: (enabled: boolean) => Promise<void>;
  fetchZone: () => Promise<void>;
  purgeCache: (urls?: string[]) => Promise<void>;
  purgeAllCache: () => Promise<void>;
}

export const useCloudflareStore = create<CloudflareState>()(
  persist(
    (set, get) => ({
      isConnected: false,
      isLoading: false,
      isUnderAttack: false,
      zone: null,
      error: null,

      checkConnection: async () => {
        set({ isLoading: true, error: null });
        try {
          const response = await api.get('/cloudflare/status');
          set({
            isConnected: response.data.data.connected,
            isLoading: false,
          });
        } catch (error: any) {
          set({
            isConnected: false,
            isLoading: false,
            error: error.message,
          });
        }
      },

      fetchZone: async () => {
        set({ isLoading: true });
        try {
          const response = await api.get('/cloudflare/zone');
          set({
            zone: response.data.data,
            isLoading: false,
          });
        } catch (error: any) {
          set({ isLoading: false, error: error.message });
        }
      },

      setUnderAttack: async (enabled: boolean) => {
        set({ isLoading: true });
        try {
          await api.post('/cloudflare/security/under-attack', { enabled });
          set({ isUnderAttack: enabled, isLoading: false });
        } catch (error: any) {
          set({ isLoading: false, error: error.message });
          throw error;
        }
      },

      purgeCache: async (urls?: string[]) => {
        set({ isLoading: true });
        try {
          await api.post('/cloudflare/cache/purge', { urls });
          set({ isLoading: false });
        } catch (error: any) {
          set({ isLoading: false, error: error.message });
          throw error;
        }
      },

      purgeAllCache: async () => {
        set({ isLoading: true });
        try {
          await api.post('/cloudflare/cache/purge/all');
          set({ isLoading: false });
        } catch (error: any) {
          set({ isLoading: false, error: error.message });
          throw error;
        }
      },
    }),
    {
      name: 'cloudflare-storage',
      partialize: (state) => ({
        isConnected: state.isConnected,
        zone: state.zone,
      }),
    }
  )
);
