// Mock data for development testing

export const mockZone = {
  id: 'zone_abc123def456',
  name: 'example.com',
  status: 'active',
  paused: false,
  type: 'full',
  development_mode: 0,
  name_servers: ['ada.ns.cloudflare.com', 'bob.ns.cloudflare.com'],
  original_name_servers: ['ns1.originaldns.com', 'ns2.originaldns.com'],
  plan: {
    id: 'pro',
    name: 'Pro',
    price: 20,
    currency: 'USD',
    frequency: 'monthly',
  },
  created_on: '2024-01-15T10:30:00Z',
  modified_on: '2025-01-08T14:20:00Z',
};

export const mockStatus = {
  connected: true,
  zone_id: 'zone_abc123def456',
  zone_name: 'example.com',
  plan: 'Pro',
  features: ['cdn', 'ssl', 'waf', 'workers', 'r2', 'stream'],
};

export const mockConnectionStatus = {
  connected: true,
  api_key_valid: true,
  zone_id: 'zone_abc123def456',
  last_check: new Date().toISOString(),
};

export const mockDnsRecords = [
  {
    id: 'dns_001',
    type: 'A',
    name: 'example.com',
    content: '192.168.1.1',
    proxied: true,
    ttl: 1,
    locked: false,
    created_on: '2024-01-15T10:30:00Z',
    modified_on: '2025-01-08T14:20:00Z',
  },
  {
    id: 'dns_002',
    type: 'A',
    name: 'www.example.com',
    content: '192.168.1.1',
    proxied: true,
    ttl: 1,
    locked: false,
    created_on: '2024-01-15T10:30:00Z',
    modified_on: '2025-01-08T14:20:00Z',
  },
  {
    id: 'dns_003',
    type: 'CNAME',
    name: 'blog.example.com',
    content: 'example.com',
    proxied: true,
    ttl: 1,
    locked: false,
    created_on: '2024-02-10T08:15:00Z',
    modified_on: '2024-02-10T08:15:00Z',
  },
  {
    id: 'dns_004',
    type: 'MX',
    name: 'example.com',
    content: 'mail.example.com',
    priority: 10,
    proxied: false,
    ttl: 3600,
    locked: false,
    created_on: '2024-01-15T10:30:00Z',
    modified_on: '2024-01-15T10:30:00Z',
  },
  {
    id: 'dns_005',
    type: 'TXT',
    name: 'example.com',
    content: 'v=spf1 include:_spf.google.com ~all',
    proxied: false,
    ttl: 3600,
    locked: false,
    created_on: '2024-03-01T12:00:00Z',
    modified_on: '2024-03-01T12:00:00Z',
  },
  {
    id: 'dns_006',
    type: 'AAAA',
    name: 'api.example.com',
    content: '2001:0db8:85a3:0000:0000:8a2e:0370:7334',
    proxied: true,
    ttl: 1,
    locked: false,
    created_on: '2024-04-15T16:45:00Z',
    modified_on: '2024-04-15T16:45:00Z',
  },
];

export const mockCacheStats = {
  requests: {
    total: 1250000,
    cached: 980000,
    uncached: 270000,
  },
  bandwidth: {
    total: 45.5, // GB
    cached: 38.2,
    uncached: 7.3,
  },
  cache_hit_ratio: 78.4,
  avg_ttl: 86400,
  last_purge: '2025-01-07T18:30:00Z',
};

export const mockSecurityLevel = 'medium';

export const mockFirewallRules = [
  {
    id: 'fw_001',
    description: 'Block bad bots',
    expression: '(http.user_agent contains "BadBot")',
    action: 'block',
    enabled: true,
    priority: 1,
    created_on: '2024-06-01T10:00:00Z',
  },
  {
    id: 'fw_002',
    description: 'Challenge suspicious IPs',
    expression: '(ip.geoip.country in {"CN" "RU"})',
    action: 'challenge',
    enabled: true,
    priority: 2,
    created_on: '2024-06-15T14:30:00Z',
  },
  {
    id: 'fw_003',
    description: 'Allow known good bots',
    expression: '(cf.client.bot)',
    action: 'allow',
    enabled: true,
    priority: 3,
    created_on: '2024-07-01T09:00:00Z',
  },
];

