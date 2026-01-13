import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Lock,
  Shield,
  CheckCircle,
  AlertTriangle,
  RefreshCw,
  ExternalLink,
  Clock,
} from 'lucide-react';
import { cloudflareApi } from '../lib/api';
import toast from 'react-hot-toast';
import { clsx } from 'clsx';

const sslModes = [
  {
    value: 'off',
    label: 'Off',
    description: 'No encryption between visitor and Cloudflare, and no encryption between Cloudflare and your origin',
    icon: '🔓',
    danger: true,
  },
  {
    value: 'flexible',
    label: 'Flexible',
    description: 'Encryption between visitor and Cloudflare only. No certificate required on origin.',
    icon: '🔐',
    warning: true,
  },
  {
    value: 'full',
    label: 'Full',
    description: 'Encryption between visitor and Cloudflare, and between Cloudflare and origin. Self-signed certificate allowed.',
    icon: '🔒',
  },
  {
    value: 'strict',
    label: 'Full (Strict)',
    description: 'Encryption everywhere. Valid CA-signed certificate required on origin.',
    icon: '🛡️',
    recommended: true,
  },
];

export function SslPage() {
  const queryClient = useQueryClient();

  const { data: sslStatus, isLoading } = useQuery({
    queryKey: ['ssl-status'],
    queryFn: () => cloudflareApi.getSslStatus(),
  });

  const { data: certificates } = useQuery({
    queryKey: ['ssl-certificates'],
    queryFn: () => cloudflareApi.listCertificates(),
  });

  const { data: zoneSettings } = useQuery({
    queryKey: ['zone-settings'],
    queryFn: () => cloudflareApi.getZoneSettings(),
  });

  const updateSettingMutation = useMutation({
    mutationFn: (settings: any) => cloudflareApi.updateZoneSettings(settings),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['zone-settings'] });
      toast.success('Settings updated');
    },
    onError: () => toast.error('Failed to update settings'),
  });

  const currentMode = sslStatus?.data?.data?.value || 'full';
  const certs = certificates?.data?.data || [];
  const settings = zoneSettings?.data?.data || [];

  const getSetting = (id: string) => {
    const setting = settings.find((s: any) => s.id === id);
    return setting?.value;
  };

  return (
    <div className="p-6">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-neutral-100 flex items-center gap-3"><Lock className="w-7 h-7 text-orange-400" />SSL/TLS</h1>
        <p className="text-neutral-400">Manage encryption and certificates</p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* SSL Mode */}
        <div className="lg:col-span-2 bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <h2 className="text-lg font-semibold text-neutral-100 mb-4">
            SSL/TLS Encryption Mode
          </h2>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            {sslModes.map((mode) => (
              <button
                key={mode.value}
                onClick={() => updateSettingMutation.mutate({ ssl: mode.value })}
                disabled={updateSettingMutation.isPending}
                className={clsx(
                  'p-4 rounded-lg border-2 transition-colors text-left',
                  currentMode === mode.value
                    ? 'border-primary-500 bg-primary-500/10'
                    : 'border-neutral-700/50 hover:border-neutral-600'
                )}
              >
                <div className="text-2xl mb-2">{mode.icon}</div>
                <div className="flex items-center gap-2 mb-1">
                  <span className="font-semibold text-neutral-100">{mode.label}</span>
                  {mode.recommended && (
                    <span className="text-xs px-1.5 py-0.5 bg-green-500/10 text-green-400 border border-green-500/20 rounded">
                      Recommended
                    </span>
                  )}
                </div>
                <p className="text-xs text-neutral-400">{mode.description}</p>
                {mode.danger && (
                  <p className="text-xs text-red-400 mt-2 flex items-center gap-1">
                    <AlertTriangle className="w-3 h-3" />
                    Not recommended
                  </p>
                )}
                {mode.warning && (
                  <p className="text-xs text-yellow-400 mt-2 flex items-center gap-1">
                    <AlertTriangle className="w-3 h-3" />
                    Partial encryption
                  </p>
                )}
              </button>
            ))}
          </div>
        </div>

        {/* SSL Settings */}
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <h2 className="text-lg font-semibold text-neutral-100 mb-4">
            SSL Settings
          </h2>

          <div className="space-y-4">
            <ToggleSetting
              label="Always Use HTTPS"
              description="Redirect all HTTP requests to HTTPS"
              enabled={getSetting('always_use_https') === 'on'}
              onChange={(enabled) => updateSettingMutation.mutate({ always_use_https: enabled ? 'on' : 'off' })}
            />

            <ToggleSetting
              label="Automatic HTTPS Rewrites"
              description="Fix mixed content by rewriting HTTP to HTTPS"
              enabled={getSetting('automatic_https_rewrites') === 'on'}
              onChange={(enabled) => updateSettingMutation.mutate({ automatic_https_rewrites: enabled ? 'on' : 'off' })}
            />

            <ToggleSetting
              label="Opportunistic Encryption"
              description="Enable HTTP/2 opportunistic encryption"
              enabled={getSetting('opportunistic_encryption') === 'on'}
              onChange={(enabled) => updateSettingMutation.mutate({ opportunistic_encryption: enabled ? 'on' : 'off' })}
            />

            <div className="pt-4 border-t border-neutral-700/50">
              <label className="block text-sm font-medium text-neutral-300 mb-2">Minimum TLS Version</label>
              <select
                value={getSetting('min_tls_version') || '1.2'}
                onChange={(e) => updateSettingMutation.mutate({ min_tls_version: e.target.value })}
                className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-900 text-neutral-100"
              >
                <option value="1.0">TLS 1.0</option>
                <option value="1.1">TLS 1.1</option>
                <option value="1.2">TLS 1.2 (Recommended)</option>
                <option value="1.3">TLS 1.3</option>
              </select>
            </div>
          </div>
        </div>

        {/* Certificates */}
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <h2 className="text-lg font-semibold text-neutral-100 mb-4">
            SSL Certificates
          </h2>

          <div className="space-y-3">
            {certs.length === 0 ? (
              <div className="text-center py-8 text-neutral-500">
                <Lock className="w-12 h-12 mx-auto mb-3 opacity-50" />
                <p>No certificates found</p>
                <p className="text-sm">Cloudflare manages your edge certificates automatically</p>
              </div>
            ) : (
              certs.map((cert: any) => (
                <div
                  key={cert.id}
                  className="p-4 border border-neutral-700/50 rounded-lg bg-neutral-700/30"
                >
                  <div className="flex items-start justify-between">
                    <div>
                      <div className="flex items-center gap-2">
                        <CheckCircle className="w-4 h-4 text-green-400" />
                        <span className="font-medium text-neutral-100">{cert.type}</span>
                      </div>
                      <p className="text-sm text-neutral-400 mt-1">
                        Hosts: {cert.hosts?.join(', ')}
                      </p>
                    </div>
                    <span className={clsx(
                      'px-2 py-1 text-xs rounded-full border',
                      cert.status === 'active' ? 'bg-green-500/10 text-green-400 border-green-500/20' : 'bg-yellow-500/10 text-yellow-400 border-yellow-500/20'
                    )}>
                      {cert.status}
                    </span>
                  </div>
                  {cert.expires_on && (
                    <p className="text-xs text-neutral-400 mt-2 flex items-center gap-1">
                      <Clock className="w-3 h-3" />
                      Expires: {new Date(cert.expires_on).toLocaleDateString()}
                    </p>
                  )}
                </div>
              ))
            )}
          </div>
        </div>

        {/* Edge Certificate */}
        <div className="lg:col-span-2 bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 bg-green-500/10 border border-green-500/20 rounded-lg">
              <Shield className="w-5 h-5 text-green-400" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-neutral-100">
                Universal SSL Certificate
              </h2>
              <p className="text-sm text-neutral-400">
                Cloudflare provides free SSL certificates for all zones
              </p>
            </div>
          </div>

          <div className="bg-green-500/10 border border-green-500/20 rounded-lg p-4">
            <div className="flex items-center gap-2 text-green-400">
              <CheckCircle className="w-5 h-5" />
              <span className="font-medium">Universal SSL is active</span>
            </div>
            <p className="text-sm text-green-300 mt-1">
              Your site is protected with a free Cloudflare SSL certificate that automatically renews.
            </p>
          </div>

          <div className="mt-4 grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="p-3 bg-neutral-700/50 rounded-lg">
              <p className="text-sm text-neutral-400">Certificate Authority</p>
              <p className="font-medium text-neutral-100">Let's Encrypt</p>
            </div>
            <div className="p-3 bg-neutral-700/50 rounded-lg">
              <p className="text-sm text-neutral-400">Signature</p>
              <p className="font-medium text-neutral-100">ECDSA</p>
            </div>
            <div className="p-3 bg-neutral-700/50 rounded-lg">
              <p className="text-sm text-neutral-400">Validity</p>
              <p className="font-medium text-neutral-100">90 days</p>
            </div>
            <div className="p-3 bg-neutral-700/50 rounded-lg">
              <p className="text-sm text-neutral-400">Auto Renewal</p>
              <p className="font-medium text-green-400">Enabled</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function ToggleSetting({ label, description, enabled, onChange }: {
  label: string;
  description: string;
  enabled: boolean;
  onChange: (enabled: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between">
      <div>
        <p className="font-medium text-neutral-100">{label}</p>
        <p className="text-sm text-neutral-400">{description}</p>
      </div>
      <button
        onClick={() => onChange(!enabled)}
        className={clsx(
          'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
          enabled ? 'bg-primary-600' : 'bg-neutral-600'
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
