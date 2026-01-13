import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useForm } from 'react-hook-form';
import {
  Shield,
  AlertTriangle,
  Lock,
  Ban,
  CheckCircle,
  XCircle,
  Plus,
  Trash2,
  RefreshCw,
  Globe,
  Bot,
  Zap,
} from 'lucide-react';
import { cloudflareApi } from '../lib/api';
import toast from 'react-hot-toast';
import { clsx } from 'clsx';

const securityLevels = [
  { value: 'off', label: 'Off', description: 'No security challenge' },
  { value: 'essentially_off', label: 'Essentially Off', description: 'Only challenges most threatening visitors' },
  { value: 'low', label: 'Low', description: 'Challenges only most threatening visitors' },
  { value: 'medium', label: 'Medium', description: 'Challenges more visitors', recommended: true },
  { value: 'high', label: 'High', description: 'Challenges all visitors except allowlisted' },
  { value: 'under_attack', label: 'Under Attack', description: 'Maximum protection, challenges all visitors' },
];

export function SecurityPage() {
  const queryClient = useQueryClient();
  const [showIpModal, setShowIpModal] = useState(false);
  const [showFirewallModal, setShowFirewallModal] = useState(false);

  const { data: securityLevel, isLoading: levelLoading } = useQuery({
    queryKey: ['security-level'],
    queryFn: () => cloudflareApi.getSecurityLevel(),
  });

  const { data: firewallRules } = useQuery({
    queryKey: ['firewall-rules'],
    queryFn: () => cloudflareApi.listFirewallRules(),
  });

  const { data: ipRules } = useQuery({
    queryKey: ['ip-access-rules'],
    queryFn: () => cloudflareApi.listIpAccessRules(),
  });

  const setLevelMutation = useMutation({
    mutationFn: (level: string) => cloudflareApi.setSecurityLevel(level),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['security-level'] });
      toast.success('Security level updated');
    },
    onError: () => toast.error('Failed to update security level'),
  });

  const toggleUnderAttackMutation = useMutation({
    mutationFn: (enabled: boolean) => cloudflareApi.toggleUnderAttack(enabled),
    onSuccess: (_, enabled) => {
      queryClient.invalidateQueries({ queryKey: ['security-level'] });
      toast.success(enabled ? 'Under Attack mode enabled' : 'Under Attack mode disabled');
    },
    onError: () => toast.error('Failed to toggle Under Attack mode'),
  });

  const blockIpMutation = useMutation({
    mutationFn: ({ ip, note }: { ip: string; note?: string }) => cloudflareApi.blockIp(ip, note),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['ip-access-rules'] });
      toast.success('IP blocked');
      setShowIpModal(false);
    },
    onError: () => toast.error('Failed to block IP'),
  });

  const createFirewallRuleMutation = useMutation({
    mutationFn: (rule: any) => cloudflareApi.createFirewallRule(rule),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['firewall-rules'] });
      toast.success('Firewall rule created');
      setShowFirewallModal(false);
    },
    onError: () => toast.error('Failed to create firewall rule'),
  });

  const currentLevel = securityLevel?.data?.data?.level || 'medium';
  const isUnderAttack = currentLevel === 'under_attack';

  return (
    <div className="p-6">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-neutral-100 flex items-center gap-3"><Shield className="w-7 h-7 text-orange-400" />Security & WAF</h1>
        <p className="text-neutral-400">Manage security settings and firewall rules</p>
      </div>

      {/* Under Attack Mode Banner */}
      {isUnderAttack && (
        <div className="mb-6 p-4 bg-red-500/10 border border-red-500/20 rounded-xl flex items-center justify-between">
          <div className="flex items-center gap-3">
            <AlertTriangle className="w-6 h-6 text-red-400" />
            <div>
              <p className="font-semibold text-red-300">Under Attack Mode Active</p>
              <p className="text-sm text-red-400">All visitors are being challenged</p>
            </div>
          </div>
          <button
            onClick={() => toggleUnderAttackMutation.mutate(false)}
            disabled={toggleUnderAttackMutation.isPending}
            className="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg"
          >
            Disable
          </button>
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Security Level */}
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 bg-blue-500/10 border border-blue-500/20 rounded-lg">
              <Shield className="w-5 h-5 text-blue-400" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-neutral-100">Security Level</h2>
              <p className="text-sm text-neutral-400">Challenge threshold for visitors</p>
            </div>
          </div>

          <div className="space-y-2">
            {securityLevels.map((level) => (
              <button
                key={level.value}
                onClick={() => setLevelMutation.mutate(level.value)}
                disabled={setLevelMutation.isPending}
                className={clsx(
                  'w-full flex items-center justify-between p-3 rounded-lg border transition-colors',
                  currentLevel === level.value
                    ? 'border-primary-500 bg-primary-500/10'
                    : 'border-neutral-700/50 hover:bg-neutral-700/30'
                )}
              >
                <div className="text-left">
                  <div className="flex items-center gap-2">
                    <span className="font-medium text-neutral-100">{level.label}</span>
                    {level.recommended && (
                      <span className="text-xs px-2 py-0.5 bg-green-500/10 text-green-400 border border-green-500/20 rounded-full">
                        Recommended
                      </span>
                    )}
                  </div>
                  <p className="text-sm text-neutral-400">{level.description}</p>
                </div>
                {currentLevel === level.value && (
                  <CheckCircle className="w-5 h-5 text-primary-500" />
                )}
              </button>
            ))}
          </div>
        </div>

        {/* Quick Actions */}
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <h2 className="text-lg font-semibold text-neutral-100 mb-4">Quick Actions</h2>

          <div className="space-y-3">
            <button
              onClick={() => toggleUnderAttackMutation.mutate(!isUnderAttack)}
              disabled={toggleUnderAttackMutation.isPending}
              className={clsx(
                'w-full flex items-center gap-3 p-4 rounded-lg border transition-colors',
                isUnderAttack
                  ? 'border-red-500 bg-red-500/10'
                  : 'border-neutral-700/50 hover:bg-neutral-700/30'
              )}
            >
              <AlertTriangle className={clsx('w-5 h-5', isUnderAttack ? 'text-red-400' : 'text-neutral-500')} />
              <div className="text-left">
                <p className="font-medium text-neutral-100">Under Attack Mode</p>
                <p className="text-sm text-neutral-400">
                  {isUnderAttack ? 'Currently active - maximum protection' : 'Activate for DDoS protection'}
                </p>
              </div>
            </button>

            <button
              onClick={() => setShowIpModal(true)}
              className="w-full flex items-center gap-3 p-4 rounded-lg border border-neutral-700/50 hover:bg-neutral-700/30"
            >
              <Ban className="w-5 h-5 text-neutral-500" />
              <div className="text-left">
                <p className="font-medium text-neutral-100">Block IP Address</p>
                <p className="text-sm text-neutral-400">Add an IP to the block list</p>
              </div>
            </button>

            <button
              onClick={() => setShowFirewallModal(true)}
              className="w-full flex items-center gap-3 p-4 rounded-lg border border-neutral-700/50 hover:bg-neutral-700/30"
            >
              <Zap className="w-5 h-5 text-neutral-500" />
              <div className="text-left">
                <p className="font-medium text-neutral-100">Create Firewall Rule</p>
                <p className="text-sm text-neutral-400">Add custom firewall logic</p>
              </div>
            </button>
          </div>
        </div>

        {/* IP Access Rules */}
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-neutral-100">IP Access Rules</h2>
            <button
              onClick={() => setShowIpModal(true)}
              className="flex items-center gap-1 text-sm text-primary-400 hover:text-primary-300"
            >
              <Plus className="w-4 h-4" />
              Add Rule
            </button>
          </div>

          <div className="space-y-2">
            {(ipRules?.data?.data || []).slice(0, 5).map((rule: any) => (
              <div
                key={rule.id}
                className="flex items-center justify-between p-3 bg-neutral-700/50 rounded-lg"
              >
                <div className="flex items-center gap-3">
                  {rule.mode === 'block' ? (
                    <XCircle className="w-5 h-5 text-red-400" />
                  ) : (
                    <CheckCircle className="w-5 h-5 text-green-400" />
                  )}
                  <div>
                    <p className="font-mono text-sm text-neutral-100">{rule.configuration?.value}</p>
                    <p className="text-xs text-neutral-400">{rule.notes || rule.mode}</p>
                  </div>
                </div>
              </div>
            ))}
            {(ipRules?.data?.data || []).length === 0 && (
              <p className="text-center text-neutral-400 py-4">No IP access rules</p>
            )}
          </div>
        </div>

        {/* Firewall Rules */}
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-neutral-100">Firewall Rules</h2>
            <button
              onClick={() => setShowFirewallModal(true)}
              className="flex items-center gap-1 text-sm text-primary-400 hover:text-primary-300"
            >
              <Plus className="w-4 h-4" />
              Add Rule
            </button>
          </div>

          <div className="space-y-2">
            {(firewallRules?.data?.data || []).slice(0, 5).map((rule: any) => (
              <div
                key={rule.id}
                className="flex items-center justify-between p-3 bg-neutral-700/50 rounded-lg"
              >
                <div>
                  <p className="font-medium text-sm text-neutral-100">{rule.description || 'Unnamed rule'}</p>
                  <p className="text-xs text-neutral-400">Action: {rule.action}</p>
                </div>
                <span className={clsx(
                  'px-2 py-1 text-xs rounded-full border',
                  rule.paused ? 'bg-neutral-500/10 text-neutral-400 border-neutral-500/20' : 'bg-green-500/10 text-green-400 border-green-500/20'
                )}>
                  {rule.paused ? 'Paused' : 'Active'}
                </span>
              </div>
            ))}
            {(firewallRules?.data?.data || []).length === 0 && (
              <p className="text-center text-neutral-400 py-4">No firewall rules</p>
            )}
          </div>
        </div>
      </div>

      {/* Block IP Modal */}
      {showIpModal && (
        <BlockIpModal
          onClose={() => setShowIpModal(false)}
          onBlock={(ip, note) => blockIpMutation.mutate({ ip, note })}
          isLoading={blockIpMutation.isPending}
        />
      )}

      {/* Firewall Rule Modal */}
      {showFirewallModal && (
        <FirewallRuleModal
          onClose={() => setShowFirewallModal(false)}
          onCreate={(rule) => createFirewallRuleMutation.mutate(rule)}
          isLoading={createFirewallRuleMutation.isPending}
        />
      )}
    </div>
  );
}

