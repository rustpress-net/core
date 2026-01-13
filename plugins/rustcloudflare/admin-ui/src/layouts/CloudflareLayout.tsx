import { Outlet, NavLink } from 'react-router-dom';
import {
  Cloud,
  LayoutDashboard,
  Database,
  Globe,
  Shield,
  Lock,
  Zap,
  Code,
  HardDrive,
  Video,
  BarChart3,
  FileCode,
  Settings,
  AlertTriangle,
  Cylinder,
} from 'lucide-react';
import { useCloudflareStore } from '../stores/cloudflareStore';
import { clsx } from 'clsx';

const navigation = [
  { name: 'Dashboard', href: '/dashboard', icon: LayoutDashboard },
  { name: 'Cache', href: '/cache', icon: Database },
  { name: 'DNS', href: '/dns', icon: Globe },
  { name: 'Security', href: '/security', icon: Shield },
  { name: 'SSL/TLS', href: '/ssl', icon: Lock },
  { name: 'Performance', href: '/performance', icon: Zap },
  { name: 'Workers', href: '/workers', icon: Code },
  { name: 'R2 Storage', href: '/r2', icon: HardDrive },
  { name: 'D1 Database', href: '/d1', icon: Cylinder },
  { name: 'Stream', href: '/stream', icon: Video },
  { name: 'Analytics', href: '/analytics', icon: BarChart3 },
  { name: 'Page Rules', href: '/rules', icon: FileCode },
  { name: 'Settings', href: '/settings', icon: Settings },
];

export function CloudflareLayout() {
  const { isConnected, isUnderAttack, zone } = useCloudflareStore();

  return (
    <div className="flex h-screen bg-neutral-900">
      {/* Sidebar */}
      <aside className="w-64 bg-neutral-800 border-r border-neutral-700/50">
        <div className="flex items-center gap-3 px-6 py-4 border-b border-neutral-700/50">
          <div className="p-2 rounded-lg bg-gradient-to-br from-cloudflare-500 to-cloudflare-600">
            <Cloud className="w-6 h-6 text-white" />
          </div>
          <div>
            <h1 className="text-lg font-bold text-neutral-100">Cloudflare</h1>
            <p className="text-xs text-neutral-400">{zone?.name || 'Not connected'}</p>
          </div>
        </div>

        {/* Connection Status */}
        <div className="px-4 py-3 border-b border-neutral-700/50">
          <div className="flex items-center gap-2">
            <span
              className={clsx(
                'w-2 h-2 rounded-full',
                isConnected ? 'bg-success-500' : 'bg-danger-500'
              )}
            />
            <span className="text-sm text-neutral-400">
              {isConnected ? 'Connected' : 'Disconnected'}
            </span>
          </div>
          {isUnderAttack && (
            <div className="mt-2 flex items-center gap-2 text-warning-400">
              <AlertTriangle className="w-4 h-4" />
              <span className="text-sm font-medium">Under Attack Mode</span>
            </div>
          )}
        </div>

        {/* Navigation */}
        <nav className="flex-1 px-2 py-4 space-y-1 overflow-y-auto hide-scrollbar">
          {navigation.map((item) => (
            <NavLink
              key={item.name}
              to={item.href}
              className={({ isActive }) =>
                clsx(
                  'flex items-center gap-3 px-4 py-2.5 text-sm font-medium rounded-lg transition-all duration-200',
                  isActive
                    ? 'bg-primary-500/10 text-primary-400 border-l-2 border-primary-500'
                    : 'text-neutral-400 hover:bg-neutral-700/50 hover:text-neutral-200'
                )
              }
            >
              <item.icon className="w-5 h-5" />
              {item.name}
            </NavLink>
          ))}
        </nav>

        {/* Plan Info */}
        <div className="px-4 py-3 border-t border-neutral-700/50">
          <div className="flex items-center justify-between">
            <div className="text-xs text-neutral-500">
              Plan: <span className="font-medium text-neutral-300">{zone?.plan?.name || 'Free'}</span>
            </div>
            <div className="px-2 py-0.5 text-xs font-medium rounded bg-primary-500/10 text-primary-400">
              Active
            </div>
          </div>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-y-auto bg-neutral-900">
        <Outlet />
      </main>
    </div>
  );
}
