import { useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
  Globe,
  Shield,
  Zap,
  Database,
  AlertTriangle,
  CheckCircle,
  LayoutDashboard,
} from 'lucide-react';
import { cloudflareApi } from '../lib/api';
import { useCloudflareStore } from '../stores/cloudflareStore';
import { StatsCard } from '../components/StatsCard';
import { TrafficChart } from '../components/TrafficChart';
import { QuickActions } from '../components/QuickActions';

export function DashboardPage() {
  const { isConnected, zone, checkConnection, fetchZone } = useCloudflareStore();

  useEffect(() => {
    checkConnection();
    fetchZone();
  }, []);

  const { data: analytics } = useQuery({
    queryKey: ['cloudflare-analytics'],
    queryFn: () => cloudflareApi.getTrafficSummary(),
    enabled: isConnected,
  });

  const { data: cacheStats } = useQuery({
    queryKey: ['cloudflare-cache-stats'],
    queryFn: () => cloudflareApi.getCacheStats(),
    enabled: isConnected,
  });

  const trafficData = analytics?.data?.data;

  return (
    <div className="p-6">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-neutral-100 flex items-center gap-3">
            <LayoutDashboard className="w-7 h-7 text-orange-400" />
          Cloudflare Dashboard
        </h1>
        <p className="text-neutral-400">
          Monitor and manage your Cloudflare integration
        </p>
      </div>

      {/* Connection Status Banner */}
      {!isConnected && (
        <div className="mb-6 p-4 bg-warning-500/10 border border-warning-500/20 rounded-lg flex items-center gap-3">
          <AlertTriangle className="w-5 h-5 text-warning-400" />
          <div>
            <p className="font-medium text-warning-300">
              Not Connected to Cloudflare
            </p>
            <p className="text-sm text-warning-400/80">
              Configure your API credentials in Settings to enable Cloudflare features.
            </p>
          </div>
        </div>
      )}

      {isConnected && (
        <div className="mb-6 p-4 bg-success-500/10 border border-success-500/20 rounded-lg flex items-center gap-3">
          <CheckCircle className="w-5 h-5 text-success-400" />
          <div>
            <p className="font-medium text-success-300">
              Connected to Cloudflare
            </p>
            <p className="text-sm text-success-400/80">
              Zone: {zone?.name} ({zone?.status})
            </p>
          </div>
        </div>
      )}

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <StatsCard
          title="Total Requests"
          value={trafficData?.summary?.requests?.toLocaleString() || '1.25M'}
          icon={<Globe className="w-5 h-5" />}
          trend={+12.5}
          color="primary"
        />
        <StatsCard
          title="Cache Hit Rate"
          value={`${trafficData?.cache_hit_rate?.toFixed(1) || '78.4'}%`}
          icon={<Database className="w-5 h-5" />}
          trend={+2.3}
          color="success"
        />
        <StatsCard
          title="Threats Blocked"
          value={trafficData?.summary?.threats_blocked?.toLocaleString() || '12.5K'}
          icon={<Shield className="w-5 h-5" />}
          trend={-5.2}
          color="warning"
        />
        <StatsCard
          title="Bandwidth Saved"
          value={formatBytes(trafficData?.summary?.bandwidth || 38200000000)}
          icon={<Zap className="w-5 h-5" />}
          trend={+8.7}
          color="accent"
        />
      </div>

      {/* Main Content Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Traffic Chart */}
        <div className="lg:col-span-2 bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <h2 className="text-lg font-semibold text-neutral-100 mb-4">
            Traffic Overview
          </h2>
          <TrafficChart />
        </div>

        {/* Quick Actions */}
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <h2 className="text-lg font-semibold text-neutral-100 mb-4">
            Quick Actions
          </h2>
          <QuickActions />
        </div>
      </div>

      {/* Zone Info */}
      {zone && (
        <div className="mt-6 bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <h2 className="text-lg font-semibold text-neutral-100 mb-4">
            Zone Information
          </h2>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div>
              <p className="text-sm text-neutral-500">Zone ID</p>
              <p className="font-mono text-sm text-neutral-300">{zone.id}</p>
            </div>
            <div>
              <p className="text-sm text-neutral-500">Domain</p>
              <p className="font-medium text-neutral-200">{zone.name}</p>
            </div>
            <div>
              <p className="text-sm text-neutral-500">Status</p>
              <span className={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium ${
                zone.status === 'active'
                  ? 'bg-success-500/10 text-success-400 border border-success-500/20'
                  : 'bg-warning-500/10 text-warning-400 border border-warning-500/20'
              }`}>
                {zone.status}
              </span>
            </div>
            <div>
              <p className="text-sm text-neutral-500">Plan</p>
              <p className="font-medium text-neutral-200">{zone.plan?.name || 'Free'}</p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}
