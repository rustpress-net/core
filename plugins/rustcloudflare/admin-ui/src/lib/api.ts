import axios, { AxiosError, InternalAxiosRequestConfig } from 'axios';
import { findMockHandler, isMockEnabled } from './mockApi';

const api = axios.create({
  baseURL: '/api/v1',
  headers: {
    'Content-Type': 'application/json',
  },
  timeout: 30000,
});

// Request interceptor for auth token
api.interceptors.request.use((config) => {
  const authStorage = localStorage.getItem('auth-storage');
  if (authStorage) {
    try {
      const { state } = JSON.parse(authStorage);
      if (state?.token) {
        config.headers.Authorization = `Bearer ${state.token}`;
      }
    } catch (e) {
      // Ignore parse errors
    }
  }
  return config;
});

// Response interceptor for mock data in development
api.interceptors.response.use(
  (response) => response,
  async (error: AxiosError) => {
    // In development mode, try to use mock data when backend is unavailable
    if (isMockEnabled() && error.config) {
      const config = error.config as InternalAxiosRequestConfig;
      const url = config.url || '';
      const method = config.method || 'get';

      const mockHandler = findMockHandler(method, url);
      if (mockHandler) {
        console.log(`[Mock API] ${method.toUpperCase()} ${url}`);
        return mockHandler();
      }
    }

    if (error.response?.status === 401) {
      // Handle unauthorized - redirect to login
      window.location.href = '/admin/login';
    }
    return Promise.reject(error);
  }
);

export default api;

