import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
  BarChart,
  Bar,
  LineChart,
  Line,
  AreaChart,
  Area,
  PieChart,
  Pie,
  Cell,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import {
  Activity,
  Globe,
  Shield,
  Zap,
  TrendingUp,
  TrendingDown,
  Users,
  Eye,
  Clock,
  RefreshCw,
  Download,
  Server,
  Wifi,
  MapPin,
} from 'lucide-react';
import { cloudflareApi } from '../lib/api';
import { clsx } from 'clsx';
import { Skeleton, StatsSkeleton, ChartSkeleton } from '../components/Skeleton';

type TimeRange = '1h' | '6h' | '24h' | '7d' | '30d';

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

const formatNumber = (num: number): string => {
  if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
  if (num >= 1000) return (num / 1000).toFixed(1) + 'K';
  return num.toString();
};

const COLORS = ['#f97316', '#3b82f6', '#10b981', '#8b5cf6', '#ec4899', '#06b6d4'];

export function AnalyticsPage() {
  const [timeRange, setTimeRange] = useState<TimeRange>('24h');

  const { data: analytics, isLoading } = useQuery({
    queryKey: ['analytics', timeRange],
    queryFn: () => cloudflareApi.getAnalytics(timeRange),
  });

  const { data: zoneAnalytics, isLoading: zoneLoading } = useQuery({
    queryKey: ['zone-analytics', timeRange],
    queryFn: () => cloudflareApi.getZoneAnalytics(),
  });

  const stats = analytics?.data?.data || {};
  const trafficData = zoneAnalytics?.data?.data?.timeseries || [];
  const countryData = zoneAnalytics?.data?.data?.countries || [];
  const contentTypeData = zoneAnalytics?.data?.data?.content_types || [];
  const statusCodeData = zoneAnalytics?.data?.data?.status_codes || [];
  const threatData = zoneAnalytics?.data?.data?.threats || [];

  const cacheHitRatio = stats.requests?.all > 0
    ? ((stats.requests?.cached || 0) / stats.requests.all * 100).toFixed(1)
    : '0';

  const timeRanges = [
    { id: '1h' as const, label: '1 Hour' },
    { id: '6h' as const, label: '6 Hours' },
    { id: '24h' as const, label: '24 Hours' },
    { id: '7d' as const, label: '7 Days' },
    { id: '30d' as const, label: '30 Days' },
  ];

  return (
    <div className="p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-neutral-100 flex items-center gap-3"><Activity className="w-7 h-7 text-orange-400" />Analytics</h1>
          <p className="text-neutral-400">
            Traffic insights and performance metrics
          </p>
        </div>
        <div className="flex items-center gap-3">
          <div className="flex bg-neutral-700/50 rounded-lg p-1">
            {timeRanges.map((range) => (
              <button
                key={range.id}
                onClick={() => setTimeRange(range.id)}
                className={clsx(
                  'px-3 py-1.5 text-sm rounded-md transition-colors',
                  timeRange === range.id
                    ? 'bg-neutral-600 shadow text-neutral-100'
                    : 'text-neutral-400 hover:text-neutral-100'
                )}
              >
                {range.label}
              </button>
            ))}
          </div>
          <button className="flex items-center gap-2 px-4 py-2 border border-neutral-600 rounded-lg hover:bg-neutral-700/50 text-neutral-300">
            <Download className="w-4 h-4" />
            Export
          </button>
        </div>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        {isLoading ? (
          <>
            <StatsSkeleton />
            <StatsSkeleton />
            <StatsSkeleton />
            <StatsSkeleton />
          </>
        ) : (
          <>
            <StatCard
              title="Total Requests"
              value={formatNumber(stats.requests?.all || 0)}
              change="+12.5%"
              trend="up"
              icon={Activity}
              color="orange"
            />
            <StatCard
              title="Bandwidth Served"
              value={formatBytes(stats.bandwidth?.all || 0)}
              change="+8.2%"
              trend="up"
              icon={Wifi}
              color="blue"
            />
            <StatCard
              title="Cache Hit Ratio"
              value={`${cacheHitRatio}%`}
              change="+2.1%"
              trend="up"
              icon={Zap}
              color="green"
            />
            <StatCard
              title="Threats Blocked"
              value={formatNumber(stats.threats?.all || 0)}
              change="-5.3%"
              trend="down"
              icon={Shield}
              color="purple"
            />
          </>
        )}
      </div>

      {/* Charts Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        {/* Traffic Chart */}
        {zoneLoading ? (
          <ChartSkeleton height={300} />
        ) : (
          <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
            <h3 className="text-lg font-semibold text-neutral-100 mb-4">
              Traffic Overview
            </h3>
            {trafficData.length > 0 ? (
              <ResponsiveContainer width="100%" height={300}>
                <AreaChart data={trafficData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#374151" opacity={0.2} />
                  <XAxis dataKey="time" stroke="#9ca3af" />
                  <YAxis stroke="#9ca3af" />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: '#262626',
                      border: '1px solid #404040',
                      borderRadius: '8px',
                      color: '#fff',
                    }}
                  />
                  <Legend />
                  <Area
                    type="monotone"
                    dataKey="requests"
                    name="Total Requests"
                    stroke="#f97316"
                    fill="#f97316"
                    fillOpacity={0.2}
                  />
                  <Area
                    type="monotone"
                    dataKey="cached"
                    name="Cached"
                    stroke="#10b981"
                    fill="#10b981"
                    fillOpacity={0.2}
                  />
                </AreaChart>
              </ResponsiveContainer>
            ) : (
              <div className="h-[300px] flex items-center justify-center text-neutral-500">
                <div className="text-center">
                  <Activity className="w-12 h-12 mx-auto mb-3 opacity-50" />
                  <p>No traffic data available</p>
                </div>
              </div>
            )}
          </div>
        )}

        {/* Bandwidth Chart */}
        {zoneLoading ? (
          <ChartSkeleton height={300} />
        ) : (
          <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
            <h3 className="text-lg font-semibold text-neutral-100 mb-4">
              Bandwidth Usage
            </h3>
            {trafficData.length > 0 ? (
              <ResponsiveContainer width="100%" height={300}>
                <LineChart data={trafficData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#374151" opacity={0.2} />
                  <XAxis dataKey="time" stroke="#9ca3af" />
                  <YAxis stroke="#9ca3af" tickFormatter={(v) => formatBytes(v)} />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: '#262626',
                      border: '1px solid #404040',
                      borderRadius: '8px',
                      color: '#fff',
                    }}
                    formatter={(value: number) => formatBytes(value)}
                  />
                  <Line
                    type="monotone"
                    dataKey="bandwidth"
                    name="Bandwidth"
                    stroke="#3b82f6"
                    strokeWidth={2}
                    dot={false}
                  />
                </LineChart>
              </ResponsiveContainer>
            ) : (
              <div className="h-[300px] flex items-center justify-center text-neutral-500">
                <div className="text-center">
                  <Wifi className="w-12 h-12 mx-auto mb-3 opacity-50" />
                  <p>No bandwidth data available</p>
                </div>
              </div>
            )}
          </div>
        )}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-6">
        {/* Country Distribution */}
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <h3 className="text-lg font-semibold text-neutral-100 mb-4 flex items-center gap-2">
            <MapPin className="w-5 h-5 text-primary-500" />
            Top Countries
          </h3>
          {zoneLoading ? (
            <div className="space-y-3">
              {Array.from({ length: 5 }).map((_, i) => (
                <div key={i}>
                  <div className="flex items-center justify-between mb-1">
                    <Skeleton width={100} height={16} />
                    <Skeleton width={40} height={16} />
                  </div>
                  <Skeleton height={8} className="w-full rounded-full" />
                </div>
              ))}
            </div>
          ) : countryData.length > 0 ? (
            <div className="space-y-3">
              {countryData.slice(0, 6).map((country: any, index: number) => (
                <div key={country.name}>
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-sm font-medium text-neutral-200">{country.name}</span>
                    <span className="text-sm text-neutral-400">{country.percentage}%</span>
                  </div>
                  <div className="w-full bg-neutral-700 rounded-full h-2">
                    <div
                      className="h-2 rounded-full"
                      style={{
                        width: `${country.percentage}%`,
                        backgroundColor: COLORS[index % COLORS.length],
                      }}
                    />
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="h-48 flex items-center justify-center text-neutral-500">
              <p>No country data available</p>
            </div>
          )}
        </div>

        {/* Content Types */}
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <h3 className="text-lg font-semibold text-neutral-100 mb-4 flex items-center gap-2">
            <Server className="w-5 h-5 text-info-500" />
            Content Types
          </h3>
          {zoneLoading ? (
            <div className="h-[220px] flex items-center justify-center">
              <Skeleton variant="circular" width={160} height={160} />
            </div>
          ) : contentTypeData.length > 0 ? (
            <>
              <ResponsiveContainer width="100%" height={220}>
                <PieChart>
                  <Pie
                    data={contentTypeData}
                    cx="50%"
                    cy="50%"
                    innerRadius={50}
                    outerRadius={80}
                    paddingAngle={2}
                    dataKey="value"
                  >
                    {contentTypeData.map((_: any, index: number) => (
                      <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                    ))}
                  </Pie>
                  <Tooltip
                    contentStyle={{
                      backgroundColor: '#262626',
                      border: '1px solid #404040',
                      borderRadius: '8px',
                      color: '#fff',
                    }}
                  />
                </PieChart>
              </ResponsiveContainer>
              <div className="flex flex-wrap justify-center gap-3 mt-2">
                {contentTypeData.map((item: any, index: number) => (
                  <div key={item.name} className="flex items-center gap-1 text-xs text-neutral-300">
                    <div
                      className="w-2 h-2 rounded-full"
                      style={{ backgroundColor: COLORS[index % COLORS.length] }}
                    />
                    {item.name}
                  </div>
                ))}
              </div>
            </>
          ) : (
            <div className="h-48 flex items-center justify-center text-neutral-500">
              <p>No content type data available</p>
            </div>
          )}
        </div>

        {/* Status Codes */}
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <h3 className="text-lg font-semibold text-neutral-100 mb-4 flex items-center gap-2">
            <Activity className="w-5 h-5 text-success-500" />
            Status Codes
          </h3>
          {zoneLoading ? (
            <div className="space-y-4">
              {Array.from({ length: 4 }).map((_, i) => (
                <div key={i}>
                  <div className="flex items-center justify-between mb-1">
                    <Skeleton width={50} height={20} />
                    <Skeleton width={80} height={16} />
                  </div>
                  <Skeleton height={8} className="w-full rounded-full" />
                </div>
              ))}
            </div>
          ) : statusCodeData.length > 0 ? (
            <div className="space-y-4">
              {statusCodeData.map((status: any) => (
                <div key={status.code}>
                  <div className="flex items-center justify-between mb-1">
                    <span
                      className={clsx(
                        'text-sm font-medium px-2 py-0.5 rounded border',
                        status.code === '2xx' && 'bg-success-500/10 text-success-400 border-success-500/20',
                        status.code === '3xx' && 'bg-info-500/10 text-info-400 border-info-500/20',
                        status.code === '4xx' && 'bg-warning-500/10 text-warning-400 border-warning-500/20',
                        status.code === '5xx' && 'bg-danger-500/10 text-danger-400 border-danger-500/20'
                      )}
                    >
                      {status.code}
                    </span>
                    <span className="text-sm text-neutral-400">
                      {formatNumber(status.count)} ({status.percentage}%)
                    </span>
                  </div>
                  <div className="w-full bg-neutral-700 rounded-full h-2">
                    <div
                      className={clsx(
                        'h-2 rounded-full',
                        status.code === '2xx' && 'bg-success-500',
                        status.code === '3xx' && 'bg-info-500',
                        status.code === '4xx' && 'bg-warning-500',
                        status.code === '5xx' && 'bg-danger-500'
                      )}
                      style={{ width: `${status.percentage}%` }}
                    />
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="h-48 flex items-center justify-center text-neutral-500">
              <p>No status code data available</p>
            </div>
          )}
        </div>
      </div>

      {/* Security Analytics */}
      {zoneLoading ? (
        <ChartSkeleton height={250} />
      ) : (
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <h3 className="text-lg font-semibold text-neutral-100 mb-4 flex items-center gap-2">
            <Shield className="w-5 h-5 text-accent-500" />
            Security Events
          </h3>
          {threatData.length > 0 ? (
            <ResponsiveContainer width="100%" height={250}>
              <BarChart data={threatData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" opacity={0.2} />
                <XAxis dataKey="time" stroke="#9ca3af" />
                <YAxis stroke="#9ca3af" />
                <Tooltip
                  contentStyle={{
                    backgroundColor: '#262626',
                    border: '1px solid #404040',
                    borderRadius: '8px',
                    color: '#fff',
                  }}
                />
                <Legend />
                <Bar dataKey="blocked" name="Blocked" fill="#ef4444" radius={[4, 4, 0, 0]} />
                <Bar dataKey="challenged" name="Challenged" fill="#f97316" radius={[4, 4, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          ) : (
            <div className="h-[250px] flex items-center justify-center text-neutral-500">
              <div className="text-center">
                <Shield className="w-12 h-12 mx-auto mb-3 opacity-50" />
                <p>No security events data available</p>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Real-time Stats */}
      <div className="mt-6 grid grid-cols-1 md:grid-cols-3 gap-6">
        {isLoading ? (
          <>
            <StatsSkeleton />
            <StatsSkeleton />
            <StatsSkeleton />
          </>
        ) : (
          <>
            <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
              <div className="flex items-center gap-3 mb-4">
                <div className="p-2 bg-success-500/10 border border-success-500/20 rounded-lg">
                  <Users className="w-5 h-5 text-success-400" />
                </div>
                <div>
                  <p className="text-sm text-neutral-400">Unique Visitors</p>
                  <p className="text-2xl font-bold text-neutral-100">{formatNumber(stats.uniques?.all || 0)}</p>
                </div>
              </div>
              <p className="text-sm text-success-400 flex items-center gap-1">
                <TrendingUp className="w-4 h-4" />
                +15.3% from last period
              </p>
            </div>

            <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
              <div className="flex items-center gap-3 mb-4">
                <div className="p-2 bg-info-500/10 border border-info-500/20 rounded-lg">
                  <Eye className="w-5 h-5 text-info-400" />
                </div>
                <div>
                  <p className="text-sm text-neutral-400">Page Views</p>
                  <p className="text-2xl font-bold text-neutral-100">{formatNumber(stats.pageviews?.all || 0)}</p>
                </div>
              </div>
              <p className="text-sm text-info-400 flex items-center gap-1">
                <TrendingUp className="w-4 h-4" />
                +8.7% from last period
              </p>
            </div>

            <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
              <div className="flex items-center gap-3 mb-4">
                <div className="p-2 bg-accent-500/10 border border-accent-500/20 rounded-lg">
                  <Clock className="w-5 h-5 text-accent-400" />
                </div>
                <div>
                  <p className="text-sm text-neutral-400">Avg Response Time</p>
                  <p className="text-2xl font-bold text-neutral-100">{stats.performance?.avg_response_time || 0}ms</p>
                </div>
              </div>
              <p className="text-sm text-success-400 flex items-center gap-1">
                <TrendingDown className="w-4 h-4" />
                -12% improvement
              </p>
            </div>
          </>
        )}
      </div>
    </div>
  );
}

function StatCard({
  title,
  value,
  change,
  trend,
  icon: Icon,
  color,
}: {
  title: string;
  value: string;
  change: string;
  trend: 'up' | 'down';
  icon: any;
  color: 'orange' | 'blue' | 'green' | 'purple';
}) {
  const colorClasses = {
    orange: 'bg-primary-500/10 border border-primary-500/20 text-primary-400',
    blue: 'bg-info-500/10 border border-info-500/20 text-info-400',
    green: 'bg-success-500/10 border border-success-500/20 text-success-400',
    purple: 'bg-accent-500/10 border border-accent-500/20 text-accent-400',
  };

  return (
    <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
      <div className="flex items-center justify-between mb-4">
        <div className={clsx('p-2 rounded-lg', colorClasses[color])}>
          <Icon className="w-5 h-5" />
        </div>
        <span
          className={clsx(
            'text-sm font-medium flex items-center gap-1',
            trend === 'up' ? 'text-success-400' : 'text-danger-400'
          )}
        >
          {trend === 'up' ? <TrendingUp className="w-4 h-4" /> : <TrendingDown className="w-4 h-4" />}
          {change}
        </span>
      </div>
      <p className="text-sm text-neutral-400">{title}</p>
      <p className="text-2xl font-bold text-neutral-100">{value}</p>
    </div>
  );
}