function BlockIpModal({ onClose, onBlock, isLoading }: {
  onClose: () => void;
  onBlock: (ip: string, note?: string) => void;
  isLoading: boolean;
}) {
  const { register, handleSubmit } = useForm();

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 shadow-2xl w-full max-w-md mx-4 p-6">
        <h2 className="text-lg font-semibold text-neutral-100 mb-4">Block IP Address</h2>
        <form onSubmit={handleSubmit((data) => onBlock(data.ip, data.note))} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-neutral-300 mb-1">IP Address</label>
            <input
              {...register('ip', { required: true })}
              placeholder="192.168.1.1 or 192.168.1.0/24"
              className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-900 text-neutral-100 placeholder-neutral-500"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-neutral-300 mb-1">Note (optional)</label>
            <input
              {...register('note')}
              placeholder="Reason for blocking"
              className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-900 text-neutral-100 placeholder-neutral-500"
            />
          </div>
          <div className="flex gap-3">
            <button type="button" onClick={onClose} className="flex-1 px-4 py-2 border border-neutral-600 rounded-lg text-neutral-300 hover:bg-neutral-700/50">
              Cancel
            </button>
            <button
              type="submit"
              disabled={isLoading}
              className="flex-1 px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg disabled:opacity-50"
            >
              {isLoading ? 'Blocking...' : 'Block IP'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

function FirewallRuleModal({ onClose, onCreate, isLoading }: {
  onClose: () => void;
  onCreate: (rule: any) => void;
  isLoading: boolean;
}) {
  const { register, handleSubmit } = useForm();

  const onSubmit = (data: any) => {
    onCreate({
      action: data.action,
      filter: { expression: data.expression },
      description: data.description,
    });
  };

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 shadow-2xl w-full max-w-md mx-4 p-6">
        <h2 className="text-lg font-semibold text-neutral-100 mb-4">Create Firewall Rule</h2>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-neutral-300 mb-1">Description</label>
            <input
              {...register('description')}
              placeholder="Rule description"
              className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-900 text-neutral-100 placeholder-neutral-500"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-neutral-300 mb-1">Expression</label>
            <textarea
              {...register('expression', { required: true })}
              placeholder='(ip.src eq 192.168.1.1) or (http.request.uri.path contains "/admin")'
              className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-900 text-neutral-100 placeholder-neutral-500 h-24"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-neutral-300 mb-1">Action</label>
            <select
              {...register('action')}
              className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-900 text-neutral-100"
            >
              <option value="block">Block</option>
              <option value="challenge">Challenge</option>
              <option value="js_challenge">JS Challenge</option>
              <option value="managed_challenge">Managed Challenge</option>
              <option value="allow">Allow</option>
              <option value="log">Log</option>
            </select>
          </div>
          <div className="flex gap-3">
            <button type="button" onClick={onClose} className="flex-1 px-4 py-2 border border-neutral-600 rounded-lg text-neutral-300 hover:bg-neutral-700/50">
              Cancel
            </button>
            <button
              type="submit"
              disabled={isLoading}
              className="flex-1 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg disabled:opacity-50"
            >
              {isLoading ? 'Creating...' : 'Create Rule'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
