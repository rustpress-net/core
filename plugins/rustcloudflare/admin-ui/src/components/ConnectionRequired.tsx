import { useCloudflareStore } from '../stores/cloudflareStore';
import { Settings, AlertTriangle, ArrowRight, Bug } from 'lucide-react';
import { Link } from 'react-router-dom';
import { useState } from 'react';

interface ConnectionRequiredProps {
  children: React.ReactNode;
}

// In development mode, allow bypassing the connection requirement for testing
const isDevelopment = import.meta.env.DEV;

export function ConnectionRequired({ children }: ConnectionRequiredProps) {
  const { isConnected, isLoading } = useCloudflareStore();
  const [devBypass, setDevBypass] = useState(false);

  // In development mode with bypass enabled, skip the connection check
  if (isDevelopment && devBypass) {
    return <>{children}</>;
  }

  // Show loading state while checking connection
  if (isLoading) {
    return (
      <>
        <div className="blur-sm pointer-events-none select-none">
          {children}
        </div>
        <div className="fixed inset-0 bg-neutral-900/60 backdrop-blur-sm flex items-center justify-center z-50">
          <div className="bg-neutral-800 rounded-2xl border border-neutral-700/50 p-8 max-w-md mx-4 text-center shadow-2xl">
            <div className="w-16 h-16 mx-auto mb-6 rounded-full bg-primary-500/10 border border-primary-500/20 flex items-center justify-center">
              <div className="w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full animate-spin" />
            </div>
            <h2 className="text-xl font-bold text-neutral-100 mb-2">
              Checking Connection
            </h2>
            <p className="text-neutral-400">
              Verifying your Cloudflare credentials...
            </p>
          </div>
        </div>
      </>
    );
  }

  // Show setup overlay when not connected
  if (!isConnected) {
    return (
      <>
        <div className="blur-sm pointer-events-none select-none opacity-40">
          {children}
        </div>
        <div className="fixed inset-0 bg-neutral-900/70 backdrop-blur-md flex items-center justify-center z-50">
          <div className="bg-neutral-800 rounded-2xl border border-neutral-700/50 p-8 max-w-md mx-4 text-center shadow-2xl">
            <div className="w-20 h-20 mx-auto mb-6 rounded-full bg-warning-500/10 border border-warning-500/20 flex items-center justify-center">
              <AlertTriangle className="w-10 h-10 text-warning-400" />
            </div>
            <h2 className="text-2xl font-bold text-neutral-100 mb-3">
              Set Up Cloudflare for RustPress
            </h2>
            <p className="text-neutral-400 mb-6">
              Connect your Cloudflare account to enable CDN management, security features, DNS controls, and more for your RustPress site.
            </p>
            <div className="space-y-3">
              <Link
                to="/settings"
                className="w-full flex items-center justify-center gap-2 px-6 py-3 bg-primary-600 hover:bg-primary-700 text-white rounded-lg font-medium transition-colors"
              >
                <Settings className="w-5 h-5" />
                Configure Settings
                <ArrowRight className="w-4 h-4" />
              </Link>
              <a
                href="https://dash.cloudflare.com/profile/api-tokens"
                target="_blank"
                rel="noopener noreferrer"
                className="w-full flex items-center justify-center gap-2 px-6 py-3 border border-neutral-600 hover:border-neutral-500 text-neutral-300 hover:text-neutral-100 rounded-lg font-medium transition-colors"
              >
                Get API Token
              </a>
            </div>
            <div className="mt-6 pt-6 border-t border-neutral-700/50">
              <p className="text-sm text-neutral-500">
                You'll need your API Token, Account ID, and Zone ID from the Cloudflare dashboard.
              </p>
            </div>

            {/* Development mode bypass button */}
            {isDevelopment && (
              <div className="mt-4 pt-4 border-t border-neutral-700/50">
                <button
                  onClick={() => setDevBypass(true)}
                  className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-neutral-700/50 hover:bg-neutral-700 text-neutral-400 hover:text-neutral-200 rounded-lg text-sm transition-colors"
                >
                  <Bug className="w-4 h-4" />
                  Skip for Testing (Dev Only)
                </button>
              </div>
            )}
          </div>
        </div>
      </>
    );
  }

  // Connected - render children normally
  return <>{children}</>;
}
