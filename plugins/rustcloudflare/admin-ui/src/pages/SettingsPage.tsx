import { useState, useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useForm } from 'react-hook-form';
import { useSearchParams } from 'react-router-dom';
import {
  Settings,
  Key,
  Globe,
  Shield,
  Bell,
  RefreshCw,
  Check,
  AlertTriangle,
  Eye,
  EyeOff,
  Save,
  TestTube,
  Trash2,
  Clock,
  Database,
  Zap,
  Link,
  Copy,
  ExternalLink,
  LogIn,
  Cloud,
  X,
} from 'lucide-react';
import { cloudflareApi } from '../lib/api';
import toast from 'react-hot-toast';
import { clsx } from 'clsx';

interface CloudflareAccount {
  id: string;
  name: string;
}

interface CloudflareZone {
  id: string;
  name: string;
  status: string;
}

interface PluginSettings {
  api_token: string;
  account_id: string;
  zone_id: string;
  auto_purge_enabled: boolean;
  auto_purge_on_post_update: boolean;
  auto_purge_on_media_upload: boolean;
  auto_purge_on_theme_change: boolean;
  development_mode_duration: number;
  cache_warming_enabled: boolean;
  cache_warming_schedule: string;
  security_email_alerts: boolean;
  security_slack_webhook: string;
  analytics_retention_days: number;
  r2_default_bucket: string;
  workers_enabled: boolean;
}