// API functions
export const cloudflareApi = {
  // Status & Connection
  getStatus: () => api.get('/cloudflare/status'),
  getZone: () => api.get('/cloudflare/zone'),
  getConnectionStatus: () => api.get('/cloudflare/connection'),
  testConnection: () => api.post('/cloudflare/connection/test'),

  // SSO Authentication
  getSsoAuthUrl: () => api.get('/cloudflare/auth/url'),
  ssoComplete: (accessToken: string, accountId?: string, zoneId?: string) =>
    api.post('/cloudflare/auth/sso-complete', { access_token: accessToken, account_id: accountId, zone_id: zoneId }),
  verifyToken: (apiToken: string) => api.post('/cloudflare/auth/verify-token', { api_token: apiToken }),
  saveCredentials: (apiToken: string, accountId: string, zoneId: string) =>
    api.post('/cloudflare/auth/save-credentials', { api_token: apiToken, account_id: accountId, zone_id: zoneId }),
  disconnect: () => api.post('/cloudflare/auth/disconnect'),
  listAccounts: (apiToken: string) => api.post('/cloudflare/auth/accounts', { api_token: apiToken }),
  listZones: (apiToken: string, accountId?: string) =>
    api.post('/cloudflare/auth/zones', { api_token: apiToken, account_id: accountId }),

  // Cache
  purgeAll: () => api.post('/cloudflare/cache/purge/all'),
  purgeUrls: (urls: string[]) => api.post('/cloudflare/cache/purge', { urls }),
  purgeTags: (tags: string[]) => api.post('/cloudflare/cache/purge/tags', { tags }),
  purgePrefix: (prefix: string) => api.post('/cloudflare/cache/purge/prefix', { prefix }),
  getCacheStats: () => api.get('/cloudflare/cache/status'),
  warmCache: (urls: string[]) => api.post('/cloudflare/cache/warm', { urls }),
  clearLocalCache: () => api.post('/cloudflare/cache/local/clear'),

  // DNS
  listDnsRecords: () => api.get('/cloudflare/dns/records'),
  createDnsRecord: (record: any) => api.post('/cloudflare/dns/records', record),
  updateDnsRecord: (id: string, record: any) => api.put(`/cloudflare/dns/records/${id}`, record),
  deleteDnsRecord: (id: string) => api.delete(`/cloudflare/dns/records/${id}`),
  exportZone: () => api.get('/cloudflare/dns/export'),
  importZone: (zoneFile: string) => api.post('/cloudflare/dns/import', { zone_file: zoneFile }),
  syncDnsRecords: () => api.post('/cloudflare/dns/sync'),

  // Security
  getSecurityLevel: () => api.get('/cloudflare/security/level'),
  setSecurityLevel: (level: string) => api.put('/cloudflare/security/level', { level }),
  toggleUnderAttack: (enabled: boolean) => api.post('/cloudflare/security/under-attack', { enabled }),
  listFirewallRules: () => api.get('/cloudflare/security/firewall/rules'),
  createFirewallRule: (rule: any) => api.post('/cloudflare/security/firewall/rules', rule),
  updateFirewallRule: (id: string, rule: any) => api.put(`/cloudflare/security/firewall/rules/${id}`, rule),
  deleteFirewallRule: (id: string) => api.delete(`/cloudflare/security/firewall/rules/${id}`),
  listIpAccessRules: () => api.get('/cloudflare/security/ip-access/rules'),
  blockIp: (ip: string, note?: string) => api.post('/cloudflare/security/ip-access/block', { ip, note }),
  allowIp: (ip: string, note?: string) => api.post('/cloudflare/security/ip-access/allow', { ip, note }),
  challengeIp: (ip: string, note?: string) => api.post('/cloudflare/security/ip-access/challenge', { ip, note }),
  deleteIpRule: (id: string) => api.delete(`/cloudflare/security/ip-access/rules/${id}`),
  getSecurityEvents: (limit?: number) => api.get('/cloudflare/security/events', { params: { limit } }),
  getWafRules: () => api.get('/cloudflare/security/waf/rules'),
  updateWafRule: (id: string, enabled: boolean) => api.patch(`/cloudflare/security/waf/rules/${id}`, { enabled }),

  // SSL
  getSslStatus: () => api.get('/cloudflare/ssl/status'),
  setSslMode: (mode: string) => api.put('/cloudflare/ssl/mode', { mode }),
  listCertificates: () => api.get('/cloudflare/ssl/certificates'),
  orderCertificate: (hosts: string[]) => api.post('/cloudflare/ssl/certificates', { hosts }),
  uploadCertificate: (certificate: string, privateKey: string) =>
    api.post('/cloudflare/ssl/certificates/custom', { certificate, private_key: privateKey }),

  // Workers
  listWorkers: () => api.get('/cloudflare/workers'),
  getWorker: (name: string) => api.get(`/cloudflare/workers/${name}`),
  deployWorker: (name: string, script: string) => api.post('/cloudflare/workers', { name, script }),
  updateWorker: (name: string, script: string) => api.put(`/cloudflare/workers/${name}`, { script }),
  deleteWorker: (name: string) => api.delete(`/cloudflare/workers/${name}`),
  listWorkerRoutes: () => api.get('/cloudflare/workers/routes'),
  createWorkerRoute: (pattern: string, script: string) =>
    api.post('/cloudflare/workers/routes', { pattern, script }),
  deleteWorkerRoute: (id: string) => api.delete(`/cloudflare/workers/routes/${id}`),
  deployTemplate: (templateId: string, name: string) =>
    api.post('/cloudflare/workers/templates/deploy', { template_id: templateId, name }),

  // KV Storage
  listKVNamespaces: () => api.get('/cloudflare/workers/kv/namespaces'),
  createKVNamespace: (title: string) => api.post('/cloudflare/workers/kv/namespaces', { title }),
  deleteKVNamespace: (id: string) => api.delete(`/cloudflare/workers/kv/namespaces/${id}`),
  listKVKeys: (namespaceId: string, prefix?: string) =>
    api.get(`/cloudflare/workers/kv/namespaces/${namespaceId}/keys`, { params: { prefix } }),
  getKVValue: (namespaceId: string, key: string) =>
    api.get(`/cloudflare/workers/kv/namespaces/${namespaceId}/values/${key}`),
  putKVValue: (namespaceId: string, key: string, value: string) =>
    api.put(`/cloudflare/workers/kv/namespaces/${namespaceId}/values/${key}`, { value }),
  deleteKVValue: (namespaceId: string, key: string) =>
    api.delete(`/cloudflare/workers/kv/namespaces/${namespaceId}/values/${key}`),

  // R2 Storage
  listR2Buckets: () => api.get('/cloudflare/r2/buckets'),
  createR2Bucket: (name: string) => api.post('/cloudflare/r2/buckets', { name }),
  deleteR2Bucket: (name: string) => api.delete(`/cloudflare/r2/buckets/${name}`),
  listR2Objects: (bucket: string, prefix?: string) =>
    api.get(`/cloudflare/r2/buckets/${bucket}/objects`, { params: { prefix } }),
  uploadR2Object: (bucket: string, key: string, file: File) => {
    const formData = new FormData();
    formData.append('file', file);
    formData.append('key', key);
    return api.post(`/cloudflare/r2/buckets/${bucket}/objects`, formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    });
  },
  deleteR2Object: (bucket: string, key: string) =>
    api.delete(`/cloudflare/r2/buckets/${bucket}/objects/${encodeURIComponent(key)}`),
  getR2ObjectUrl: (bucket: string, key: string) =>
    api.get(`/cloudflare/r2/buckets/${bucket}/objects/${encodeURIComponent(key)}/url`),

  // Analytics
  getAnalytics: (timeRange?: string) => api.get('/cloudflare/analytics', { params: { time_range: timeRange } }),
  getZoneAnalytics: () => api.get('/cloudflare/analytics/zone'),
  getTrafficSummary: () => api.get('/cloudflare/analytics/traffic'),
  getBandwidthStats: () => api.get('/cloudflare/analytics/bandwidth'),
  getSecurityAnalytics: () => api.get('/cloudflare/analytics/security'),
  getCacheAnalytics: () => api.get('/cloudflare/analytics/cache'),

  // Page Rules
  listPageRules: () => api.get('/cloudflare/page-rules'),
  createPageRule: (rule: any) => api.post('/cloudflare/page-rules', rule),
  updatePageRule: (id: string, rule: any) => api.put(`/cloudflare/page-rules/${id}`, rule),
  deletePageRule: (id: string) => api.delete(`/cloudflare/page-rules/${id}`),

  // Zone Settings
  getZoneSettings: () => api.get('/cloudflare/zone/settings'),
  updateZoneSettings: (settings: any) => api.patch('/cloudflare/zone/settings', settings),
  toggleDevMode: (enabled: boolean) => api.post('/cloudflare/zone/development-mode', { enabled }),

  // Plugin Settings
  getPluginSettings: () => api.get('/cloudflare/settings'),
  updatePluginSettings: (settings: any) => api.put('/cloudflare/settings', settings),
  resetPluginSettings: () => api.post('/cloudflare/settings/reset'),

  // Auto-Purge Settings
  getAutoPurgeSettings: () => api.get('/cloudflare/settings/auto-purge'),
  updateAutoPurgeSettings: (settings: {
    auto_purge_enabled?: boolean;
    auto_purge_on_post_update?: boolean;
    auto_purge_on_page_update?: boolean;
    auto_purge_on_media_upload?: boolean;
    auto_purge_on_theme_change?: boolean;
    auto_purge_on_menu_update?: boolean;
    auto_purge_entire_site?: boolean;
    auto_purge_homepage?: boolean;
    auto_purge_archives?: boolean;
    auto_purge_custom_urls?: string;
    auto_purge_delay_ms?: number;
  }) => api.put('/cloudflare/settings/auto-purge', settings),

  // Cache Warming Settings
  updateCacheWarmingSettings: (settings: {
    cache_warming_enabled?: boolean;
    cache_warming_schedule?: string;
  }) => api.put('/cloudflare/settings/cache-warming', settings),

  // Notification Settings
  updateNotificationSettings: (settings: {
    security_email_alerts?: boolean;
    security_slack_webhook?: string;
  }) => api.put('/cloudflare/settings/notifications', settings),

  // Advanced Settings
  updateAdvancedSettings: (settings: {
    development_mode_duration?: number;
    analytics_retention_days?: number;
    r2_default_bucket?: string;
    workers_enabled?: boolean;
  }) => api.put('/cloudflare/settings/advanced', settings),

  // D1 Databases
  listD1Databases: () => api.get('/cloudflare/d1/databases'),
  getD1Database: (id: string) => api.get(`/cloudflare/d1/databases/${id}`),
  createD1Database: (name: string) => api.post('/cloudflare/d1/databases', { name }),
  deleteD1Database: (id: string) => api.delete(`/cloudflare/d1/databases/${id}`),
  executeD1Query: (databaseId: string, sql: string) =>
    api.post(`/cloudflare/d1/databases/${databaseId}/query`, { sql }),
  executeD1Batch: (databaseId: string, statements: string[]) =>
    api.post(`/cloudflare/d1/databases/${databaseId}/batch`, { statements }),
  listD1Tables: (databaseId: string) =>
    api.get(`/cloudflare/d1/databases/${databaseId}/tables`),
  getD1TableSchema: (databaseId: string, tableName: string) =>
    api.get(`/cloudflare/d1/databases/${databaseId}/tables/${tableName}/schema`),

  // Stream Videos
  listStreamVideos: () => api.get('/cloudflare/stream/videos'),
  getStreamVideo: (id: string) => api.get(`/cloudflare/stream/videos/${id}`),
  deleteStreamVideo: (id: string) => api.delete(`/cloudflare/stream/videos/${id}`),
  searchStreamVideos: (query: string) =>
    api.get('/cloudflare/stream/videos/search', { params: { q: query } }),
  getStreamVideoUrls: (id: string) => api.get(`/cloudflare/stream/videos/${id}/urls`),
  getStreamEmbedCode: (id: string, options?: {
    autoplay?: boolean;
    muted?: boolean;
    loop?: boolean;
    controls?: boolean;
    posterTime?: number;
    startTime?: number;
    width?: string;
    height?: string;
  }) => api.get(`/cloudflare/stream/videos/${id}/embed`, { params: options }),
  getStreamStats: () => api.get('/cloudflare/stream/stats'),

  // Stream Live Inputs
  listLiveInputs: () => api.get('/cloudflare/stream/live-inputs'),
  createLiveInput: (options: { name?: string; recordingMode?: string; timeoutSeconds?: number }) =>
    api.post('/cloudflare/stream/live-inputs', {
      name: options.name,
      recording_mode: options.recordingMode,
      timeout_seconds: options.timeoutSeconds,
    }),
  deleteLiveInput: (id: string) => api.delete(`/cloudflare/stream/live-inputs/${id}`),
  getLiveInputUrls: (id: string) => api.get(`/cloudflare/stream/live-inputs/${id}/urls`),
};

export { api };