export const mockIpAccessRules = [
  {
    id: 'ip_001',
    ip: '192.168.1.100',
    mode: 'whitelist',
    notes: 'Office IP',
    created_on: '2024-05-10T11:00:00Z',
  },
  {
    id: 'ip_002',
    ip: '10.0.0.50',
    mode: 'block',
    notes: 'Known attacker',
    created_on: '2024-08-20T16:45:00Z',
  },
];

export const mockSecurityEvents = [
  {
    id: 'evt_001',
    action: 'block',
    source: 'firewall',
    client_ip: '203.0.113.50',
    country: 'CN',
    rule_id: 'fw_001',
    timestamp: new Date(Date.now() - 300000).toISOString(),
    user_agent: 'BadBot/1.0',
    uri: '/wp-admin/admin-ajax.php',
  },
  {
    id: 'evt_002',
    action: 'challenge',
    source: 'securityLevel',
    client_ip: '198.51.100.25',
    country: 'RU',
    rule_id: 'fw_002',
    timestamp: new Date(Date.now() - 600000).toISOString(),
    user_agent: 'Mozilla/5.0',
    uri: '/login',
  },
  {
    id: 'evt_003',
    action: 'block',
    source: 'waf',
    client_ip: '192.0.2.100',
    country: 'US',
    rule_id: 'waf_sqli_001',
    timestamp: new Date(Date.now() - 900000).toISOString(),
    user_agent: 'curl/7.68.0',
    uri: '/api/users?id=1%27%20OR%201=1',
  },
  {
    id: 'evt_004',
    action: 'managed_challenge',
    source: 'bot_management',
    client_ip: '172.16.0.55',
    country: 'DE',
    rule_id: 'bm_001',
    timestamp: new Date(Date.now() - 1200000).toISOString(),
    user_agent: 'Python-urllib/3.9',
    uri: '/sitemap.xml',
  },
];

export const mockWafRules = [
  { id: 'waf_sqli', name: 'SQL Injection', enabled: true, mode: 'block' },
  { id: 'waf_xss', name: 'Cross-Site Scripting', enabled: true, mode: 'block' },
  { id: 'waf_rce', name: 'Remote Code Execution', enabled: true, mode: 'block' },
  { id: 'waf_lfi', name: 'Local File Inclusion', enabled: true, mode: 'block' },
  { id: 'waf_rfi', name: 'Remote File Inclusion', enabled: true, mode: 'challenge' },
];

export const mockSslStatus = {
  status: 'active',
  mode: 'full_strict',
  certificate_pack: {
    id: 'cert_001',
    type: 'universal',
    hosts: ['example.com', '*.example.com'],
    status: 'active',
    expires_on: '2025-07-15T00:00:00Z',
  },
  always_use_https: true,
  automatic_https_rewrites: true,
  min_tls_version: '1.2',
  opportunistic_encryption: true,
};

export const mockCertificates = [
  {
    id: 'cert_001',
    type: 'universal',
    hosts: ['example.com', '*.example.com'],
    status: 'active',
    issuer: 'Cloudflare Inc ECC CA-3',
    expires_on: '2025-07-15T00:00:00Z',
    uploaded_on: '2024-07-15T00:00:00Z',
  },
  {
    id: 'cert_002',
    type: 'dedicated',
    hosts: ['api.example.com'],
    status: 'active',
    issuer: 'Cloudflare Inc RSA CA-2',
    expires_on: '2025-09-01T00:00:00Z',
    uploaded_on: '2024-09-01T00:00:00Z',
  },
];

export const mockWorkers = [
  {
    id: 'worker_001',
    name: 'edge-auth',
    created_on: '2024-08-15T10:00:00Z',
    modified_on: '2025-01-05T14:30:00Z',
    routes: ['example.com/api/*'],
    size: 15420,
    usage_model: 'bundled',
  },
  {
    id: 'worker_002',
    name: 'image-optimizer',
    created_on: '2024-09-20T08:00:00Z',
    modified_on: '2024-12-10T16:45:00Z',
    routes: ['example.com/images/*'],
    size: 28650,
    usage_model: 'bundled',
  },
  {
    id: 'worker_003',
    name: 'geo-redirect',
    created_on: '2024-10-01T12:00:00Z',
    modified_on: '2024-10-01T12:00:00Z',
    routes: ['example.com/*'],
    size: 5230,
    usage_model: 'unbound',
  },
];

