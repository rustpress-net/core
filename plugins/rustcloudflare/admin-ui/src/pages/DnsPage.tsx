import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useForm } from 'react-hook-form';
import {
  Globe,
  Plus,
  Edit2,
  Trash2,
  Shield,
  Cloud,
  RefreshCw,
  Download,
  Upload,
  Search,
  Check,
  X,
} from 'lucide-react';
import { cloudflareApi } from '../lib/api';
import toast from 'react-hot-toast';
import { clsx } from 'clsx';

interface DnsRecord {
  id: string;
  type: string;
  name: string;
  content: string;
  proxied: boolean;
  ttl: number;
  priority?: number;
}

const recordTypes = ['A', 'AAAA', 'CNAME', 'MX', 'TXT', 'SRV', 'CAA', 'NS'];

export function DnsPage() {
  const queryClient = useQueryClient();
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [editingRecord, setEditingRecord] = useState<DnsRecord | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterType, setFilterType] = useState<string>('all');

  const { data: records, isLoading } = useQuery({
    queryKey: ['dns-records'],
    queryFn: () => cloudflareApi.listDnsRecords(),
  });

  const createMutation = useMutation({
    mutationFn: (record: any) => cloudflareApi.createDnsRecord(record),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['dns-records'] });
      toast.success('DNS record created');
      setIsAddModalOpen(false);
    },
    onError: () => toast.error('Failed to create record'),
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, record }: { id: string; record: any }) =>
      cloudflareApi.updateDnsRecord(id, record),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['dns-records'] });
      toast.success('DNS record updated');
      setEditingRecord(null);
    },
    onError: () => toast.error('Failed to update record'),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => cloudflareApi.deleteDnsRecord(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['dns-records'] });
      toast.success('DNS record deleted');
    },
    onError: () => toast.error('Failed to delete record'),
  });

  const handleExportZone = async () => {
    try {
      const response = await cloudflareApi.exportZone();
      const blob = new Blob([response.data], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'zone.txt';
      a.click();
      toast.success('Zone file exported');
    } catch {
      toast.error('Failed to export zone file');
    }
  };

  const dnsRecords: DnsRecord[] = records?.data?.data || [];
  const filteredRecords = dnsRecords.filter((record) => {
    const matchesSearch =
      record.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      record.content.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesType = filterType === 'all' || record.type === filterType;
    return matchesSearch && matchesType;
  });

  return (
    <div className="p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-neutral-100 flex items-center gap-3"><Globe className="w-7 h-7 text-orange-400" />DNS Management</h1>
          <p className="text-neutral-400">Manage your domain's DNS records</p>
        </div>
        <div className="flex gap-3">
          <button
            onClick={handleExportZone}
            className="flex items-center gap-2 px-4 py-2 border border-neutral-600 rounded-lg hover:bg-neutral-700/50 text-neutral-300"
          >
            <Download className="w-4 h-4" />
            Export Zone
          </button>
          <button
            onClick={() => setIsAddModalOpen(true)}
            className="flex items-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg"
          >
            <Plus className="w-4 h-4" />
            Add Record
          </button>
        </div>
      </div>

      {/* Filters */}
      <div className="flex gap-4 mb-6">
        <div className="flex-1 relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
          <input
            type="text"
            placeholder="Search records..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-full pl-10 pr-4 py-2 border border-neutral-600 rounded-lg bg-neutral-900 text-neutral-100 placeholder-neutral-500"
          />
        </div>
        <select
          value={filterType}
          onChange={(e) => setFilterType(e.target.value)}
          className="px-4 py-2 border border-neutral-600 rounded-lg bg-neutral-900 text-neutral-100"
        >
          <option value="all">All Types</option>
          {recordTypes.map((type) => (
            <option key={type} value={type}>{type}</option>
          ))}
        </select>
      </div>

      {/* Records Table */}
      <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 overflow-hidden">
        <table className="w-full">
          <thead className="bg-neutral-700/50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-neutral-400 uppercase tracking-wider">
                Type
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-neutral-400 uppercase tracking-wider">
                Name
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-neutral-400 uppercase tracking-wider">
                Content
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-neutral-400 uppercase tracking-wider">
                Proxy
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-neutral-400 uppercase tracking-wider">
                TTL
              </th>
              <th className="px-6 py-3 text-right text-xs font-medium text-neutral-400 uppercase tracking-wider">
                Actions
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-neutral-700/50">
            {isLoading ? (
              <tr>
                <td colSpan={6} className="px-6 py-8 text-center text-neutral-400">
                  <RefreshCw className="w-6 h-6 animate-spin mx-auto mb-2" />
                  Loading records...
                </td>
              </tr>
            ) : filteredRecords.length === 0 ? (
              <tr>
                <td colSpan={6} className="px-6 py-8 text-center text-neutral-400">
                  No DNS records found
                </td>
              </tr>
            ) : (
              filteredRecords.map((record) => (
                <tr key={record.id} className="hover:bg-neutral-700/30">
                  <td className="px-6 py-4">
                    <span className={clsx(
                      'inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border',
                      record.type === 'A' && 'bg-blue-500/10 text-blue-400 border-blue-500/20',
                      record.type === 'AAAA' && 'bg-purple-500/10 text-purple-400 border-purple-500/20',
                      record.type === 'CNAME' && 'bg-green-500/10 text-green-400 border-green-500/20',
                      record.type === 'MX' && 'bg-yellow-500/10 text-yellow-400 border-yellow-500/20',
                      record.type === 'TXT' && 'bg-neutral-500/10 text-neutral-400 border-neutral-500/20',
                      !['A', 'AAAA', 'CNAME', 'MX', 'TXT'].includes(record.type) && 'bg-primary-500/10 text-primary-400 border-primary-500/20'
                    )}>
                      {record.type}
                    </span>
                  </td>
                  <td className="px-6 py-4 font-mono text-sm text-neutral-100">{record.name}</td>
                  <td className="px-6 py-4 font-mono text-sm max-w-xs truncate text-neutral-300">{record.content}</td>
                  <td className="px-6 py-4">
                    {record.proxied ? (
                      <Cloud className="w-5 h-5 text-primary-500" />
                    ) : (
                      <Globe className="w-5 h-5 text-neutral-500" />
                    )}
                  </td>
                  <td className="px-6 py-4 text-sm text-neutral-400">
                    {record.ttl === 1 ? 'Auto' : `${record.ttl}s`}
                  </td>
                  <td className="px-6 py-4 text-right">
                    <button
                      onClick={() => setEditingRecord(record)}
                      className="p-1 text-neutral-400 hover:text-primary-400 mr-2"
                    >
                      <Edit2 className="w-4 h-4" />
                    </button>
                    <button
                      onClick={() => {
                        if (confirm('Delete this record?')) {
                          deleteMutation.mutate(record.id);
                        }
                      }}
                      className="p-1 text-neutral-400 hover:text-red-400"
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

      {/* Add/Edit Modal */}
      {(isAddModalOpen || editingRecord) && (
        <DnsRecordModal
          record={editingRecord}
          onClose={() => {
            setIsAddModalOpen(false);
            setEditingRecord(null);
          }}
          onSave={(data) => {
            if (editingRecord) {
              updateMutation.mutate({ id: editingRecord.id, record: data });
            } else {
              createMutation.mutate(data);
            }
          }}
          isLoading={createMutation.isPending || updateMutation.isPending}
        />
      )}
    </div>
  );
}

interface DnsRecordModalProps {
  record: DnsRecord | null;
  onClose: () => void;
  onSave: (data: any) => void;
  isLoading: boolean;
}

function DnsRecordModal({ record, onClose, onSave, isLoading }: DnsRecordModalProps) {
  const { register, handleSubmit, watch } = useForm({
    defaultValues: record || {
      type: 'A',
      name: '',
      content: '',
      ttl: 1,
      proxied: true,
      priority: 10,
    },
  });

  const recordType = watch('type');
  const showPriority = recordType === 'MX' || recordType === 'SRV';
  const canProxy = ['A', 'AAAA', 'CNAME'].includes(recordType);

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 shadow-2xl w-full max-w-md mx-4">
        <div className="flex items-center justify-between px-6 py-4 border-b border-neutral-700/50">
          <h2 className="text-lg font-semibold text-neutral-100">
            {record ? 'Edit DNS Record' : 'Add DNS Record'}
          </h2>
          <button onClick={onClose} className="text-neutral-400 hover:text-neutral-200">
            <X className="w-5 h-5" />
          </button>
        </div>

        <form onSubmit={handleSubmit(onSave)} className="p-6 space-y-4">
          <div>
            <label className="block text-sm font-medium text-neutral-300 mb-1">
              Type
            </label>
            <select
              {...register('type')}
              disabled={!!record}
              className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-900 text-neutral-100 disabled:opacity-50"
            >
              {recordTypes.map((type) => (
                <option key={type} value={type}>{type}</option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-neutral-300 mb-1">
              Name
            </label>
            <input
              {...register('name')}
              placeholder="@ for root, or subdomain"
              className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-900 text-neutral-100 placeholder-neutral-500"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-neutral-300 mb-1">
              Content
            </label>
            <input
              {...register('content')}
              placeholder={recordType === 'A' ? 'IPv4 address' : recordType === 'AAAA' ? 'IPv6 address' : 'Value'}
              className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-900 text-neutral-100 placeholder-neutral-500"
            />
          </div>

          {showPriority && (
            <div>
              <label className="block text-sm font-medium text-neutral-300 mb-1">
                Priority
              </label>
              <input
                {...register('priority', { valueAsNumber: true })}
                type="number"
                className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-900 text-neutral-100"
              />
            </div>
          )}

          <div>
            <label className="block text-sm font-medium text-neutral-300 mb-1">
              TTL
            </label>
            <select
              {...register('ttl', { valueAsNumber: true })}
              className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-900 text-neutral-100"
            >
              <option value={1}>Auto</option>
              <option value={60}>1 minute</option>
              <option value={300}>5 minutes</option>
              <option value={3600}>1 hour</option>
              <option value={86400}>1 day</option>
            </select>
          </div>

          {canProxy && (
            <div className="flex items-center gap-2">
              <input
                {...register('proxied')}
                type="checkbox"
                id="proxied"
                className="w-4 h-4 rounded border-neutral-600 bg-neutral-900 text-primary-600"
              />
              <label htmlFor="proxied" className="text-sm text-neutral-300">
                Proxy through Cloudflare (orange cloud)
              </label>
            </div>
          )}

          <div className="flex gap-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-4 py-2 border border-neutral-600 rounded-lg hover:bg-neutral-700/50 text-neutral-300"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isLoading}
              className="flex-1 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg disabled:opacity-50 flex items-center justify-center gap-2"
            >
              {isLoading && <RefreshCw className="w-4 h-4 animate-spin" />}
              {record ? 'Update' : 'Create'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
