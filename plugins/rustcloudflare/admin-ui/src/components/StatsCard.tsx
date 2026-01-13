import { TrendingUp, TrendingDown } from 'lucide-react';
import { clsx } from 'clsx';

interface StatsCardProps {
  title: string;
  value: string;
  icon: React.ReactNode;
  trend?: number;
  color: 'primary' | 'success' | 'warning' | 'danger' | 'accent' | 'info';
}

const colorClasses = {
  primary: 'bg-primary-500/10 text-primary-400 border border-primary-500/20',
  success: 'bg-success-500/10 text-success-400 border border-success-500/20',
  warning: 'bg-warning-500/10 text-warning-400 border border-warning-500/20',
  danger: 'bg-danger-500/10 text-danger-400 border border-danger-500/20',
  accent: 'bg-accent-500/10 text-accent-400 border border-accent-500/20',
  info: 'bg-info-500/10 text-info-400 border border-info-500/20',
};

export function StatsCard({ title, value, icon, trend, color }: StatsCardProps) {
  return (
    <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6 card-hover">
      <div className="flex items-center justify-between">
        <div className={clsx('p-3 rounded-lg', colorClasses[color])}>
          {icon}
        </div>
        {trend !== undefined && (
          <div className={clsx(
            'flex items-center gap-1 text-sm font-medium',
            trend >= 0 ? 'text-success-400' : 'text-danger-400'
          )}>
            {trend >= 0 ? <TrendingUp className="w-4 h-4" /> : <TrendingDown className="w-4 h-4" />}
            {Math.abs(trend)}%
          </div>
        )}
      </div>
      <div className="mt-4">
        <p className="text-2xl font-bold text-neutral-100">{value}</p>
        <p className="text-sm text-neutral-400">{title}</p>
      </div>
    </div>
  );
}
