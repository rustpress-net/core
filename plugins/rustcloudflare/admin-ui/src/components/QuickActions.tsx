import { useState } from 'react';
import { Trash2, AlertTriangle, RefreshCw, Code } from 'lucide-react';
import { useCloudflareStore } from '../stores/cloudflareStore';
import toast from 'react-hot-toast';
import { clsx } from 'clsx';

const colorClasses = {
  danger: 'bg-danger-500/10 text-danger-400 border border-danger-500/20',
  warning: 'bg-warning-500/10 text-warning-400 border border-warning-500/20',
  success: 'bg-success-500/10 text-success-400 border border-success-500/20',
  primary: 'bg-primary-500/10 text-primary-400 border border-primary-500/20',
  accent: 'bg-accent-500/10 text-accent-400 border border-accent-500/20',
};

export function QuickActions() {
  const { purgeAllCache, setUnderAttack, isUnderAttack, isLoading } = useCloudflareStore();
  const [isPurging, setIsPurging] = useState(false);
  const [isTogglingAttack, setIsTogglingAttack] = useState(false);

  const handlePurgeAll = async () => {
    if (!confirm('Are you sure you want to purge the entire cache?')) return;

    setIsPurging(true);
    try {
      await purgeAllCache();
      toast.success('Cache purged successfully');
    } catch (error) {
      toast.error('Failed to purge cache');
    } finally {
      setIsPurging(false);
    }
  };

  const handleToggleUnderAttack = async () => {
    const newState = !isUnderAttack;
    const message = newState
      ? 'Enable Under Attack mode? This will challenge all visitors.'
      : 'Disable Under Attack mode?';

    if (!confirm(message)) return;

    setIsTogglingAttack(true);
    try {
      await setUnderAttack(newState);
      toast.success(newState ? 'Under Attack mode enabled' : 'Under Attack mode disabled');
    } catch (error) {
      toast.error('Failed to toggle Under Attack mode');
    } finally {
      setIsTogglingAttack(false);
    }
  };

  const actions = [
    {
      name: 'Purge All Cache',
      description: 'Clear entire CDN cache',
      icon: <Trash2 className="w-5 h-5" />,
      onClick: handlePurgeAll,
      loading: isPurging,
      color: 'danger' as const,
    },
    {
      name: isUnderAttack ? 'Disable Under Attack' : 'Enable Under Attack',
      description: isUnderAttack ? 'Return to normal mode' : 'Challenge all visitors',
      icon: <AlertTriangle className="w-5 h-5" />,
      onClick: handleToggleUnderAttack,
      loading: isTogglingAttack,
      color: isUnderAttack ? 'success' as const : 'warning' as const,
    },
    {
      name: 'Development Mode',
      description: 'Bypass cache for testing',
      icon: <Code className="w-5 h-5" />,
      onClick: () => toast('Feature coming soon'),
      color: 'primary' as const,
    },
    {
      name: 'Sync DNS',
      description: 'Sync DNS records',
      icon: <RefreshCw className="w-5 h-5" />,
      onClick: () => toast('Feature coming soon'),
      color: 'accent' as const,
    },
  ];

  return (
    <div className="space-y-3">
      {actions.map((action) => (
        <button
          key={action.name}
          onClick={action.onClick}
          disabled={action.loading || isLoading}
          className="w-full flex items-center gap-3 p-3 rounded-lg border border-neutral-700/50 hover:bg-neutral-700/30 transition-all duration-200 disabled:opacity-50"
        >
          <div className={clsx('p-2 rounded-lg', colorClasses[action.color])}>
            {action.loading ? (
              <RefreshCw className="w-5 h-5 animate-spin" />
            ) : (
              action.icon
            )}
          </div>
          <div className="text-left">
            <p className="font-medium text-neutral-200">{action.name}</p>
            <p className="text-sm text-neutral-400">{action.description}</p>
          </div>
        </button>
      ))}
    </div>
  );
}
