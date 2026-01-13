import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Zap,
  Image,
  Gauge,
  Globe,
  Smartphone,
  Code,
  RefreshCw,
  CheckCircle,
  ArrowRight,
} from 'lucide-react';
import { cloudflareApi } from '../lib/api';
import toast from 'react-hot-toast';
import { clsx } from 'clsx';

export function PerformancePage() {
  const queryClient = useQueryClient();

  const { data: zoneSettings, isLoading } = useQuery({
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

  const settings = zoneSettings?.data?.data || [];
  const getSetting = (id: string) => {
    const setting = settings.find((s: any) => s.id === id);
    return setting?.value;
  };

  const updateSetting = (id: string, value: any) => {
    updateSettingMutation.mutate({ [id]: value });
  };

  return (
    <div className="p-6">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-neutral-100 flex items-center gap-3"><Zap className="w-7 h-7 text-orange-400" />Performance</h1>
        <p className="text-neutral-400">Optimize your site's speed and performance</p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Speed Optimization */}
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <div className="flex items-center gap-3 mb-6">
            <div className="p-2 bg-blue-500/10 border border-blue-500/20 rounded-lg">
              <Zap className="w-5 h-5 text-blue-400" />
            </div>
            <h2 className="text-lg font-semibold text-neutral-100">Speed</h2>
          </div>

          <div className="space-y-4">
            <ToggleSetting
              label="Auto Minify JavaScript"
              description="Remove unnecessary characters from JS files"
              enabled={getSetting('minify')?.js === 'on'}
              onChange={(enabled) => updateSetting('minify', {
                ...getSetting('minify'),
                js: enabled ? 'on' : 'off'
              })}
            />

            <ToggleSetting
              label="Auto Minify CSS"
              description="Remove unnecessary characters from CSS files"
              enabled={getSetting('minify')?.css === 'on'}
              onChange={(enabled) => updateSetting('minify', {
                ...getSetting('minify'),
                css: enabled ? 'on' : 'off'
              })}
            />

            <ToggleSetting
              label="Auto Minify HTML"
              description="Remove unnecessary characters from HTML"
              enabled={getSetting('minify')?.html === 'on'}
              onChange={(enabled) => updateSetting('minify', {
                ...getSetting('minify'),
                html: enabled ? 'on' : 'off'
              })}
            />

            <ToggleSetting
              label="Brotli Compression"
              description="Enable Brotli compression for faster delivery"
              enabled={getSetting('brotli') === 'on'}
              onChange={(enabled) => updateSetting('brotli', enabled ? 'on' : 'off')}
            />

            <ToggleSetting
              label="Early Hints"
              description="Speed up page loads with 103 responses"
              enabled={getSetting('early_hints') === 'on'}
              onChange={(enabled) => updateSetting('early_hints', enabled ? 'on' : 'off')}
            />

            <ToggleSetting
              label="Rocket Loader"
              description="Improve paint times by async loading scripts"
              enabled={getSetting('rocket_loader') === 'on'}
              onChange={(enabled) => updateSetting('rocket_loader', enabled ? 'on' : 'off')}
            />
          </div>
        </div>

        {/* Protocol Optimization */}
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <div className="flex items-center gap-3 mb-6">
            <div className="p-2 bg-purple-500/10 border border-purple-500/20 rounded-lg">
              <Globe className="w-5 h-5 text-purple-400" />
            </div>
            <h2 className="text-lg font-semibold text-neutral-100">Protocol</h2>
          </div>

          <div className="space-y-4">
            <ToggleSetting
              label="HTTP/2"
              description="Enable HTTP/2 for improved performance"
              enabled={getSetting('http2') === 'on'}
              onChange={(enabled) => updateSetting('http2', enabled ? 'on' : 'off')}
            />

            <ToggleSetting
              label="HTTP/3 (QUIC)"
              description="Enable HTTP/3 with QUIC protocol"
              enabled={getSetting('http3') === 'on'}
              onChange={(enabled) => updateSetting('http3', enabled ? 'on' : 'off')}
            />

            <ToggleSetting
              label="0-RTT Connection Resumption"
              description="Faster connections with session resumption"
              enabled={getSetting('0rtt') === 'on'}
              onChange={(enabled) => updateSetting('0rtt', enabled ? 'on' : 'off')}
            />

            <ToggleSetting
              label="WebSockets"
              description="Allow WebSocket connections"
              enabled={getSetting('websockets') === 'on'}
              onChange={(enabled) => updateSetting('websockets', enabled ? 'on' : 'off')}
            />

            <ToggleSetting
              label="gRPC"
              description="Enable gRPC protocol support"
              enabled={getSetting('grpc') === 'on'}
              onChange={(enabled) => updateSetting('grpc', enabled ? 'on' : 'off')}
            />
          </div>
        </div>

        {/* Image Optimization */}
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <div className="flex items-center gap-3 mb-6">
            <div className="p-2 bg-green-500/10 border border-green-500/20 rounded-lg">
              <Image className="w-5 h-5 text-green-400" />
            </div>
            <h2 className="text-lg font-semibold text-neutral-100">Image Optimization</h2>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-neutral-300 mb-2">Polish (Image Compression)</label>
              <select
                value={getSetting('polish') || 'off'}
                onChange={(e) => updateSetting('polish', e.target.value)}
                className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-900 text-neutral-100"
              >
                <option value="off">Off</option>
                <option value="lossless">Lossless</option>
                <option value="lossy">Lossy (Recommended)</option>
              </select>
            </div>

            <ToggleSetting
              label="WebP Conversion"
              description="Convert images to WebP format"
              enabled={getSetting('webp') === 'on'}
              onChange={(enabled) => updateSetting('webp', enabled ? 'on' : 'off')}
            />

            <ToggleSetting
              label="Mirage"
              description="Optimize image loading for mobile"
              enabled={getSetting('mirage') === 'on'}
              onChange={(enabled) => updateSetting('mirage', enabled ? 'on' : 'off')}
            />

            <ToggleSetting
              label="Image Resizing"
              description="Enable on-the-fly image resizing"
              enabled={getSetting('image_resizing') === 'on'}
              onChange={(enabled) => updateSetting('image_resizing', enabled ? 'on' : 'off')}
            />
          </div>
        </div>

        {/* Argo */}
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <div className="flex items-center gap-3 mb-6">
            <div className="p-2 bg-primary-500/10 border border-primary-500/20 rounded-lg">
              <Gauge className="w-5 h-5 text-primary-400" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-neutral-100">Argo</h2>
              <span className="text-xs px-2 py-0.5 bg-primary-500/10 text-primary-400 border border-primary-500/20 rounded-full">
                Premium
              </span>
            </div>
          </div>

          <div className="space-y-4">
            <ToggleSetting
              label="Argo Smart Routing"
              description="Route traffic through fastest paths"
              enabled={getSetting('argo')?.smart_routing === 'on'}
              onChange={(enabled) => updateSetting('argo', {
                ...getSetting('argo'),
                smart_routing: enabled ? 'on' : 'off'
              })}
            />

            <ToggleSetting
              label="Argo Tiered Caching"
              description="Reduce origin requests with tiered caching"
              enabled={getSetting('argo')?.tiered_caching === 'on'}
              onChange={(enabled) => updateSetting('argo', {
                ...getSetting('argo'),
                tiered_caching: enabled ? 'on' : 'off'
              })}
            />

            <div className="pt-4 border-t border-neutral-700/50">
              <p className="text-sm text-neutral-400">
                Argo uses real-time network data to route around congestion, packet loss, and outages.
              </p>
            </div>
          </div>
        </div>

        {/* Mobile Optimization */}
        <div className="lg:col-span-2 bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <div className="flex items-center gap-3 mb-6">
            <div className="p-2 bg-indigo-500/10 border border-indigo-500/20 rounded-lg">
              <Smartphone className="w-5 h-5 text-indigo-400" />
            </div>
            <h2 className="text-lg font-semibold text-neutral-100">Mobile Optimization</h2>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="p-4 border border-neutral-700/50 rounded-lg bg-neutral-700/30">
              <div className="flex items-center gap-2 mb-2">
                <CheckCircle className="w-5 h-5 text-green-400" />
                <span className="font-medium text-neutral-100">Mobile Redirect</span>
              </div>
              <p className="text-sm text-neutral-400">
                Redirect mobile visitors to a mobile-optimized subdomain
              </p>
            </div>

            <div className="p-4 border border-neutral-700/50 rounded-lg bg-neutral-700/30">
              <div className="flex items-center gap-2 mb-2">
                <CheckCircle className="w-5 h-5 text-green-400" />
                <span className="font-medium text-neutral-100">AMP Real URL</span>
              </div>
              <p className="text-sm text-neutral-400">
                Display your domain in AMP viewer instead of Google's
              </p>
            </div>

            <div className="p-4 border border-neutral-700/50 rounded-lg bg-neutral-700/30">
              <div className="flex items-center gap-2 mb-2">
                <CheckCircle className="w-5 h-5 text-green-400" />
                <span className="font-medium text-neutral-100">Prefetch URLs</span>
              </div>
              <p className="text-sm text-neutral-400">
                Prefetch linked content for faster navigation
              </p>
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
