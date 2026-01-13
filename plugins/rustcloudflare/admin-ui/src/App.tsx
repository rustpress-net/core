import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Toaster } from 'react-hot-toast';
import { useEffect } from 'react';
import { CloudflareLayout } from './layouts/CloudflareLayout';
import { ConnectionRequired } from './components/ConnectionRequired';
import { useCloudflareStore } from './stores/cloudflareStore';

// Pages
import { DashboardPage } from './pages/DashboardPage';
import { CachePage } from './pages/CachePage';
import { DnsPage } from './pages/DnsPage';
import { SecurityPage } from './pages/SecurityPage';
import { SslPage } from './pages/SslPage';
import { PerformancePage } from './pages/PerformancePage';
import { WorkersPage } from './pages/WorkersPage';
import { R2Page } from './pages/R2Page';
import { StreamPage } from './pages/StreamPage';
import { AnalyticsPage } from './pages/AnalyticsPage';
import { RulesPage } from './pages/RulesPage';
import { D1Page } from './pages/D1Page';
import { SettingsPage } from './pages/SettingsPage';

// Wrapper component for pages that require Cloudflare connection
function Protected({ children }: { children: React.ReactNode }) {
  return <ConnectionRequired>{children}</ConnectionRequired>;
}

// Initialize connection check on app load
function AppInitializer({ children }: { children: React.ReactNode }) {
  const { checkConnection, fetchZone } = useCloudflareStore();

  useEffect(() => {
    checkConnection();
    fetchZone();
  }, [checkConnection, fetchZone]);

  return <>{children}</>;
}

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // 5 minutes
      retry: 1,
    },
  },
});

// Use basename only in production (when embedded in RustPress)
const basename = import.meta.env.DEV ? '/' : '/admin/cloudflare';

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter basename={basename} future={{ v7_startTransition: true, v7_relativeSplatPath: true }}>
        <AppInitializer>
          <Routes>
            <Route element={<CloudflareLayout />}>
              <Route index element={<Navigate to="/dashboard" replace />} />
              <Route path="/dashboard" element={<Protected><DashboardPage /></Protected>} />
              <Route path="/cache" element={<Protected><CachePage /></Protected>} />
              <Route path="/dns" element={<Protected><DnsPage /></Protected>} />
              <Route path="/security" element={<Protected><SecurityPage /></Protected>} />
              <Route path="/ssl" element={<Protected><SslPage /></Protected>} />
              <Route path="/performance" element={<Protected><PerformancePage /></Protected>} />
              <Route path="/workers" element={<Protected><WorkersPage /></Protected>} />
              <Route path="/r2" element={<Protected><R2Page /></Protected>} />
              <Route path="/stream" element={<Protected><StreamPage /></Protected>} />
              <Route path="/d1" element={<Protected><D1Page /></Protected>} />
              <Route path="/analytics" element={<Protected><AnalyticsPage /></Protected>} />
              <Route path="/rules" element={<Protected><RulesPage /></Protected>} />
              {/* Settings page doesn't require connection - users need to configure credentials here */}
              <Route path="/settings" element={<SettingsPage />} />
            </Route>
          </Routes>
        </AppInitializer>
      </BrowserRouter>
      <Toaster position="top-right" />
    </QueryClientProvider>
  );
}
