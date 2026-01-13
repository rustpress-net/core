import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  FileText,
  Plus,
  Edit2,
  Trash2,
  RefreshCw,
  ArrowUp,
  ArrowDown,
  X,
  Globe,
  Shield,
  Zap,
  Database,
  Lock,
  AlertTriangle,
} from 'lucide-react';
import { cloudflareApi } from '../lib/api';
import toast from 'react-hot-toast';
import { clsx } from 'clsx';
import { Skeleton, ListSkeleton } from '../components/Skeleton';

interface PageRule {
  id: string;
  targets: Array<{ target: string; constraint: { operator: string; value: string } }>;
  actions: Array<{ id: string; value?: any }>;
  priority: number;
  status: 'active' | 'disabled';
}

const actionTypes = [
  { id: 'forwarding_url', label: 'Forwarding URL', icon: Globe },
  { id: 'always_use_https', label: 'Always Use HTTPS', icon: Lock },
  { id: 'cache_level', label: 'Cache Level', icon: Database },
  { id: 'browser_cache_ttl', label: 'Browser Cache TTL', icon: Zap },
  { id: 'edge_cache_ttl', label: 'Edge Cache TTL', icon: Zap },
  { id: 'security_level', label: 'Security Level', icon: Shield },
  { id: 'disable_security', label: 'Disable Security', icon: AlertTriangle },
  { id: 'ssl', label: 'SSL', icon: Lock },
  { id: 'minify', label: 'Minify', icon: Zap },
  { id: 'rocket_loader', label: 'Rocket Loader', icon: Zap },
];

const getActionLabel = (actionId: string): string => {
  const action = actionTypes.find((a) => a.id === actionId);
  return action?.label || actionId;
};

const formatActionValue = (action: { id: string; value?: any }): string => {
  if (action.id === 'forwarding_url') {
    return `${action.value?.status_code || 301} → ${action.value?.url || ''}`;
  }
  if (action.id === 'cache_level') {
    return action.value || 'standard';
  }
  if (typeof action.value === 'object') {
    return JSON.stringify(action.value);
  }
  return String(action.value ?? 'enabled');
};