export const mockWorkerRoutes = [
  {
    id: 'route_001',
    pattern: 'example.com/api/*',
    script: 'edge-auth',
    enabled: true,
  },
  {
    id: 'route_002',
    pattern: 'example.com/images/*',
    script: 'image-optimizer',
    enabled: true,
  },
  {
    id: 'route_003',
    pattern: 'example.com/*',
    script: 'geo-redirect',
    enabled: false,
  },
];

export const mockKVNamespaces = [
  {
    id: 'kv_001',
    title: 'USER_SESSIONS',
    supports_url_encoding: true,
    keys_count: 1250,
    size_bytes: 2456000,
  },
  {
    id: 'kv_002',
    title: 'CACHE_METADATA',
    supports_url_encoding: true,
    keys_count: 8500,
    size_bytes: 12500000,
  },
  {
    id: 'kv_003',
    title: 'RATE_LIMITS',
    supports_url_encoding: true,
    keys_count: 450,
    size_bytes: 89000,
  },
];

export const mockKVKeys = [
  { name: 'user:abc123', expiration: 1704672000 },
  { name: 'user:def456', expiration: 1704758400 },
  { name: 'user:ghi789', expiration: null },
  { name: 'session:sess_001', expiration: 1704499200 },
  { name: 'config:global', expiration: null },
];

export const mockR2Buckets = [
  {
    name: 'media-assets',
    creation_date: '2024-06-01T00:00:00Z',
    location: 'WNAM',
    storage_class: 'Standard',
    objects: 15420,
    size_bytes: 5250000000,
  },
  {
    name: 'backups',
    creation_date: '2024-03-15T00:00:00Z',
    location: 'ENAM',
    storage_class: 'Standard',
    objects: 256,
    size_bytes: 125000000000,
  },
  {
    name: 'logs',
    creation_date: '2024-01-01T00:00:00Z',
    location: 'WNAM',
    storage_class: 'Infrequent Access',
    objects: 45000,
    size_bytes: 8500000000,
  },
];

export const mockR2Objects = [
  {
    key: 'images/hero.jpg',
    size: 2500000,
    last_modified: '2025-01-05T10:30:00Z',
    etag: '"abc123def456"',
    storage_class: 'Standard',
  },
  {
    key: 'images/logo.png',
    size: 45000,
    last_modified: '2024-12-20T08:15:00Z',
    etag: '"ghi789jkl012"',
    storage_class: 'Standard',
  },
  {
    key: 'documents/report.pdf',
    size: 1250000,
    last_modified: '2025-01-07T16:45:00Z',
    etag: '"mno345pqr678"',
    storage_class: 'Standard',
  },
  {
    key: 'videos/',
    size: 0,
    last_modified: '2024-11-01T12:00:00Z',
    etag: '"folder"',
    storage_class: 'Standard',
    is_folder: true,
  },
];

export const mockStreamVideos = {
  videos: [
    {
      uid: 'video_001',
      thumbnail: 'https://placehold.co/320x180/f97316/fff?text=Product+Demo',
      readyToStream: true,
      status: { state: 'ready' },
      meta: { name: 'Product Demo 2025' },
      created: '2025-01-03T10:00:00Z',
      duration: 245,
      size: 125000000,
      playback: { hls: 'https://customer-123.cloudflarestream.com/video_001/manifest.m3u8' },
      preview: 'https://customer-123.cloudflarestream.com/video_001/watch',
    },
    {
      uid: 'video_002',
      thumbnail: 'https://placehold.co/320x180/f97316/fff?text=Tutorial',
      readyToStream: true,
      status: { state: 'ready' },
      meta: { name: 'Tutorial - Getting Started' },
      created: '2024-12-15T14:30:00Z',
      duration: 480,
      size: 250000000,
      playback: { hls: 'https://customer-123.cloudflarestream.com/video_002/manifest.m3u8' },
      preview: 'https://customer-123.cloudflarestream.com/video_002/watch',
    },
    {
      uid: 'video_003',
      thumbnail: 'https://placehold.co/320x180/666/fff?text=Processing',
      readyToStream: false,
      status: { state: 'processing' },
      meta: { name: 'Webinar Recording' },
      created: '2025-01-08T09:00:00Z',
      duration: 3600,
      size: 1500000000,
      playback: {},
      preview: null,
    },
  ],
};

export const mockStreamStats = {
  total_videos: 3,
  total_duration_seconds: 4325,
  total_storage_bytes: 1875000000,
  ready_videos: 2,
  processing_videos: 1,
  live_inputs: 2,
  active_live_streams: 1,
};