export function SettingsPage() {
  const queryClient = useQueryClient();
  const [searchParams, setSearchParams] = useSearchParams();
  const [showToken, setShowToken] = useState(false);
  const [testingConnection, setTestingConnection] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<'idle' | 'success' | 'error'>('idle');
  const [ssoLoading, setSsoLoading] = useState(false);
  const [showSelectionModal, setShowSelectionModal] = useState(false);
  const [ssoToken, setSsoToken] = useState<string | null>(null);
  const [ssoAccounts, setSsoAccounts] = useState<CloudflareAccount[]>([]);
  const [ssoZones, setSsoZones] = useState<CloudflareZone[]>([]);
  const [selectedAccount, setSelectedAccount] = useState<string>('');
  const [selectedZone, setSelectedZone] = useState<string>('');

  const { data: settings, isLoading } = useQuery({
    queryKey: ['plugin-settings'],
    queryFn: () => cloudflareApi.getPluginSettings(),
  });

  const { data: connectionInfo, refetch: refetchConnection } = useQuery({
    queryKey: ['connection-info'],
    queryFn: () => cloudflareApi.getConnectionStatus(),
  });

  const { register, handleSubmit, watch, reset } = useForm<PluginSettings>({
    defaultValues: settings?.data?.data,
  });

  // Handle SSO callback URL params
  useEffect(() => {
    const error = searchParams.get('error');
    const ssoSuccess = searchParams.get('sso_success');
    const zone = searchParams.get('zone');
    const token = searchParams.get('sso_token');
    const accountsJson = searchParams.get('sso_accounts');
    const zonesJson = searchParams.get('sso_zones');

    // Clear URL params after processing
    const clearParams = () => {
      setSearchParams({}, { replace: true });
    };

    if (error) {
      toast.error(decodeURIComponent(error));
      clearParams();
    } else if (ssoSuccess === 'true' && zone) {
      toast.success(`Connected to ${decodeURIComponent(zone)}`);
      refetchConnection();
      clearParams();
    } else if (token && accountsJson && zonesJson) {
      // Need to select account/zone
      try {
        const accounts = JSON.parse(decodeURIComponent(accountsJson));
        const zones = JSON.parse(decodeURIComponent(zonesJson));
        setSsoToken(decodeURIComponent(token));
        setSsoAccounts(accounts);
        setSsoZones(zones);
        setShowSelectionModal(true);
        if (accounts.length === 1) setSelectedAccount(accounts[0].id);
        if (zones.length === 1) setSelectedZone(zones[0].id);
        clearParams();
      } catch (e) {
        toast.error('Failed to parse SSO response');
        clearParams();
      }
    }
  }, [searchParams, setSearchParams, refetchConnection]);

  const updateSettingsMutation = useMutation({
    mutationFn: (data: PluginSettings) => cloudflareApi.updatePluginSettings(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['plugin-settings'] });
      toast.success('Settings saved');
    },
    onError: () => toast.error('Failed to save settings'),
  });

  // Auto-purge settings mutation
  const updateAutoPurgeMutation = useMutation({
    mutationFn: (data: {
      auto_purge_enabled?: boolean;
      auto_purge_on_post_update?: boolean;
      auto_purge_on_media_upload?: boolean;
      auto_purge_on_theme_change?: boolean;
    }) => cloudflareApi.updateAutoPurgeSettings(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['plugin-settings'] });
    },
    onError: () => toast.error('Failed to update auto-purge settings'),
  });

  // Cache warming settings mutation
  const updateCacheWarmingMutation = useMutation({
    mutationFn: (data: { cache_warming_enabled?: boolean; cache_warming_schedule?: string }) =>
      cloudflareApi.updateCacheWarmingSettings(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['plugin-settings'] });
    },
    onError: () => toast.error('Failed to update cache warming settings'),
  });

  // Notification settings mutation
  const updateNotificationMutation = useMutation({
    mutationFn: (data: { security_email_alerts?: boolean; security_slack_webhook?: string }) =>
      cloudflareApi.updateNotificationSettings(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['plugin-settings'] });
    },
    onError: () => toast.error('Failed to update notification settings'),
  });

  // Advanced settings mutation
  const updateAdvancedMutation = useMutation({
    mutationFn: (data: {
      development_mode_duration?: number;
      analytics_retention_days?: number;
      r2_default_bucket?: string;
      workers_enabled?: boolean;
    }) => cloudflareApi.updateAdvancedSettings(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['plugin-settings'] });
    },
    onError: () => toast.error('Failed to update advanced settings'),
  });

  // Start SSO login
  const startSsoLogin = async () => {
    setSsoLoading(true);
    try {
      const response = await cloudflareApi.getSsoAuthUrl();
      const url = response.data?.data;
      if (url) {
        window.location.href = url;
      } else {
        toast.error('SSO not configured');
      }
    } catch {
      toast.error('Failed to start SSO login');
    } finally {
      setSsoLoading(false);
    }
  };

  // Complete SSO with selected account/zone
  const completeSsoConnection = async () => {
    if (!ssoToken || !selectedAccount || !selectedZone) {
      toast.error('Please select an account and zone');
      return;
    }

    setSsoLoading(true);
    try {
      const response = await cloudflareApi.ssoComplete(ssoToken, selectedAccount, selectedZone);
      if (response.data?.success && response.data?.connected) {
        toast.success(response.data.message || 'Connected successfully');
        setShowSelectionModal(false);
        setSsoToken(null);
        refetchConnection();
        queryClient.invalidateQueries({ queryKey: ['plugin-settings'] });
      } else {
        toast.error(response.data?.message || 'Connection failed');
      }
    } catch {
      toast.error('Failed to complete SSO connection');
    } finally {
      setSsoLoading(false);
    }
  };

  // Disconnect from Cloudflare
  const disconnectCloudflare = async () => {
    try {
      await cloudflareApi.disconnect();
      toast.success('Disconnected from Cloudflare');
      refetchConnection();
      queryClient.invalidateQueries({ queryKey: ['plugin-settings'] });
    } catch {
      toast.error('Failed to disconnect');
    }
  };

  const testConnection = async () => {
    setTestingConnection(true);
    try {
      await cloudflareApi.testConnection();
      setConnectionStatus('success');
      toast.success('Connection successful');
    } catch {
      setConnectionStatus('error');
      toast.error('Connection failed');
    } finally {
      setTestingConnection(false);
    }
  };

  const clearCache = async () => {
    try {
      await cloudflareApi.clearLocalCache();
      queryClient.invalidateQueries();
      toast.success('Local cache cleared');
    } catch {
      toast.error('Failed to clear cache');
    }
  };

  const connection = connectionInfo?.data || {
    connected: false,
    zone_name: '',
    plan: '',
    status: '',
  };

  const currentSettings = settings?.data?.data || {};

  return (
    <div className="p-6">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-neutral-100 flex items-center gap-3"><Settings className="w-7 h-7 text-orange-400" />Settings</h1>
        <p className="text-neutral-400">Configure your Cloudflare integration</p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Main Settings */}
        <div className="lg:col-span-2 space-y-6">
          {/* Cloudflare Connection */}
          <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
            <div className="flex items-center gap-3 mb-6">
              <div className="p-2 bg-primary-500/10 border border-primary-500/20 rounded-lg">
                <Cloud className="w-5 h-5 text-primary-400" />
              </div>
              <div>
                <h2 className="text-lg font-semibold text-neutral-100">
                  Cloudflare Connection
                </h2>
                <p className="text-sm text-neutral-500">Connect to your Cloudflare account with one click</p>
              </div>
            </div>

            {connection.connected ? (
              <div className="space-y-4">
                <div className="flex items-center gap-3 p-4 bg-success-500/10 border border-success-500/20 rounded-lg">
                  <Check className="w-6 h-6 text-success-400" />
                  <div className="flex-1">
                    <p className="font-medium text-success-300">Connected to Cloudflare</p>
                    {connection.zone_name && (
                      <p className="text-sm text-neutral-400">Zone: {connection.zone_name}</p>
                    )}
                  </div>
                </div>

                <div className="flex gap-3">
                  <button
                    onClick={disconnectCloudflare}
                    className="flex items-center gap-2 px-4 py-2.5 border border-danger-500/30 text-danger-400 rounded-lg hover:bg-danger-500/10 transition-colors"
                  >
                    <X className="w-4 h-4" />
                    Disconnect
                  </button>
                  <button
                    onClick={startSsoLogin}
                    disabled={ssoLoading}
                    className="flex items-center gap-2 px-4 py-2.5 border border-neutral-700 text-neutral-300 rounded-lg hover:bg-neutral-700 transition-colors"
                  >
                    {ssoLoading ? (
                      <RefreshCw className="w-4 h-4 animate-spin" />
                    ) : (
                      <RefreshCw className="w-4 h-4" />
                    )}
                    Reconnect
                  </button>
                </div>
              </div>
            ) : (
              <div className="space-y-6">
                {/* SSO Login - Primary Option */}
                <div className="text-center py-8 border border-dashed border-neutral-700 rounded-lg">
                  <Cloud className="w-12 h-12 mx-auto mb-4 text-primary-400" />
                  <h3 className="text-lg font-medium text-neutral-100 mb-2">
                    Connect with Cloudflare SSO
                  </h3>
                  <p className="text-sm text-neutral-500 mb-6 max-w-md mx-auto">
                    Sign in with your Cloudflare account to automatically connect and configure your zone.
                  </p>
                  <button
                    onClick={startSsoLogin}
                    disabled={ssoLoading}
                    className="inline-flex items-center gap-3 px-6 py-3 bg-[#f6821f] hover:bg-[#e5751b] text-white rounded-lg font-medium transition-colors"
                  >
                    {ssoLoading ? (
                      <RefreshCw className="w-5 h-5 animate-spin" />
                    ) : (
                      <LogIn className="w-5 h-5" />
                    )}
                    Sign in with Cloudflare
                  </button>
                </div>

                {/* Manual Token Entry - Alternative */}
                <div className="pt-6 border-t border-neutral-700">
                  <p className="text-sm text-neutral-500 mb-4">
                    Or connect manually with an API token:
                  </p>
                  <form onSubmit={handleSubmit((data) => updateSettingsMutation.mutate(data))}>
                    <div className="space-y-4">
                      <div>
                        <label className="block text-sm font-medium text-neutral-300 mb-1">
                          API Token
                        </label>
                        <div className="relative">
                          <input
                            type={showToken ? 'text' : 'password'}
                            {...register('api_token')}
                            placeholder="Enter your Cloudflare API token"
                            defaultValue={currentSettings.api_token}
                            className="w-full px-3 py-2 pr-10 border border-neutral-700 rounded-lg bg-neutral-900 text-neutral-100 placeholder-neutral-500 focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                          />
                          <button
                            type="button"
                            onClick={() => setShowToken(!showToken)}
                            className="absolute right-3 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-200"
                          >
                            {showToken ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                          </button>
                        </div>
                        <p className="text-xs text-neutral-500 mt-1">
                          Create a token at{' '}
                          <a
                            href="https://dash.cloudflare.com/profile/api-tokens"
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-primary-400 hover:underline"
                          >
                            Cloudflare Dashboard
                          </a>
                        </p>
                      </div>

                      <div className="grid grid-cols-2 gap-4">
                        <div>
                          <label className="block text-sm font-medium text-neutral-300 mb-1">
                            Account ID
                          </label>
                          <input
                            type="text"
                            {...register('account_id')}
                            placeholder="Account ID"
                            defaultValue={currentSettings.account_id}
                            className="w-full px-3 py-2 border border-neutral-700 rounded-lg bg-neutral-900 text-neutral-100 placeholder-neutral-500 focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                          />
                        </div>
                        <div>
                          <label className="block text-sm font-medium text-neutral-300 mb-1">
                            Zone ID
                          </label>
                          <input
                            type="text"
                            {...register('zone_id')}
                            placeholder="Zone ID"
                            defaultValue={currentSettings.zone_id}
                            className="w-full px-3 py-2 border border-neutral-700 rounded-lg bg-neutral-900 text-neutral-100 placeholder-neutral-500 focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                          />
                        </div>
                      </div>

                      <button
                        type="submit"
                        disabled={updateSettingsMutation.isPending}
                        className="flex items-center gap-2 px-4 py-2.5 bg-primary-600 hover:bg-primary-700 text-white rounded-lg font-medium transition-colors"
                      >
                        {updateSettingsMutation.isPending ? (
                          <RefreshCw className="w-4 h-4 animate-spin" />
                        ) : (
                          <Save className="w-4 h-4" />
                        )}
                        Save & Connect
                      </button>
                    </div>
                  </form>
                </div>
              </div>
            )}
          </div>

          {/* Auto Purge Settings */}
          <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
            <div className="flex items-center gap-3 mb-6">
              <div className="p-2 bg-accent-500/10 border border-accent-500/20 rounded-lg">
                <RefreshCw className="w-5 h-5 text-accent-400" />
              </div>
              <div>
                <h2 className="text-lg font-semibold text-neutral-100">
                  Automatic Cache Purge
                </h2>
                <p className="text-sm text-neutral-500">Configure when to automatically purge cache</p>
              </div>
            </div>

            <div className="space-y-4">
              <ToggleSetting
                label="Enable Auto-Purge"
                description="Automatically purge Cloudflare cache on content changes"
                enabled={currentSettings.auto_purge_enabled}
                onChange={(enabled) => updateAutoPurgeMutation.mutate({ auto_purge_enabled: enabled })}
              />

              <div className={clsx(
                "pl-4 border-l-2 border-neutral-700 space-y-4 transition-opacity",
                !currentSettings.auto_purge_enabled && "opacity-50 pointer-events-none"
              )}>
                <ToggleSetting
                  label="On Post Update"
                  description="Purge when a post or page is published or updated"
                  enabled={currentSettings.auto_purge_on_post_update}
                  onChange={(enabled) => updateAutoPurgeMutation.mutate({ auto_purge_on_post_update: enabled })}
                />

                <ToggleSetting
                  label="On Media Upload"
                  description="Purge when media files are uploaded or deleted"
                  enabled={currentSettings.auto_purge_on_media_upload}
                  onChange={(enabled) => updateAutoPurgeMutation.mutate({ auto_purge_on_media_upload: enabled })}
                />

                <ToggleSetting
                  label="On Theme Change"
                  description="Purge when theme or plugins are changed"
                  enabled={currentSettings.auto_purge_on_theme_change}
                  onChange={(enabled) => updateAutoPurgeMutation.mutate({ auto_purge_on_theme_change: enabled })}
                />
              </div>
            </div>
          </div>

          {/* Cache Warming */}
          <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
            <div className="flex items-center gap-3 mb-6">
              <div className="p-2 bg-success-500/10 border border-success-500/20 rounded-lg">
                <Zap className="w-5 h-5 text-success-400" />
              </div>
              <div>
                <h2 className="text-lg font-semibold text-neutral-100">
                  Cache Warming
                </h2>
                <p className="text-sm text-neutral-500">Pre-warm cache for faster first visits</p>
              </div>
            </div>

            <div className="space-y-4">
              <ToggleSetting
                label="Enable Cache Warming"
                description="Automatically warm cache after purge operations"
                enabled={currentSettings.cache_warming_enabled}
                onChange={(enabled) => updateCacheWarmingMutation.mutate({ cache_warming_enabled: enabled })}
              />

              <div className={clsx(!currentSettings.cache_warming_enabled && "opacity-50 pointer-events-none")}>
                <label className="block text-sm font-medium text-neutral-300 mb-1">
                  Warming Schedule
                </label>
                <select
                  value={currentSettings.cache_warming_schedule || 'immediate'}
                  onChange={(e) => updateCacheWarmingMutation.mutate({ cache_warming_schedule: e.target.value })}
                  className="w-full px-3 py-2 border border-neutral-700 rounded-lg bg-neutral-900 text-neutral-100 focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                >
                  <option value="immediate">Immediately after purge</option>
                  <option value="hourly">Every hour</option>
                  <option value="daily">Daily at midnight</option>
                  <option value="manual">Manual only</option>
                </select>
              </div>
            </div>
          </div>

          {/* Notifications */}
          <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
            <div className="flex items-center gap-3 mb-6">
              <div className="p-2 bg-warning-500/10 border border-warning-500/20 rounded-lg">
                <Bell className="w-5 h-5 text-warning-400" />
              </div>
              <div>
                <h2 className="text-lg font-semibold text-neutral-100">
                  Notifications
                </h2>
                <p className="text-sm text-neutral-500">Configure security alerts and notifications</p>
              </div>
            </div>

            <div className="space-y-4">
              <ToggleSetting
                label="Email Alerts"
                description="Receive email notifications for security events"
                enabled={currentSettings.security_email_alerts}
                onChange={(enabled) => updateNotificationMutation.mutate({ security_email_alerts: enabled })}
              />

              <div>
                <label className="block text-sm font-medium text-neutral-300 mb-1">
                  Slack Webhook URL
                </label>
                <input
                  type="url"
                  placeholder="https://hooks.slack.com/services/..."
                  defaultValue={currentSettings.security_slack_webhook || ''}
                  onBlur={(e) => {
                    if (e.target.value !== currentSettings.security_slack_webhook) {
                      updateNotificationMutation.mutate({ security_slack_webhook: e.target.value });
                    }
                  }}
                  className="w-full px-3 py-2 border border-neutral-700 rounded-lg bg-neutral-900 text-neutral-100 placeholder-neutral-500 focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                />
                <p className="text-xs text-neutral-500 mt-1">
                  Receive security alerts in your Slack channel
                </p>
              </div>
            </div>
          </div>

          {/* Advanced Settings */}
          <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
            <div className="flex items-center gap-3 mb-6">
              <div className="p-2 bg-neutral-700 rounded-lg">
                <Settings className="w-5 h-5 text-neutral-400" />
              </div>
              <div>
                <h2 className="text-lg font-semibold text-neutral-100">
                  Advanced Settings
                </h2>
                <p className="text-sm text-neutral-500">Additional configuration options</p>
              </div>
            </div>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-neutral-300 mb-1">
                  Development Mode Duration (minutes)
                </label>
                <select
                  value={currentSettings.development_mode_duration || 180}
                  onChange={(e) => updateAdvancedMutation.mutate({ development_mode_duration: parseInt(e.target.value) })}
                  className="w-full px-3 py-2 border border-neutral-700 rounded-lg bg-neutral-900 text-neutral-100 focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                >
                  <option value={60}>1 hour</option>
                  <option value={180}>3 hours</option>
                  <option value={360}>6 hours</option>
                  <option value={720}>12 hours</option>
                  <option value={1440}>24 hours</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-neutral-300 mb-1">
                  Analytics Retention (days)
                </label>
                <select
                  value={currentSettings.analytics_retention_days || 30}
                  onChange={(e) => updateAdvancedMutation.mutate({ analytics_retention_days: parseInt(e.target.value) })}
                  className="w-full px-3 py-2 border border-neutral-700 rounded-lg bg-neutral-900 text-neutral-100 focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                >
                  <option value={7}>7 days</option>
                  <option value={30}>30 days</option>
                  <option value={90}>90 days</option>
                  <option value={365}>1 year</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-neutral-300 mb-1">
                  Default R2 Bucket
                </label>
                <input
                  type="text"
                  placeholder="my-default-bucket"
                  defaultValue={currentSettings.r2_default_bucket || ''}
                  onBlur={(e) => {
                    if (e.target.value !== currentSettings.r2_default_bucket) {
                      updateAdvancedMutation.mutate({ r2_default_bucket: e.target.value });
                    }
                  }}
                  className="w-full px-3 py-2 border border-neutral-700 rounded-lg bg-neutral-900 text-neutral-100 placeholder-neutral-500 focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                />
              </div>

              <ToggleSetting
                label="Enable Workers"
                description="Allow deploying and managing Cloudflare Workers"
                enabled={currentSettings.workers_enabled}
                onChange={(enabled) => updateAdvancedMutation.mutate({ workers_enabled: enabled })}
              />
            </div>
          </div>
        </div>

        {/* Sidebar */}
        <div className="space-y-6">
          {/* Connection Status */}
          <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
            <h3 className="text-sm font-medium text-neutral-300 mb-4">
              Connection Status
            </h3>

            <div
              className={clsx(
                'flex items-center gap-3 p-4 rounded-lg mb-4',
                connection.connected
                  ? 'bg-success-500/10 border border-success-500/20'
                  : 'bg-danger-500/10 border border-danger-500/20'
              )}
            >
              {connection.connected ? (
                <Check className="w-5 h-5 text-success-400" />
              ) : (
                <AlertTriangle className="w-5 h-5 text-danger-400" />
              )}
              <div>
                <p
                  className={clsx(
                    'font-medium',
                    connection.connected ? 'text-success-300' : 'text-danger-300'
                  )}
                >
                  {connection.connected ? 'Connected' : 'Not Connected'}
                </p>
                {connection.zone_name && (
                  <p className="text-sm text-neutral-400">{connection.zone_name}</p>
                )}
              </div>
            </div>

            {connection.connected && (
              <div className="space-y-3">
                <div className="flex items-center justify-between text-sm">
                  <span className="text-neutral-500">Plan</span>
                  <span className="font-medium text-neutral-200">{connection.plan || 'Free'}</span>
                </div>
                <div className="flex items-center justify-between text-sm">
                  <span className="text-neutral-500">Status</span>
                  <span className="px-2 py-0.5 text-xs bg-success-500/10 text-success-400 border border-success-500/20 rounded-full">
                    {connection.status || 'Active'}
                  </span>
                </div>
              </div>
            )}
          </div>

          {/* Quick Actions */}
          <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
            <h3 className="text-sm font-medium text-neutral-300 mb-4">
              Quick Actions
            </h3>

            <div className="space-y-2">
              <button
                onClick={clearCache}
                className="w-full flex items-center gap-2 px-4 py-2.5 text-left text-sm text-neutral-300 border border-neutral-700 rounded-lg hover:bg-neutral-700 transition-colors"
              >
                <Database className="w-4 h-4 text-neutral-400" />
                Clear Local Cache
              </button>

              <button className="w-full flex items-center gap-2 px-4 py-2.5 text-left text-sm text-neutral-300 border border-neutral-700 rounded-lg hover:bg-neutral-700 transition-colors">
                <RefreshCw className="w-4 h-4 text-neutral-400" />
                Sync DNS Records
              </button>

              <a
                href="https://dash.cloudflare.com"
                target="_blank"
                rel="noopener noreferrer"
                className="w-full flex items-center gap-2 px-4 py-2.5 text-left text-sm text-neutral-300 border border-neutral-700 rounded-lg hover:bg-neutral-700 transition-colors"
              >
                <ExternalLink className="w-4 h-4 text-neutral-400" />
                Open Cloudflare Dashboard
              </a>
            </div>
          </div>

          {/* API Reference */}
          <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
            <h3 className="text-sm font-medium text-neutral-300 mb-4">
              API Endpoints
            </h3>

            <div className="space-y-2 text-sm">
              <div className="flex items-center justify-between">
                <code className="text-xs bg-neutral-900 text-neutral-300 px-2 py-1 rounded">
                  /api/cloudflare/cache
                </code>
                <button className="p-1 text-neutral-400 hover:text-neutral-200 transition-colors">
                  <Copy className="w-3 h-3" />
                </button>
              </div>
              <div className="flex items-center justify-between">
                <code className="text-xs bg-neutral-900 text-neutral-300 px-2 py-1 rounded">
                  /api/cloudflare/dns
                </code>
                <button className="p-1 text-neutral-400 hover:text-neutral-200 transition-colors">
                  <Copy className="w-3 h-3" />
                </button>
              </div>
              <div className="flex items-center justify-between">
                <code className="text-xs bg-neutral-900 text-neutral-300 px-2 py-1 rounded">
                  /api/cloudflare/security
                </code>
                <button className="p-1 text-neutral-400 hover:text-neutral-200 transition-colors">
                  <Copy className="w-3 h-3" />
                </button>
              </div>
            </div>
          </div>

          {/* Danger Zone */}
          <div className="bg-neutral-800 rounded-xl border border-danger-500/30 p-6">
            <h3 className="text-sm font-medium text-danger-400 mb-4">Danger Zone</h3>

            <button className="w-full flex items-center gap-2 px-4 py-2.5 text-left text-sm text-danger-400 border border-danger-500/30 rounded-lg hover:bg-danger-500/10 transition-colors">
              <Trash2 className="w-4 h-4" />
              Reset All Settings
            </button>
          </div>
        </div>
      </div>

      {/* SSO Account/Zone Selection Modal */}
      {showSelectionModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
          <div
            className="absolute inset-0 bg-black/70"
            onClick={() => setShowSelectionModal(false)}
          />
          <div className="relative bg-neutral-800 rounded-xl border border-neutral-700 p-6 w-full max-w-md mx-4 shadow-2xl">
            <button
              onClick={() => setShowSelectionModal(false)}
              className="absolute top-4 right-4 p-1 text-neutral-400 hover:text-neutral-200"
            >
              <X className="w-5 h-5" />
            </button>

            <div className="flex items-center gap-3 mb-6">
              <div className="p-2 bg-primary-500/10 border border-primary-500/20 rounded-lg">
                <Cloud className="w-6 h-6 text-primary-400" />
              </div>
              <div>
                <h2 className="text-lg font-semibold text-neutral-100">Select Zone</h2>
                <p className="text-sm text-neutral-500">Choose your Cloudflare account and zone</p>
              </div>
            </div>

            <div className="space-y-4">
              {/* Account Selection */}
              {ssoAccounts.length > 1 && (
                <div>
                  <label className="block text-sm font-medium text-neutral-300 mb-2">
                    Account
                  </label>
                  <select
                    value={selectedAccount}
                    onChange={(e) => setSelectedAccount(e.target.value)}
                    className="w-full px-3 py-2.5 border border-neutral-700 rounded-lg bg-neutral-900 text-neutral-100 focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                  >
                    <option value="">Select an account...</option>
                    {ssoAccounts.map((account) => (
                      <option key={account.id} value={account.id}>
                        {account.name}
                      </option>
                    ))}
                  </select>
                </div>
              )}

              {/* Zone Selection */}
              <div>
                <label className="block text-sm font-medium text-neutral-300 mb-2">
                  Zone
                </label>
                <select
                  value={selectedZone}
                  onChange={(e) => setSelectedZone(e.target.value)}
                  className="w-full px-3 py-2.5 border border-neutral-700 rounded-lg bg-neutral-900 text-neutral-100 focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                >
                  <option value="">Select a zone...</option>
                  {ssoZones.map((zone) => (
                    <option key={zone.id} value={zone.id}>
                      {zone.name} ({zone.status})
                    </option>
                  ))}
                </select>
              </div>

              <div className="flex gap-3 pt-4">
                <button
                  onClick={() => setShowSelectionModal(false)}
                  className="flex-1 px-4 py-2.5 border border-neutral-700 text-neutral-300 rounded-lg hover:bg-neutral-700 transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={completeSsoConnection}
                  disabled={ssoLoading || !selectedZone}
                  className="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 bg-primary-600 hover:bg-primary-700 text-white rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {ssoLoading ? (
                    <RefreshCw className="w-4 h-4 animate-spin" />
                  ) : (
                    <Check className="w-4 h-4" />
                  )}
                  Connect
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function ToggleSetting({
  label,
  description,
  enabled,
  onChange,
}: {
  label: string;
  description: string;
  enabled: boolean;
  onChange: (enabled: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between">
      <div>
        <p className="font-medium text-neutral-100">{label}</p>
        <p className="text-sm text-neutral-500">{description}</p>
      </div>
      <button
        onClick={() => onChange(!enabled)}
        className={clsx(
          'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
          enabled ? 'bg-primary-600' : 'bg-neutral-700'
        )}
      >
        <span
          className={clsx(
            'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
            enabled ? 'translate-x-6' : 'translate-x-1'
          )}
        />
      </button>
    </div>
  );
}
