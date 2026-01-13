import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';

// Mock data - would come from API
const data = [
  { time: '00:00', requests: 4000, cached: 2400 },
  { time: '04:00', requests: 3000, cached: 1398 },
  { time: '08:00', requests: 9800, cached: 7800 },
  { time: '12:00', requests: 15000, cached: 12000 },
  { time: '16:00', requests: 12000, cached: 9600 },
  { time: '20:00', requests: 8000, cached: 6400 },
  { time: '24:00', requests: 5000, cached: 4000 },
];

export function TrafficChart() {
  return (
    <ResponsiveContainer width="100%" height={300}>
      <AreaChart data={data}>
        <defs>
          <linearGradient id="colorRequests" x1="0" y1="0" x2="0" y2="1">
            <stop offset="5%" stopColor="#6366f1" stopOpacity={0.3} />
            <stop offset="95%" stopColor="#6366f1" stopOpacity={0} />
          </linearGradient>
          <linearGradient id="colorCached" x1="0" y1="0" x2="0" y2="1">
            <stop offset="5%" stopColor="#22c55e" stopOpacity={0.3} />
            <stop offset="95%" stopColor="#22c55e" stopOpacity={0} />
          </linearGradient>
        </defs>
        <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
        <XAxis
          dataKey="time"
          stroke="#64748b"
          tick={{ fontSize: 12, fill: '#94a3b8' }}
        />
        <YAxis
          stroke="#64748b"
          tick={{ fontSize: 12, fill: '#94a3b8' }}
          tickFormatter={(value) => `${(value / 1000).toFixed(0)}k`}
        />
        <Tooltip
          contentStyle={{
            backgroundColor: 'rgba(30, 41, 59, 0.95)',
            border: '1px solid rgba(99, 102, 241, 0.2)',
            borderRadius: '8px',
            color: '#f1f5f9',
          }}
          labelStyle={{ color: '#94a3b8' }}
        />
        <Area
          type="monotone"
          dataKey="requests"
          stroke="#6366f1"
          strokeWidth={2}
          fillOpacity={1}
          fill="url(#colorRequests)"
          name="Total Requests"
        />
        <Area
          type="monotone"
          dataKey="cached"
          stroke="#22c55e"
          strokeWidth={2}
          fillOpacity={1}
          fill="url(#colorCached)"
          name="Cached"
        />
      </AreaChart>
    </ResponsiveContainer>
  );
}