export const mockStreamLiveInputs = {
  live_inputs: [
    {
      uid: 'live_001',
      meta: { name: 'Main Studio' },
      created: '2024-10-01T00:00:00Z',
      rtmps: { url: 'rtmps://live.cloudflare.com:443/live', streamKey: 'live_key_abc123' },
      srt: { url: 'srt://live.cloudflare.com:778', streamId: 'live_001' },
      webRTC: { url: 'https://customer-123.cloudflarestream.com/live_001/webRTC/publish' },
      status: { current: { state: 'connected' } },
    },
    {
      uid: 'live_002',
      meta: { name: 'Backup Stream' },
      created: '2024-11-15T00:00:00Z',
      rtmps: { url: 'rtmps://live.cloudflare.com:443/live', streamKey: 'live_key_def456' },
      srt: { url: 'srt://live.cloudflare.com:778', streamId: 'live_002' },
      webRTC: { url: 'https://customer-123.cloudflarestream.com/live_002/webRTC/publish' },
      status: { current: { state: 'idle' } },
    },
  ],
};

export const mockD1Databases = {
  databases: [
    {
      uuid: 'd1_001',
      name: 'production-db',
      version: 'v1',
      num_tables: 12,
      file_size: 5242880,
      created_at: '2024-06-15T10:00:00Z',
    },
    {
      uuid: 'd1_002',
      name: 'analytics-db',
      version: 'v1',
      num_tables: 8,
      file_size: 10485760,
      created_at: '2024-08-20T14:30:00Z',
    },
    {
      uuid: 'd1_003',
      name: 'cache-db',
      version: 'v1',
      num_tables: 3,
      file_size: 1048576,
      created_at: '2024-10-01T09:00:00Z',
    },
  ],
};

export const mockD1Tables = ['users', 'posts', 'comments', 'sessions', 'settings'];

export const mockD1QueryResult = {
  results: [
    { id: 1, name: 'users', type: 'table', tbl_name: 'users', rootpage: 2, sql: 'CREATE TABLE users (...)' },
    { id: 2, name: 'posts', type: 'table', tbl_name: 'posts', rootpage: 3, sql: 'CREATE TABLE posts (...)' },
    { id: 3, name: 'comments', type: 'table', tbl_name: 'comments', rootpage: 4, sql: 'CREATE TABLE comments (...)' },
  ],
  success: true,
  meta: {
    duration: 0.5,
    changes: 0,
    rows_read: 3,
    rows_written: 0,
  },
};

export const mockAnalytics = {
  summary: {
    requests: 1250000,
    bandwidth: 45.5, // GB
    unique_visitors: 85000,
    page_views: 320000,
    threats_blocked: 12500,
    ssl_requests_percent: 98.5,
  },
  requests_by_country: [
    { country: 'US', requests: 450000, percentage: 36 },
    { country: 'GB', requests: 187500, percentage: 15 },
    { country: 'DE', requests: 150000, percentage: 12 },
    { country: 'FR', requests: 125000, percentage: 10 },
    { country: 'CA', requests: 100000, percentage: 8 },
    { country: 'Other', requests: 237500, percentage: 19 },
  ],
  traffic_by_hour: Array.from({ length: 24 }, (_, i) => ({
    hour: i,
    requests: Math.floor(30000 + Math.random() * 40000),
    bandwidth: Math.floor(1 + Math.random() * 3),
    cached: Math.floor(0.6 + Math.random() * 0.3),
  })),
  traffic_by_day: Array.from({ length: 7 }, (_, i) => {
    const date = new Date();
    date.setDate(date.getDate() - (6 - i));
    return {
      date: date.toISOString().split('T')[0],
      requests: Math.floor(150000 + Math.random() * 50000),
      bandwidth: Math.floor(5 + Math.random() * 3),
      unique_visitors: Math.floor(10000 + Math.random() * 5000),
    };
  }),
  status_codes: [
    { code: '2xx', count: 1100000, percentage: 88 },
    { code: '3xx', count: 87500, percentage: 7 },
    { code: '4xx', count: 50000, percentage: 4 },
    { code: '5xx', count: 12500, percentage: 1 },
  ],
  content_types: [
    { type: 'HTML', requests: 320000, bandwidth: 2.5 },
    { type: 'JavaScript', requests: 280000, bandwidth: 8.2 },
    { type: 'CSS', requests: 150000, bandwidth: 1.8 },
    { type: 'Images', requests: 400000, bandwidth: 28.5 },
    { type: 'Other', requests: 100000, bandwidth: 4.5 },
  ],
};