export function RulesPage() {
  const queryClient = useQueryClient();
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [editingRule, setEditingRule] = useState<PageRule | null>(null);

  const { data: rules, isLoading } = useQuery({
    queryKey: ['page-rules'],
    queryFn: () => cloudflareApi.listPageRules(),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => cloudflareApi.deletePageRule(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['page-rules'] });
      toast.success('Page rule deleted');
    },
    onError: () => toast.error('Failed to delete rule'),
  });

  const rulesList: PageRule[] = rules?.data?.data || [];

  return (
    <div className="p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-neutral-100 flex items-center gap-3"><FileText className="w-7 h-7 text-orange-400" />Page Rules</h1>
          <p className="text-neutral-400">
            Configure URL-based behaviors and settings
          </p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg"
        >
          <Plus className="w-4 h-4" />
          Create Page Rule
        </button>
      </div>

      {/* Info Banner */}
      <div className="bg-info-500/10 border border-info-500/20 rounded-lg p-4 mb-6">
        <div className="flex items-start gap-3">
          <FileText className="w-5 h-5 text-info-400 mt-0.5" />
          <div>
            <p className="text-sm text-info-300">
              Page Rules trigger actions based on URL patterns. Rules are evaluated in priority order.
              You can have up to 3 free Page Rules per zone.
            </p>
          </div>
        </div>
      </div>

      {/* Rules List */}
      <div className="bg-neutral-800 rounded-xl border border-neutral-700/50">
        {isLoading ? (
          <div className="p-6">
            <ListSkeleton items={3} />
          </div>
        ) : rulesList.length === 0 ? (
          <div className="p-12 text-center text-neutral-400">
            <FileText className="w-16 h-16 mx-auto mb-4 opacity-50" />
            <p className="text-lg font-medium mb-2 text-neutral-200">No page rules configured</p>
            <p className="text-sm mb-4">Create your first page rule to customize URL-based behaviors</p>
            <button
              onClick={() => setShowCreateModal(true)}
              className="px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg"
            >
              Create Page Rule
            </button>
          </div>
        ) : (
          <div className="divide-y divide-neutral-700/50">
            {rulesList.map((rule, index) => (
              <div
                key={rule.id}
                className={clsx(
                  'p-4 hover:bg-neutral-700/30',
                  rule.status === 'disabled' && 'opacity-60'
                )}
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <span className="px-2 py-0.5 text-xs font-medium bg-neutral-700/50 text-neutral-300 rounded">
                        #{rule.priority}
                      </span>
                      <span
                        className={clsx(
                          'px-2 py-0.5 text-xs rounded-full',
                          rule.status === 'active'
                            ? 'bg-success-500/10 text-success-400 border border-success-500/20'
                            : 'bg-neutral-700/50 text-neutral-400'
                        )}
                      >
                        {rule.status}
                      </span>
                    </div>

                    <div className="flex items-center gap-2 mb-3">
                      <Globe className="w-4 h-4 text-neutral-500" />
                      <code className="text-sm font-mono bg-neutral-700/50 text-neutral-200 px-2 py-1 rounded">
                        {rule.targets[0]?.constraint?.value || '*'}
                      </code>
                    </div>

                    <div className="flex flex-wrap gap-2">
                      {rule.actions.map((action, idx) => (
                        <span
                          key={idx}
                          className="inline-flex items-center gap-1 px-2 py-1 text-xs bg-primary-500/10 text-primary-400 border border-primary-500/20 rounded"
                        >
                          {getActionLabel(action.id)}:
                          <span className="font-medium">{formatActionValue(action)}</span>
                        </span>
                      ))}
                    </div>
                  </div>

                  <div className="flex items-center gap-1 ml-4">
                    <button
                      disabled={index === 0}
                      className="p-1.5 text-neutral-500 hover:text-neutral-300 disabled:opacity-30"
                    >
                      <ArrowUp className="w-4 h-4" />
                    </button>
                    <button
                      disabled={index === rulesList.length - 1}
                      className="p-1.5 text-neutral-500 hover:text-neutral-300 disabled:opacity-30"
                    >
                      <ArrowDown className="w-4 h-4" />
                    </button>
                    <button
                      onClick={() => setEditingRule(rule)}
                      className="p-1.5 text-neutral-500 hover:text-info-400"
                    >
                      <Edit2 className="w-4 h-4" />
                    </button>
                    <button
                      onClick={() => {
                        if (confirm('Delete this page rule?')) {
                          deleteMutation.mutate(rule.id);
                        }
                      }}
                      className="p-1.5 text-neutral-500 hover:text-danger-400"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Common Rules Templates */}
      <div className="mt-8">
        <h3 className="text-lg font-semibold text-neutral-100 mb-4">
          Quick Templates
        </h3>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <TemplateCard
            title="Force HTTPS"
            description="Redirect all HTTP requests to HTTPS"
            pattern="http://*example.com/*"
            actions={[{ label: 'Always Use HTTPS' }]}
            onClick={() => {
              setShowCreateModal(true);
            }}
          />
          <TemplateCard
            title="301 Redirect"
            description="Permanent redirect to a new URL"
            pattern="example.com/old-page"
            actions={[{ label: 'Forwarding URL: 301' }]}
            onClick={() => setShowCreateModal(true)}
          />
          <TemplateCard
            title="Bypass Cache"
            description="Disable caching for specific URLs"
            pattern="example.com/api/*"
            actions={[{ label: 'Cache Level: Bypass' }]}
            onClick={() => setShowCreateModal(true)}
          />
          <TemplateCard
            title="Cache Everything"
            description="Cache all content including HTML"
            pattern="example.com/static/*"
            actions={[{ label: 'Cache Level: Cache Everything' }]}
            onClick={() => setShowCreateModal(true)}
          />
          <TemplateCard
            title="High Security"
            description="Enable strict security for admin pages"
            pattern="example.com/admin/*"
            actions={[{ label: 'Security Level: High' }]}
            onClick={() => setShowCreateModal(true)}
          />
          <TemplateCard
            title="Disable Apps"
            description="Disable Cloudflare apps for specific pages"
            pattern="example.com/embed/*"
            actions={[{ label: 'Disable Apps' }]}
            onClick={() => setShowCreateModal(true)}
          />
        </div>
      </div>

      {/* Create/Edit Modal */}
      {(showCreateModal || editingRule) && (
        <PageRuleModal
          rule={editingRule}
          onClose={() => {
            setShowCreateModal(false);
            setEditingRule(null);
          }}
          onSuccess={() => {
            queryClient.invalidateQueries({ queryKey: ['page-rules'] });
            setShowCreateModal(false);
            setEditingRule(null);
          }}
        />
      )}
    </div>
  );
}

function TemplateCard({
  title,
  description,
  pattern,
  actions,
  onClick,
}: {
  title: string;
  description: string;
  pattern: string;
  actions: Array<{ label: string }>;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className="p-4 border border-neutral-700/50 bg-neutral-800/50 rounded-lg hover:border-primary-500/50 text-left transition-colors"
    >
      <h4 className="font-medium text-neutral-100 mb-1">{title}</h4>
      <p className="text-sm text-neutral-400 mb-3">{description}</p>
      <div className="flex flex-wrap gap-2">
        <span className="text-xs font-mono bg-neutral-700/50 text-neutral-300 px-2 py-0.5 rounded">
          {pattern}
        </span>
        {actions.map((action, idx) => (
          <span
            key={idx}
            className="text-xs bg-primary-500/10 text-primary-400 px-2 py-0.5 rounded"
          >
            {action.label}
          </span>
        ))}
      </div>
    </button>
  );
}

function PageRuleModal({
  rule,
  onClose,
  onSuccess,
}: {
  rule: PageRule | null;
  onClose: () => void;
  onSuccess: () => void;
}) {
  const [urlPattern, setUrlPattern] = useState(rule?.targets[0]?.constraint?.value || '');
  const [selectedActions, setSelectedActions] = useState<Array<{ id: string; value?: any }>>(
    rule?.actions || []
  );
  const [status, setStatus] = useState<'active' | 'disabled'>(rule?.status || 'active');

  const createMutation = useMutation({
    mutationFn: (data: any) =>
      rule ? cloudflareApi.updatePageRule(rule.id, data) : cloudflareApi.createPageRule(data),
    onSuccess: () => {
      toast.success(rule ? 'Page rule updated' : 'Page rule created');
      onSuccess();
    },
    onError: () => toast.error('Failed to save page rule'),
  });

  const handleSubmit = () => {
    const data = {
      targets: [
        {
          target: 'url',
          constraint: { operator: 'matches', value: urlPattern },
        },
      ],
      actions: selectedActions,
      status,
    };
    createMutation.mutate(data);
  };

  const addAction = (actionId: string) => {
    if (!selectedActions.find((a) => a.id === actionId)) {
      setSelectedActions([...selectedActions, { id: actionId }]);
    }
  };

  const removeAction = (actionId: string) => {
    setSelectedActions(selectedActions.filter((a) => a.id !== actionId));
  };

  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
      <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 shadow-xl w-full max-w-2xl mx-4 max-h-[90vh] overflow-y-auto">
        <div className="flex items-center justify-between px-6 py-4 border-b border-neutral-700/50">
          <h2 className="text-lg font-semibold text-neutral-100">
            {rule ? 'Edit Page Rule' : 'Create Page Rule'}
          </h2>
          <button onClick={onClose} className="text-neutral-400 hover:text-neutral-200">
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="p-6 space-y-6">
          {/* URL Pattern */}
          <div>
            <label className="block text-sm font-medium text-neutral-300 mb-1">
              URL Pattern
            </label>
            <input
              type="text"
              value={urlPattern}
              onChange={(e) => setUrlPattern(e.target.value)}
              placeholder="*example.com/path/*"
              className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-700/50 text-neutral-100 placeholder-neutral-500 focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
            />
            <p className="text-xs text-neutral-500 mt-1">
              Use * as wildcard. Example: *example.com/admin/*
            </p>
          </div>

          {/* Actions */}
          <div>
            <label className="block text-sm font-medium text-neutral-300 mb-2">
              Actions
            </label>

            {/* Selected Actions */}
            <div className="space-y-2 mb-4">
              {selectedActions.map((action) => {
                const actionType = actionTypes.find((a) => a.id === action.id);
                return (
                  <div
                    key={action.id}
                    className="flex items-center justify-between p-3 bg-neutral-700/30 rounded-lg"
                  >
                    <div className="flex items-center gap-3">
                      {actionType && <actionType.icon className="w-4 h-4 text-neutral-400" />}
                      <span className="font-medium text-neutral-200">{getActionLabel(action.id)}</span>
                    </div>
                    <button
                      onClick={() => removeAction(action.id)}
                      className="p-1 text-neutral-400 hover:text-danger-400"
                    >
                      <X className="w-4 h-4" />
                    </button>
                  </div>
                );
              })}
            </div>

            {/* Add Action */}
            <select
              onChange={(e) => {
                if (e.target.value) {
                  addAction(e.target.value);
                  e.target.value = '';
                }
              }}
              className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-700/50 text-neutral-100 focus:border-primary-500"
            >
              <option value="">+ Add action...</option>
              {actionTypes
                .filter((a) => !selectedActions.find((s) => s.id === a.id))
                .map((action) => (
                  <option key={action.id} value={action.id}>
                    {action.label}
                  </option>
                ))}
            </select>
          </div>

          {/* Status */}
          <div className="flex items-center justify-between">
            <div>
              <p className="font-medium text-neutral-100">Rule Status</p>
              <p className="text-sm text-neutral-400">Enable or disable this rule</p>
            </div>
            <button
              onClick={() => setStatus(status === 'active' ? 'disabled' : 'active')}
              className={clsx(
                'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                status === 'active' ? 'bg-primary-600' : 'bg-neutral-600'
              )}
            >
              <span
                className={clsx(
                  'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
                  status === 'active' ? 'translate-x-6' : 'translate-x-1'
                )}
              />
            </button>
          </div>

          {/* Actions */}
          <div className="flex gap-3 pt-4 border-t border-neutral-700/50">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-4 py-2 border border-neutral-600 rounded-lg hover:bg-neutral-700/50 text-neutral-300"
            >
              Cancel
            </button>
            <button
              onClick={handleSubmit}
              disabled={!urlPattern || selectedActions.length === 0 || createMutation.isPending}
              className="flex-1 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg disabled:opacity-50 flex items-center justify-center gap-2"
            >
              {createMutation.isPending && <RefreshCw className="w-4 h-4 animate-spin" />}
              {rule ? 'Update Rule' : 'Create Rule'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
