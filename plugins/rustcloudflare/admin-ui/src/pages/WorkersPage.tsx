import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Code,
  Plus,
  Trash2,
  Globe,
  Database,
  RefreshCw,
  Upload,
  Settings,
  Eye,
  Copy,
  ChevronRight,
  Folder,
  File,
  X,
} from 'lucide-react';
import { cloudflareApi } from '../lib/api';
import toast from 'react-hot-toast';
import { clsx } from 'clsx';
import { Skeleton, ListSkeleton, TableRowSkeleton } from '../components/Skeleton';

type TabType = 'workers' | 'routes' | 'kv';

interface Worker {
  id: string;
  name: string;
  script?: string;
  created_on: string;
  modified_on: string;
}

interface WorkerRoute {
  id: string;
  pattern: string;
  script: string;
}

interface KVNamespace {
  id: string;
  title: string;
  supports_url_encoding?: boolean;
}

export function WorkersPage() {
  const queryClient = useQueryClient();
  const [activeTab, setActiveTab] = useState<TabType>('workers');
  const [showDeployModal, setShowDeployModal] = useState(false);
  const [showRouteModal, setShowRouteModal] = useState(false);
  const [showKVModal, setShowKVModal] = useState(false);
  const [selectedNamespace, setSelectedNamespace] = useState<string | null>(null);

  const { data: workers, isLoading: workersLoading } = useQuery({
    queryKey: ['workers'],
    queryFn: () => cloudflareApi.listWorkers(),
    enabled: activeTab === 'workers',
  });

  const { data: routes, isLoading: routesLoading } = useQuery({
    queryKey: ['worker-routes'],
    queryFn: () => cloudflareApi.listWorkerRoutes(),
    enabled: activeTab === 'routes',
  });

  const { data: namespaces, isLoading: kvLoading } = useQuery({
    queryKey: ['kv-namespaces'],
    queryFn: () => cloudflareApi.listKVNamespaces(),
    enabled: activeTab === 'kv',
  });

  const { data: kvKeys } = useQuery({
    queryKey: ['kv-keys', selectedNamespace],
    queryFn: () => cloudflareApi.listKVKeys(selectedNamespace!),
    enabled: !!selectedNamespace,
  });

  const deleteWorkerMutation = useMutation({
    mutationFn: (name: string) => cloudflareApi.deleteWorker(name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['workers'] });
      toast.success('Worker deleted');
    },
    onError: () => toast.error('Failed to delete worker'),
  });

  const deleteRouteMutation = useMutation({
    mutationFn: (id: string) => cloudflareApi.deleteWorkerRoute(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['worker-routes'] });
      toast.success('Route deleted');
    },
    onError: () => toast.error('Failed to delete route'),
  });

  const deleteNamespaceMutation = useMutation({
    mutationFn: (id: string) => cloudflareApi.deleteKVNamespace(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['kv-namespaces'] });
      toast.success('Namespace deleted');
    },
    onError: () => toast.error('Failed to delete namespace'),
  });

  const workersList: Worker[] = workers?.data?.data || [];
  const routesList: WorkerRoute[] = routes?.data?.data || [];
  const namespacesList: KVNamespace[] = namespaces?.data?.data || [];
  const keysList: string[] = kvKeys?.data?.data?.map((k: any) => k.name) || [];

  const tabs = [
    { id: 'workers' as const, label: 'Workers', icon: Code, count: workersList.length },
    { id: 'routes' as const, label: 'Routes', icon: Globe, count: routesList.length },
    { id: 'kv' as const, label: 'KV Storage', icon: Database, count: namespacesList.length },
  ];

  return (
    <div className="p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-neutral-100 flex items-center gap-3"><Code className="w-7 h-7 text-orange-400" />Edge Computing</h1>
          <p className="text-neutral-400">Manage Workers, Routes, and KV Storage</p>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-2 mb-6 border-b border-neutral-700/50">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={clsx(
              'flex items-center gap-2 px-4 py-3 border-b-2 transition-colors',
              activeTab === tab.id
                ? 'border-primary-500 text-primary-400'
                : 'border-transparent text-neutral-400 hover:text-neutral-200'
            )}
          >
            <tab.icon className="w-4 h-4" />
            {tab.label}
            {tab.count > 0 && (
              <span className="px-2 py-0.5 text-xs bg-neutral-700/50 rounded-full">
                {tab.count}
              </span>
            )}
          </button>
        ))}
      </div>

      {/* Workers Tab */}
      {activeTab === 'workers' && (
        <div>
          <div className="flex justify-end mb-4">
            <button
              onClick={() => setShowDeployModal(true)}
              className="flex items-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg"
            >
              <Upload className="w-4 h-4" />
              Deploy Worker
            </button>
          </div>

          {/* Worker Templates */}
          <div className="mb-6">
            <h3 className="text-sm font-medium text-neutral-300 mb-3">
              Quick Deploy Templates
            </h3>
            <div className="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-5 gap-3">
              {[
                { id: 'cache', name: 'Smart Cache', desc: 'Intelligent caching at the edge' },
                { id: 'security', name: 'Security Headers', desc: 'Add security headers to responses' },
                { id: 'image', name: 'Image Optimizer', desc: 'Optimize images on-the-fly' },
                { id: 'redirect', name: 'URL Redirect', desc: 'Handle URL redirections' },
                { id: 'analytics', name: 'Edge Analytics', desc: 'Collect analytics at the edge' },
              ].map((template) => (
                <button
                  key={template.id}
                  className="p-3 border border-neutral-700/50 bg-neutral-800/50 rounded-lg hover:border-primary-500/50 text-left transition-colors"
                >
                  <p className="font-medium text-sm text-neutral-200">{template.name}</p>
                  <p className="text-xs text-neutral-500 mt-1">{template.desc}</p>
                </button>
              ))}
            </div>
          </div>

          {/* Workers List */}
          <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 overflow-hidden">
            <table className="w-full">
              <thead className="bg-neutral-700/30">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-neutral-400 uppercase tracking-wider">
                    Worker
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-neutral-400 uppercase tracking-wider">
                    Created
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-neutral-400 uppercase tracking-wider">
                    Modified
                  </th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-neutral-400 uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-neutral-700/50">
                {workersLoading ? (
                  <>
                    <TableRowSkeleton columns={4} />
                    <TableRowSkeleton columns={4} />
                    <TableRowSkeleton columns={4} />
                  </>
                ) : workersList.length === 0 ? (
                  <tr>
                    <td colSpan={4} className="px-6 py-12 text-center text-neutral-400">
                      <Code className="w-12 h-12 mx-auto mb-3 opacity-50" />
                      <p className="text-neutral-200 font-medium">No workers deployed</p>
                      <p className="text-sm">Deploy your first worker to run code at the edge</p>
                    </td>
                  </tr>
                ) : (
                  workersList.map((worker) => (
                    <tr key={worker.id} className="hover:bg-neutral-700/30">
                      <td className="px-6 py-4">
                        <div className="flex items-center gap-3">
                          <div className="p-2 bg-primary-500/10 border border-primary-500/20 rounded-lg">
                            <Code className="w-4 h-4 text-primary-400" />
                          </div>
                          <span className="font-medium text-neutral-200">{worker.name}</span>
                        </div>
                      </td>
                      <td className="px-6 py-4 text-sm text-neutral-400">
                        {new Date(worker.created_on).toLocaleDateString()}
                      </td>
                      <td className="px-6 py-4 text-sm text-neutral-400">
                        {new Date(worker.modified_on).toLocaleDateString()}
                      </td>
                      <td className="px-6 py-4 text-right">
                        <button className="p-1 text-neutral-500 hover:text-info-400 mr-2">
                          <Eye className="w-4 h-4" />
                        </button>
                        <button className="p-1 text-neutral-500 hover:text-success-400 mr-2">
                          <Settings className="w-4 h-4" />
                        </button>
                        <button
                          onClick={() => {
                            if (confirm('Delete this worker?')) {
                              deleteWorkerMutation.mutate(worker.name);
                            }
                          }}
                          className="p-1 text-neutral-500 hover:text-danger-400"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Routes Tab */}
      {activeTab === 'routes' && (
        <div>
          <div className="flex justify-end mb-4">
            <button
              onClick={() => setShowRouteModal(true)}
              className="flex items-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg"
            >
              <Plus className="w-4 h-4" />
              Add Route
            </button>
          </div>

          <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 overflow-hidden">
            <table className="w-full">
              <thead className="bg-neutral-700/30">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-neutral-400 uppercase tracking-wider">
                    Pattern
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-neutral-400 uppercase tracking-wider">
                    Worker
                  </th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-neutral-400 uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-neutral-700/50">
                {routesLoading ? (
                  <>
                    <TableRowSkeleton columns={3} />
                    <TableRowSkeleton columns={3} />
                  </>
                ) : routesList.length === 0 ? (
                  <tr>
                    <td colSpan={3} className="px-6 py-12 text-center text-neutral-400">
                      <Globe className="w-12 h-12 mx-auto mb-3 opacity-50" />
                      <p className="text-neutral-200 font-medium">No routes configured</p>
                      <p className="text-sm">Create routes to trigger workers on specific URLs</p>
                    </td>
                  </tr>
                ) : (
                  routesList.map((route) => (
                    <tr key={route.id} className="hover:bg-neutral-700/30">
                      <td className="px-6 py-4 font-mono text-sm text-neutral-200">{route.pattern}</td>
                      <td className="px-6 py-4">
                        <span className="px-2 py-1 text-xs bg-primary-500/10 text-primary-400 rounded">
                          {route.script}
                        </span>
                      </td>
                      <td className="px-6 py-4 text-right">
                        <button
                          onClick={() => {
                            if (confirm('Delete this route?')) {
                              deleteRouteMutation.mutate(route.id);
                            }
                          }}
                          className="p-1 text-neutral-500 hover:text-danger-400"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* KV Tab */}
      {activeTab === 'kv' && (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Namespaces List */}
          <div className="lg:col-span-1">
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-medium text-neutral-100">Namespaces</h3>
              <button
                onClick={() => setShowKVModal(true)}
                className="p-2 text-primary-400 hover:bg-primary-500/10 rounded-lg"
              >
                <Plus className="w-4 h-4" />
              </button>
            </div>

            <div className="bg-neutral-800 rounded-xl border border-neutral-700/50">
              {kvLoading ? (
                <div className="p-6">
                  <ListSkeleton items={3} />
                </div>
              ) : namespacesList.length === 0 ? (
                <div className="p-8 text-center text-neutral-400">
                  <Database className="w-12 h-12 mx-auto mb-3 opacity-50" />
                  <p className="text-sm">No namespaces</p>
                </div>
              ) : (
                <div className="divide-y divide-neutral-700/50">
                  {namespacesList.map((ns) => (
                    <button
                      key={ns.id}
                      onClick={() => setSelectedNamespace(ns.id)}
                      className={clsx(
                        'w-full flex items-center justify-between p-4 hover:bg-neutral-700/30',
                        selectedNamespace === ns.id && 'bg-primary-500/10'
                      )}
                    >
                      <div className="flex items-center gap-3">
                        <Folder className="w-4 h-4 text-neutral-500" />
                        <span className="font-medium text-neutral-200">{ns.title}</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            if (confirm('Delete this namespace?')) {
                              deleteNamespaceMutation.mutate(ns.id);
                            }
                          }}
                          className="p-1 text-neutral-500 hover:text-danger-400"
                        >
                          <Trash2 className="w-3 h-3" />
                        </button>
                        <ChevronRight className="w-4 h-4 text-neutral-500" />
                      </div>
                    </button>
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Keys Browser */}
          <div className="lg:col-span-2">
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-medium text-neutral-100">
                {selectedNamespace
                  ? `Keys in ${namespacesList.find((ns) => ns.id === selectedNamespace)?.title}`
                  : 'Select a namespace'}
              </h3>
              {selectedNamespace && (
                <button className="flex items-center gap-2 px-3 py-1.5 text-sm border border-neutral-600 rounded-lg hover:bg-neutral-700/50 text-neutral-300">
                  <Plus className="w-4 h-4" />
                  Add Key
                </button>
              )}
            </div>

            <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 min-h-[400px]">
              {!selectedNamespace ? (
                <div className="flex items-center justify-center h-[400px] text-neutral-500">
                  <div className="text-center">
                    <Database className="w-12 h-12 mx-auto mb-3 opacity-50" />
                    <p>Select a namespace to view keys</p>
                  </div>
                </div>
              ) : keysList.length === 0 ? (
                <div className="flex items-center justify-center h-[400px] text-neutral-500">
                  <div className="text-center">
                    <File className="w-12 h-12 mx-auto mb-3 opacity-50" />
                    <p>No keys in this namespace</p>
                  </div>
                </div>
              ) : (
                <div className="divide-y divide-neutral-700/50">
                  {keysList.map((key) => (
                    <div
                      key={key}
                      className="flex items-center justify-between p-4 hover:bg-neutral-700/30"
                    >
                      <div className="flex items-center gap-3">
                        <File className="w-4 h-4 text-neutral-500" />
                        <span className="font-mono text-sm text-neutral-200">{key}</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <button className="p-1 text-neutral-500 hover:text-info-400">
                          <Eye className="w-4 h-4" />
                        </button>
                        <button className="p-1 text-neutral-500 hover:text-neutral-300">
                          <Copy className="w-4 h-4" />
                        </button>
                        <button className="p-1 text-neutral-500 hover:text-danger-400">
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Deploy Worker Modal */}
      {showDeployModal && (
        <DeployWorkerModal
          onClose={() => setShowDeployModal(false)}
          onSuccess={() => {
            queryClient.invalidateQueries({ queryKey: ['workers'] });
            setShowDeployModal(false);
          }}
        />
      )}

      {/* Add Route Modal */}
      {showRouteModal && (
        <AddRouteModal
          workers={workersList}
          onClose={() => setShowRouteModal(false)}
          onSuccess={() => {
            queryClient.invalidateQueries({ queryKey: ['worker-routes'] });
            setShowRouteModal(false);
          }}
        />
      )}

      {/* Create KV Namespace Modal */}
      {showKVModal && (
        <CreateKVModal
          onClose={() => setShowKVModal(false)}
          onSuccess={() => {
            queryClient.invalidateQueries({ queryKey: ['kv-namespaces'] });
            setShowKVModal(false);
          }}
        />
      )}
    </div>
  );
}

function DeployWorkerModal({
  onClose,
  onSuccess,
}: {
  onClose: () => void;
  onSuccess: () => void;
}) {
  const [name, setName] = useState('');
  const [script, setScript] = useState('');

  const deployMutation = useMutation({
    mutationFn: () => cloudflareApi.deployWorker(name, script),
    onSuccess: () => {
      toast.success('Worker deployed');
      onSuccess();
    },
    onError: () => toast.error('Failed to deploy worker'),
  });

  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
      <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 shadow-xl w-full max-w-2xl mx-4">
        <div className="flex items-center justify-between px-6 py-4 border-b border-neutral-700/50">
          <h2 className="text-lg font-semibold text-neutral-100">Deploy Worker</h2>
          <button onClick={onClose} className="text-neutral-400 hover:text-neutral-200">
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="p-6 space-y-4">
          <div>
            <label className="block text-sm font-medium text-neutral-300 mb-1">
              Worker Name
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="my-worker"
              className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-700/50 text-neutral-100 placeholder-neutral-500"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-neutral-300 mb-1">
              Worker Script
            </label>
            <textarea
              value={script}
              onChange={(e) => setScript(e.target.value)}
              placeholder="addEventListener('fetch', event => { ... })"
              rows={12}
              className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-700/50 text-neutral-100 font-mono text-sm placeholder-neutral-500"
            />
          </div>

          <div className="flex gap-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-4 py-2 border border-neutral-600 rounded-lg hover:bg-neutral-700/50 text-neutral-300"
            >
              Cancel
            </button>
            <button
              onClick={() => deployMutation.mutate()}
              disabled={!name || !script || deployMutation.isPending}
              className="flex-1 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg disabled:opacity-50 flex items-center justify-center gap-2"
            >
              {deployMutation.isPending && <RefreshCw className="w-4 h-4 animate-spin" />}
              Deploy
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function AddRouteModal({
  workers,
  onClose,
  onSuccess,
}: {
  workers: Worker[];
  onClose: () => void;
  onSuccess: () => void;
}) {
  const [pattern, setPattern] = useState('');
  const [script, setScript] = useState('');

  const createMutation = useMutation({
    mutationFn: () => cloudflareApi.createWorkerRoute(pattern, script),
    onSuccess: () => {
      toast.success('Route created');
      onSuccess();
    },
    onError: () => toast.error('Failed to create route'),
  });

  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
      <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 shadow-xl w-full max-w-md mx-4">
        <div className="flex items-center justify-between px-6 py-4 border-b border-neutral-700/50">
          <h2 className="text-lg font-semibold text-neutral-100">Add Route</h2>
          <button onClick={onClose} className="text-neutral-400 hover:text-neutral-200">
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="p-6 space-y-4">
          <div>
            <label className="block text-sm font-medium text-neutral-300 mb-1">
              Route Pattern
            </label>
            <input
              type="text"
              value={pattern}
              onChange={(e) => setPattern(e.target.value)}
              placeholder="example.com/*"
              className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-700/50 text-neutral-100 placeholder-neutral-500"
            />
            <p className="text-xs text-neutral-500 mt-1">Use * as wildcard</p>
          </div>

          <div>
            <label className="block text-sm font-medium text-neutral-300 mb-1">
              Worker
            </label>
            <select
              value={script}
              onChange={(e) => setScript(e.target.value)}
              className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-700/50 text-neutral-100"
            >
              <option value="">Select a worker</option>
              {workers.map((worker) => (
                <option key={worker.id} value={worker.name}>
                  {worker.name}
                </option>
              ))}
            </select>
          </div>

          <div className="flex gap-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-4 py-2 border border-neutral-600 rounded-lg hover:bg-neutral-700/50 text-neutral-300"
            >
              Cancel
            </button>
            <button
              onClick={() => createMutation.mutate()}
              disabled={!pattern || !script || createMutation.isPending}
              className="flex-1 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg disabled:opacity-50 flex items-center justify-center gap-2"
            >
              {createMutation.isPending && <RefreshCw className="w-4 h-4 animate-spin" />}
              Create
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function CreateKVModal({
  onClose,
  onSuccess,
}: {
  onClose: () => void;
  onSuccess: () => void;
}) {
  const [title, setTitle] = useState('');

  const createMutation = useMutation({
    mutationFn: () => cloudflareApi.createKVNamespace(title),
    onSuccess: () => {
      toast.success('Namespace created');
      onSuccess();
    },
    onError: () => toast.error('Failed to create namespace'),
  });

  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
      <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 shadow-xl w-full max-w-md mx-4">
        <div className="flex items-center justify-between px-6 py-4 border-b border-neutral-700/50">
          <h2 className="text-lg font-semibold text-neutral-100">Create KV Namespace</h2>
          <button onClick={onClose} className="text-neutral-400 hover:text-neutral-200">
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="p-6 space-y-4">
          <div>
            <label className="block text-sm font-medium text-neutral-300 mb-1">
              Namespace Name
            </label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="MY_KV_STORE"
              className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-700/50 text-neutral-100 placeholder-neutral-500"
            />
          </div>

          <div className="flex gap-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-4 py-2 border border-neutral-600 rounded-lg hover:bg-neutral-700/50 text-neutral-300"
            >
              Cancel
            </button>
            <button
              onClick={() => createMutation.mutate()}
              disabled={!title || createMutation.isPending}
              className="flex-1 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg disabled:opacity-50 flex items-center justify-center gap-2"
            >
              {createMutation.isPending && <RefreshCw className="w-4 h-4 animate-spin" />}
              Create
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