export const mockBandwidthStats = {
  total: 45.5, // GB
  cached: 38.2,
  uncached: 7.3,
  saved: 84, // percentage
  by_content_type: {
    html: 2.5,
    javascript: 8.2,
    css: 1.8,
    images: 28.5,
    other: 4.5,
  },
};

export const mockSecurityAnalytics = {
  threats_blocked: 12500,
  threats_by_type: [
    { type: 'Bad Bot', count: 5200 },
    { type: 'SQL Injection', count: 2800 },
    { type: 'XSS', count: 1950 },
    { type: 'DDoS', count: 1500 },
    { type: 'Other', count: 1050 },
  ],
  threats_by_country: [
    { country: 'CN', count: 4500 },
    { country: 'RU', count: 3200 },
    { country: 'US', count: 1800 },
    { country: 'BR', count: 1500 },
    { country: 'Other', count: 1500 },
  ],
  waf_triggered: 4750,
  rate_limited: 2300,
  bot_score_distribution: [
    { range: '0-10', count: 8500 },
    { range: '11-30', count: 2500 },
    { range: '31-50', count: 1000 },
    { range: '51-70', count: 300 },
    { range: '71-100', count: 200 },
  ],
};

export const mockPageRules = [
  {
    id: 'rule_001',
    targets: [{ target: 'url', constraint: { operator: 'matches', value: '*example.com/api/*' } }],
    actions: [
      { id: 'cache_level', value: 'bypass' },
      { id: 'security_level', value: 'high' },
    ],
    priority: 1,
    status: 'active',
    created_on: '2024-05-01T10:00:00Z',
    modified_on: '2024-05-01T10:00:00Z',
  },
  {
    id: 'rule_002',
    targets: [{ target: 'url', constraint: { operator: 'matches', value: '*example.com/static/*' } }],
    actions: [
      { id: 'cache_level', value: 'cache_everything' },
      { id: 'edge_cache_ttl', value: 86400 },
      { id: 'browser_cache_ttl', value: 31536000 },
    ],
    priority: 2,
    status: 'active',
    created_on: '2024-06-15T14:30:00Z',
    modified_on: '2024-06-15T14:30:00Z',
  },
  {
    id: 'rule_003',
    targets: [{ target: 'url', constraint: { operator: 'matches', value: '*example.com/admin/*' } }],
    actions: [
      { id: 'ssl', value: 'full_strict' },
      { id: 'security_level', value: 'high' },
      { id: 'waf', value: 'on' },
    ],
    priority: 3,
    status: 'active',
    created_on: '2024-07-20T08:00:00Z',
    modified_on: '2024-12-01T16:45:00Z',
  },
];

export const mockZoneSettings = {
  always_online: 'on',
  always_use_https: 'on',
  automatic_https_rewrites: 'on',
  browser_cache_ttl: 14400,
  browser_check: 'on',
  cache_level: 'aggressive',
  challenge_ttl: 1800,
  development_mode: 'off',
  email_obfuscation: 'on',
  hotlink_protection: 'off',
  ip_geolocation: 'on',
  ipv6: 'on',
  minify: { css: 'on', html: 'on', js: 'on' },
  mirage: 'on',
  mobile_redirect: { status: 'off' },
  opportunistic_encryption: 'on',
  polish: 'lossless',
  rocket_loader: 'on',
  security_level: 'medium',
  server_side_exclude: 'on',
  ssl: 'full_strict',
  tls_1_3: 'on',
  waf: 'on',
  webp: 'on',
  websockets: 'on',
};

export const mockPluginSettings = {
  api_email: 'admin@example.com',
  api_key: '••••••••••••••••••••••••••••••••••••',
  zone_id: 'zone_abc123def456',
  auto_purge_enabled: true,
  auto_purge_on_post_update: true,
  auto_purge_on_theme_change: true,
  auto_purge_on_plugin_update: false,
  cache_warming_enabled: true,
  cache_warming_urls: ['/'],
  notifications_enabled: true,
  notification_email: 'admin@example.com',
  development_mode: false,
};

// Helper to simulate API delay
export const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

// Check if we're in development mode
export const isDev = import.meta.env.DEV;
