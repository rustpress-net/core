import { useState, useEffect, useCallback } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Database, Plus, Play, Table2, Code, Loader2, Trash2, RefreshCw,
  Download, Copy, ChevronRight, Clock, Zap, HardDrive, FileText,
  Terminal, Eye, X, AlertCircle, CheckCircle2, History
} from 'lucide-react';
import { cloudflareApi } from '../lib/api';
import toast from 'react-hot-toast';
import { ListSkeleton } from '../components/Skeleton';
import { useCloudflareStore } from '../stores/cloudflareStore';

interface D1Database {
  uuid: string;
  name: string;
  version: string;
  num_tables?: number;
  file_size?: number;
  created_at?: string;
}

interface TableSchema {
  name: string;
  type: string;
  notnull: boolean;
  dflt_value: string | null;
  pk: boolean;
}

interface QueryResult {
  results: any[];
  success: boolean;
  meta?: {
    duration?: number;
    changes?: number;
    rows_read?: number;
    rows_written?: number;
  };
}

interface QueryHistoryItem {
  sql: string;
  timestamp: Date;
  duration?: number;
  rowCount?: number;
  success: boolean;
}

const QUICK_QUERIES = [
  { label: 'Show Tables', sql: "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;" },
  { label: 'Table Info', sql: "PRAGMA table_info(table_name);" },
  { label: 'Count Rows', sql: "SELECT COUNT(*) as count FROM table_name;" },
  { label: 'Recent Data', sql: "SELECT * FROM table_name ORDER BY rowid DESC LIMIT 10;" },
];

export function D1Page() {
  const { isConnected } = useCloudflareStore();
  const queryClient = useQueryClient();
  const [selectedDb, setSelectedDb] = useState<D1Database | null>(null);
  const [sqlQuery, setSqlQuery] = useState("SELECT name, type FROM sqlite_master WHERE type='table' ORDER BY name;");
  const [queryResults, setQueryResults] = useState<QueryResult | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState(false);
  const [showSchemaModal, setShowSchemaModal] = useState<string | null>(null);
  const [newDbName, setNewDbName] = useState('');
  const [queryHistory, setQueryHistory] = useState<QueryHistoryItem[]>([]);
  const [activeTab, setActiveTab] = useState<'editor' | 'history'>('editor');
  const [tableSchema, setTableSchema] = useState<TableSchema[]>([]);

  const { data: databases, isLoading, refetch: refetchDatabases } = useQuery({
    queryKey: ['d1-databases'],
    queryFn: async () => {
      const response = await cloudflareApi.listD1Databases();
      const data = response.data?.data ?? response.data;
      return (data?.databases ?? data ?? []) as D1Database[];
    },
    enabled: isConnected,
  });

  const { data: tables, isLoading: tablesLoading, refetch: refetchTables } = useQuery({
    queryKey: ['d1-tables', selectedDb?.uuid],
    queryFn: async () => {
      if (!selectedDb) return [];
      const response = await cloudflareApi.listD1Tables(selectedDb.uuid);
      const data = response.data?.data ?? response.data;
      return (data?.tables ?? data ?? []) as string[];
    },
    enabled: !!selectedDb && isConnected,
  });

  const createDbMutation = useMutation({
    mutationFn: async (name: string) => {
      const response = await cloudflareApi.createD1Database(name);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['d1-databases'] });
      toast.success('Database created successfully');
      setShowCreateModal(false);
      setNewDbName('');
    },
    onError: () => {
      toast.error('Failed to create database');
    },
  });

  const deleteDbMutation = useMutation({
    mutationFn: async (databaseId: string) => {
      const response = await cloudflareApi.deleteD1Database(databaseId);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['d1-databases'] });
      toast.success('Database deleted successfully');
      setShowDeleteModal(false);
      setSelectedDb(null);
    },
    onError: () => {
      toast.error('Failed to delete database');
    },
  });

  const executeQueryMutation = useMutation({
    mutationFn: async ({ databaseId, sql }: { databaseId: string; sql: string }) => {
      const response = await cloudflareApi.executeD1Query(databaseId, sql);
      const data = response.data?.data ?? response.data;
      return data as QueryResult;
    },
    onSuccess: (data, variables) => {
      setQueryResults(data);
      const historyItem: QueryHistoryItem = {
        sql: variables.sql,
        timestamp: new Date(),
        duration: data?.meta?.duration,
        rowCount: data?.results?.length,
        success: true,
      };
      setQueryHistory(prev => [historyItem, ...prev.slice(0, 49)]);
      toast.success(`Query executed in ${data?.meta?.duration?.toFixed(2) || 0}ms`);
    },
    onError: (_, variables) => {
      const historyItem: QueryHistoryItem = {
        sql: variables.sql,
        timestamp: new Date(),
        success: false,
      };
      setQueryHistory(prev => [historyItem, ...prev.slice(0, 49)]);
      toast.error('Query execution failed');
    },
  });

  const handleExecuteQuery = useCallback(() => {
    if (!selectedDb || !sqlQuery.trim()) return;
    executeQueryMutation.mutate({ databaseId: selectedDb.uuid, sql: sqlQuery });
  }, [selectedDb, sqlQuery, executeQueryMutation]);

  // Keyboard shortcut for execute
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
        e.preventDefault();
        handleExecuteQuery();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleExecuteQuery]);

  const formatBytes = (bytes?: number) => {
    if (!bytes) return '0 B';
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(2)} ${sizes[i]}`;
  };

  const formatDate = (dateStr?: string) => {
    if (!dateStr) return 'N/A';
    return new Date(dateStr).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    toast.success('Copied to clipboard');
  };

  const exportResults = () => {
    if (!queryResults?.results.length) return;
    const csv = [
      Object.keys(queryResults.results[0]).join(','),
      ...queryResults.results.map(row =>
        Object.values(row).map(v =>
          typeof v === 'string' ? `"${v.replace(/"/g, '""')}"` : v
        ).join(',')
      )
    ].join('\n');

    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `query-results-${Date.now()}.csv`;
    a.click();
    URL.revokeObjectURL(url);
    toast.success('Results exported as CSV');
  };

  const viewTableSchema = async (tableName: string) => {
    if (!selectedDb) return;
    setShowSchemaModal(tableName);
    try {
      const response = await cloudflareApi.executeD1Query(
        selectedDb.uuid,
        `PRAGMA table_info(${tableName});`
      );
      const data = response.data?.data ?? response.data;
      if (data?.results) {
        setTableSchema(data.results.map((r: any) => ({
          name: r.name,
          type: r.type,
          notnull: r.notnull === 1,
          dflt_value: r.dflt_value,
          pk: r.pk === 1,
        })));
      }
    } catch {
      toast.error('Failed to fetch table schema');
    }
  };

  // Stats calculations
  const totalTables = databases?.reduce((acc, db) => acc + (db.num_tables || 0), 0) || 0;
  const totalSize = databases?.reduce((acc, db) => acc + (db.file_size || 0), 0) || 0;

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white flex items-center gap-3">
            <Database className="w-7 h-7 text-orange-400" />
            D1 Databases
          </h1>
          <p className="text-neutral-400 mt-1">Serverless SQL databases at the edge</p>
        </div>
        <div className="flex items-center gap-3">
          <button
            onClick={() => refetchDatabases()}
            className="flex items-center gap-2 px-3 py-2 bg-neutral-700 hover:bg-neutral-600 text-white rounded-lg transition-colors"
          >
            <RefreshCw className="w-4 h-4" />
          </button>
          <button
            onClick={() => setShowCreateModal(true)}
            className="flex items-center gap-2 px-4 py-2 bg-orange-500 hover:bg-orange-600 text-white rounded-lg transition-colors font-medium"
          >
            <Plus className="w-4 h-4" />
            Create Database
          </button>
        </div>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-gradient-to-br from-orange-500/20 to-orange-600/10 rounded-xl border border-orange-500/30 p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-orange-500/20 rounded-lg">
              <Database className="w-5 h-5 text-orange-400" />
            </div>
            <div>
              <p className="text-neutral-400 text-sm">Databases</p>
              <p className="text-2xl font-bold text-white">{databases?.length || 0}</p>
            </div>
          </div>
        </div>
        <div className="bg-gradient-to-br from-blue-500/20 to-blue-600/10 rounded-xl border border-blue-500/30 p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-blue-500/20 rounded-lg">
              <Table2 className="w-5 h-5 text-blue-400" />
            </div>
            <div>
              <p className="text-neutral-400 text-sm">Total Tables</p>
              <p className="text-2xl font-bold text-white">{totalTables}</p>
            </div>
          </div>
        </div>
        <div className="bg-gradient-to-br from-purple-500/20 to-purple-600/10 rounded-xl border border-purple-500/30 p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-purple-500/20 rounded-lg">
              <HardDrive className="w-5 h-5 text-purple-400" />
            </div>
            <div>
              <p className="text-neutral-400 text-sm">Total Size</p>
              <p className="text-2xl font-bold text-white">{formatBytes(totalSize)}</p>
            </div>
          </div>
        </div>
        <div className="bg-gradient-to-br from-green-500/20 to-green-600/10 rounded-xl border border-green-500/30 p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-green-500/20 rounded-lg">
              <Zap className="w-5 h-5 text-green-400" />
            </div>
            <div>
              <p className="text-neutral-400 text-sm">Queries Run</p>
              <p className="text-2xl font-bold text-white">{queryHistory.length}</p>
            </div>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Database List - Sidebar */}
        <div className="lg:col-span-3">
          <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 overflow-hidden">
            <div className="p-4 border-b border-neutral-700/50">
              <h2 className="text-lg font-semibold text-white flex items-center gap-2">
                <Database className="w-5 h-5 text-orange-400" />
                Databases
              </h2>
            </div>

            <div className="p-2 max-h-[500px] overflow-y-auto">
              {isLoading ? (
                <div className="p-2"><ListSkeleton items={3} /></div>
              ) : databases && databases.length > 0 ? (
                <div className="space-y-1">
                  {databases.map((db) => (
                    <button
                      key={db.uuid}
                      onClick={() => setSelectedDb(db)}
                      className={`w-full text-left p-3 rounded-lg transition-all group ${
                        selectedDb?.uuid === db.uuid
                          ? 'bg-orange-500/20 border-l-4 border-orange-500'
                          : 'hover:bg-neutral-700/50 border-l-4 border-transparent'
                      }`}
                    >
                      <div className="flex items-center justify-between">
                        <div className="font-medium text-white truncate">{db.name}</div>
                        <ChevronRight className={`w-4 h-4 text-neutral-500 transition-transform ${
                          selectedDb?.uuid === db.uuid ? 'rotate-90 text-orange-400' : ''
                        }`} />
                      </div>
                      <div className="flex items-center gap-3 text-xs text-neutral-400 mt-1">
                        <span className="flex items-center gap-1">
                          <Table2 className="w-3 h-3" />
                          {db.num_tables || 0}
                        </span>
                        <span className="flex items-center gap-1">
                          <HardDrive className="w-3 h-3" />
                          {formatBytes(db.file_size)}
                        </span>
                      </div>
                    </button>
                  ))}
                </div>
              ) : (
                <div className="text-center py-8 text-neutral-400">
                  <Database className="w-12 h-12 mx-auto mb-3 opacity-50" />
                  <p className="text-sm">No databases yet</p>
                  <button
                    onClick={() => setShowCreateModal(true)}
                    className="text-orange-400 hover:text-orange-300 text-sm mt-2"
                  >
                    Create your first database
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Main Content Area */}
        <div className="lg:col-span-9 space-y-4">
          {selectedDb ? (
            <>
              {/* Database Header */}
              <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-4">
                    <div className="p-3 bg-orange-500/20 rounded-xl">
                      <Database className="w-6 h-6 text-orange-400" />
                    </div>
                    <div>
                      <h2 className="text-xl font-bold text-white">{selectedDb.name}</h2>
                      <div className="flex items-center gap-2 mt-1">
                        <code className="text-xs text-neutral-400 bg-neutral-700/50 px-2 py-0.5 rounded">
                          {selectedDb.uuid}
                        </code>
                        <button
                          onClick={() => copyToClipboard(selectedDb.uuid)}
                          className="text-neutral-500 hover:text-white"
                        >
                          <Copy className="w-3 h-3" />
                        </button>
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center gap-3">
                    <div className="flex items-center gap-6 text-sm mr-4">
                      <div className="text-center">
                        <p className="text-2xl font-bold text-white">{selectedDb.num_tables || 0}</p>
                        <p className="text-neutral-400 text-xs">Tables</p>
                      </div>
                      <div className="text-center">
                        <p className="text-2xl font-bold text-white">{formatBytes(selectedDb.file_size)}</p>
                        <p className="text-neutral-400 text-xs">Size</p>
                      </div>
                    </div>
                    <button
                      onClick={() => refetchTables()}
                      className="p-2 bg-neutral-700 hover:bg-neutral-600 rounded-lg transition-colors"
                      title="Refresh tables"
                    >
                      <RefreshCw className="w-4 h-4 text-white" />
                    </button>
                    <button
                      onClick={() => setShowDeleteModal(true)}
                      className="p-2 bg-red-500/20 hover:bg-red-500/30 rounded-lg transition-colors"
                      title="Delete database"
                    >
                      <Trash2 className="w-4 h-4 text-red-400" />
                    </button>
                  </div>
                </div>
              </div>

              {/* Tables Grid */}
              <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-4">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="text-sm font-medium text-white flex items-center gap-2">
                    <Table2 className="w-4 h-4 text-blue-400" />
                    Tables
                    {tables && <span className="text-neutral-500">({tables.length})</span>}
                  </h3>
                </div>

                {tablesLoading ? (
                  <div className="flex items-center justify-center py-8">
                    <Loader2 className="w-6 h-6 animate-spin text-neutral-400" />
                  </div>
                ) : tables && tables.length > 0 ? (
                  <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-2">
                    {tables.map((table) => (
                      <div
                        key={table}
                        className="group bg-neutral-700/30 hover:bg-neutral-700/50 rounded-lg p-3 transition-all border border-transparent hover:border-neutral-600"
                      >
                        <div className="flex items-center justify-between">
                          <div className="flex items-center gap-2 min-w-0">
                            <FileText className="w-4 h-4 text-blue-400 flex-shrink-0" />
                            <span className="text-white text-sm font-medium truncate">{table}</span>
                          </div>
                          <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                            <button
                              onClick={() => viewTableSchema(table)}
                              className="p-1 hover:bg-neutral-600 rounded"
                              title="View schema"
                            >
                              <Eye className="w-3.5 h-3.5 text-neutral-400" />
                            </button>
                            <button
                              onClick={() => setSqlQuery(`SELECT * FROM ${table} LIMIT 100;`)}
                              className="p-1 hover:bg-neutral-600 rounded"
                              title="Query table"
                            >
                              <Terminal className="w-3.5 h-3.5 text-neutral-400" />
                            </button>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <div className="text-center py-6 text-neutral-400">
                    <Table2 className="w-8 h-8 mx-auto mb-2 opacity-50" />
                    <p className="text-sm">No tables in this database</p>
                  </div>
                )}
              </div>

              {/* Query Section */}
              <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 overflow-hidden">
                {/* Tabs */}
                <div className="flex border-b border-neutral-700">
                  <button
                    onClick={() => setActiveTab('editor')}
                    className={`flex items-center gap-2 px-4 py-3 text-sm font-medium transition-colors ${
                      activeTab === 'editor'
                        ? 'text-orange-400 border-b-2 border-orange-400 bg-neutral-700/30'
                        : 'text-neutral-400 hover:text-white'
                    }`}
                  >
                    <Code className="w-4 h-4" />
                    SQL Editor
                  </button>
                  <button
                    onClick={() => setActiveTab('history')}
                    className={`flex items-center gap-2 px-4 py-3 text-sm font-medium transition-colors ${
                      activeTab === 'history'
                        ? 'text-orange-400 border-b-2 border-orange-400 bg-neutral-700/30'
                        : 'text-neutral-400 hover:text-white'
                    }`}
                  >
                    <History className="w-4 h-4" />
                    History
                    {queryHistory.length > 0 && (
                      <span className="px-1.5 py-0.5 bg-neutral-600 rounded text-xs">{queryHistory.length}</span>
                    )}
                  </button>
                </div>

                {activeTab === 'editor' ? (
                  <div className="p-4">
                    {/* Quick Queries */}
                    <div className="flex flex-wrap gap-2 mb-3">
                      {QUICK_QUERIES.map((q) => (
                        <button
                          key={q.label}
                          onClick={() => setSqlQuery(q.sql)}
                          className="px-2 py-1 text-xs bg-neutral-700/50 hover:bg-neutral-700 text-neutral-300 rounded transition-colors"
                        >
                          {q.label}
                        </button>
                      ))}
                    </div>

                    {/* SQL Editor */}
                    <div className="relative">
                      <textarea
                        value={sqlQuery}
                        onChange={(e) => setSqlQuery(e.target.value)}
                        placeholder="Enter your SQL query..."
                        className="w-full h-40 bg-neutral-900 border border-neutral-700 rounded-lg p-4 font-mono text-sm text-white placeholder-neutral-500 focus:outline-none focus:border-orange-500 resize-none"
                        spellCheck={false}
                      />
                      <div className="absolute bottom-3 right-3 flex items-center gap-2">
                        <span className="text-xs text-neutral-500">Ctrl+Enter to run</span>
                      </div>
                    </div>

                    {/* Execute Button */}
                    <div className="flex items-center justify-between mt-3">
                      <div className="text-xs text-neutral-500">
                        {sqlQuery.length} characters
                      </div>
                      <button
                        onClick={handleExecuteQuery}
                        disabled={executeQueryMutation.isPending || !sqlQuery.trim()}
                        className="flex items-center gap-2 px-5 py-2.5 bg-green-500 hover:bg-green-600 disabled:bg-neutral-600 disabled:cursor-not-allowed text-white rounded-lg transition-colors font-medium"
                      >
                        {executeQueryMutation.isPending ? (
                          <Loader2 className="w-4 h-4 animate-spin" />
                        ) : (
                          <Play className="w-4 h-4" />
                        )}
                        Execute Query
                      </button>
                    </div>
                  </div>
                ) : (
                  <div className="p-4 max-h-80 overflow-y-auto">
                    {queryHistory.length > 0 ? (
                      <div className="space-y-2">
                        {queryHistory.map((item, i) => (
                          <div
                            key={i}
                            className="bg-neutral-700/30 rounded-lg p-3 hover:bg-neutral-700/50 cursor-pointer group"
                            onClick={() => {
                              setSqlQuery(item.sql);
                              setActiveTab('editor');
                            }}
                          >
                            <div className="flex items-center justify-between mb-2">
                              <div className="flex items-center gap-2">
                                {item.success ? (
                                  <CheckCircle2 className="w-4 h-4 text-green-400" />
                                ) : (
                                  <AlertCircle className="w-4 h-4 text-red-400" />
                                )}
                                <span className="text-xs text-neutral-400">
                                  {item.timestamp.toLocaleTimeString()}
                                </span>
                              </div>
                              <div className="flex items-center gap-3 text-xs text-neutral-500">
                                {item.duration && <span>{item.duration.toFixed(2)}ms</span>}
                                {item.rowCount !== undefined && <span>{item.rowCount} rows</span>}
                              </div>
                            </div>
                            <code className="text-sm text-neutral-300 font-mono line-clamp-2">
                              {item.sql}
                            </code>
                          </div>
                        ))}
                      </div>
                    ) : (
                      <div className="text-center py-8 text-neutral-400">
                        <Clock className="w-8 h-8 mx-auto mb-2 opacity-50" />
                        <p className="text-sm">No query history yet</p>
                      </div>
                    )}
                  </div>
                )}
              </div>

              {/* Query Results */}
              {queryResults && (
                <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 overflow-hidden">
                  <div className="flex items-center justify-between p-4 border-b border-neutral-700/50">
                    <div className="flex items-center gap-4">
                      <h3 className="text-sm font-medium text-white">
                        Results
                      </h3>
                      <span className="px-2 py-0.5 bg-green-500/20 text-green-400 rounded text-xs font-medium">
                        {queryResults.results?.length || 0} rows
                      </span>
                    </div>
                    <div className="flex items-center gap-4">
                      {queryResults.meta && (
                        <div className="flex items-center gap-4 text-xs text-neutral-400">
                          <span className="flex items-center gap-1">
                            <Clock className="w-3 h-3" />
                            {queryResults.meta.duration?.toFixed(2)}ms
                          </span>
                          <span>{queryResults.meta.rows_read || 0} read</span>
                          <span>{queryResults.meta.rows_written || 0} written</span>
                        </div>
                      )}
                      {queryResults.results?.length > 0 && (
                        <button
                          onClick={exportResults}
                          className="flex items-center gap-1 px-3 py-1.5 bg-neutral-700 hover:bg-neutral-600 text-white rounded text-xs transition-colors"
                        >
                          <Download className="w-3 h-3" />
                          Export CSV
                        </button>
                      )}
                    </div>
                  </div>

                  {queryResults.results?.length > 0 ? (
                    <div className="overflow-x-auto max-h-96">
                      <table className="w-full text-sm">
                        <thead className="bg-neutral-700/50 sticky top-0">
                          <tr>
                            <th className="text-left py-2 px-3 text-neutral-400 font-medium text-xs w-10">#</th>
                            {Object.keys(queryResults.results[0]).map((key) => (
                              <th
                                key={key}
                                className="text-left py-2 px-3 text-neutral-400 font-medium text-xs whitespace-nowrap"
                              >
                                {key}
                              </th>
                            ))}
                          </tr>
                        </thead>
                        <tbody className="divide-y divide-neutral-700/50">
                          {queryResults.results.slice(0, 100).map((row, i) => (
                            <tr key={i} className="hover:bg-neutral-700/30">
                              <td className="py-2 px-3 text-neutral-500 text-xs">{i + 1}</td>
                              {Object.values(row).map((value: any, j) => (
                                <td key={j} className="py-2 px-3 text-neutral-300 font-mono text-xs max-w-xs truncate">
                                  {value === null ? (
                                    <span className="text-neutral-500 italic">NULL</span>
                                  ) : typeof value === 'object' ? (
                                    <span className="text-blue-400">{JSON.stringify(value)}</span>
                                  ) : typeof value === 'boolean' ? (
                                    <span className={value ? 'text-green-400' : 'text-red-400'}>
                                      {String(value)}
                                    </span>
                                  ) : (
                                    String(value)
                                  )}
                                </td>
                              ))}
                            </tr>
                          ))}
                        </tbody>
                      </table>
                      {queryResults.results.length > 100 && (
                        <div className="p-3 text-center text-sm text-neutral-400 bg-neutral-700/30">
                          Showing first 100 of {queryResults.results.length} results
                        </div>
                      )}
                    </div>
                  ) : (
                    <div className="text-center py-8 text-neutral-400">
                      <CheckCircle2 className="w-8 h-8 mx-auto mb-2 text-green-400" />
                      <p className="text-sm">Query executed successfully</p>
                      <p className="text-xs mt-1">No results returned</p>
                    </div>
                  )}
                </div>
              )}
            </>
          ) : (
            <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-16 text-center">
              <Database className="w-20 h-20 mx-auto mb-6 text-neutral-600" />
              <h3 className="text-xl font-medium text-neutral-300 mb-2">
                Select a Database
              </h3>
              <p className="text-neutral-400 max-w-md mx-auto">
                Choose a database from the sidebar to run queries, view tables, and manage your data
              </p>
            </div>
          )}
        </div>
      </div>

      {/* Create Database Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50">
          <div className="bg-neutral-800 rounded-2xl border border-neutral-700 p-6 w-full max-w-md shadow-2xl">
            <div className="flex items-center gap-3 mb-6">
              <div className="p-2 bg-orange-500/20 rounded-lg">
                <Plus className="w-5 h-5 text-orange-400" />
              </div>
              <h2 className="text-lg font-semibold text-white">Create New Database</h2>
            </div>
            <div className="space-y-4">
              <div>
                <label className="block text-sm text-neutral-400 mb-2">Database Name</label>
                <input
                  type="text"
                  value={newDbName}
                  onChange={(e) => setNewDbName(e.target.value)}
                  placeholder="my-database"
                  className="w-full bg-neutral-900 border border-neutral-700 rounded-lg px-4 py-3 text-white placeholder-neutral-500 focus:outline-none focus:border-orange-500 font-mono"
                  autoFocus
                />
                <p className="text-xs text-neutral-500 mt-2">
                  Use lowercase letters, numbers, and hyphens only
                </p>
              </div>
            </div>
            <div className="flex justify-end gap-3 mt-6">
              <button
                onClick={() => {
                  setShowCreateModal(false);
                  setNewDbName('');
                }}
                className="px-4 py-2 text-neutral-400 hover:text-white transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={() => createDbMutation.mutate(newDbName)}
                disabled={!newDbName.trim() || createDbMutation.isPending}
                className="flex items-center gap-2 px-5 py-2 bg-orange-500 hover:bg-orange-600 disabled:bg-neutral-600 text-white rounded-lg transition-colors font-medium"
              >
                {createDbMutation.isPending && <Loader2 className="w-4 h-4 animate-spin" />}
                Create Database
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Delete Database Modal */}
      {showDeleteModal && selectedDb && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50">
          <div className="bg-neutral-800 rounded-2xl border border-neutral-700 p-6 w-full max-w-md shadow-2xl">
            <div className="flex items-center gap-3 mb-4">
              <div className="p-2 bg-red-500/20 rounded-lg">
                <Trash2 className="w-5 h-5 text-red-400" />
              </div>
              <h2 className="text-lg font-semibold text-white">Delete Database</h2>
            </div>
            <p className="text-neutral-300 mb-2">
              Are you sure you want to delete <strong className="text-white">{selectedDb.name}</strong>?
            </p>
            <p className="text-sm text-red-400 mb-6">
              This action cannot be undone. All tables and data will be permanently deleted.
            </p>
            <div className="flex justify-end gap-3">
              <button
                onClick={() => setShowDeleteModal(false)}
                className="px-4 py-2 text-neutral-400 hover:text-white transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={() => deleteDbMutation.mutate(selectedDb.uuid)}
                disabled={deleteDbMutation.isPending}
                className="flex items-center gap-2 px-5 py-2 bg-red-500 hover:bg-red-600 disabled:bg-neutral-600 text-white rounded-lg transition-colors font-medium"
              >
                {deleteDbMutation.isPending && <Loader2 className="w-4 h-4 animate-spin" />}
                Delete Database
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Table Schema Modal */}
      {showSchemaModal && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50">
          <div className="bg-neutral-800 rounded-2xl border border-neutral-700 p-6 w-full max-w-2xl shadow-2xl">
            <div className="flex items-center justify-between mb-6">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-blue-500/20 rounded-lg">
                  <Table2 className="w-5 h-5 text-blue-400" />
                </div>
                <div>
                  <h2 className="text-lg font-semibold text-white">Table Schema</h2>
                  <p className="text-sm text-neutral-400">{showSchemaModal}</p>
                </div>
              </div>
              <button
                onClick={() => {
                  setShowSchemaModal(null);
                  setTableSchema([]);
                }}
                className="p-2 hover:bg-neutral-700 rounded-lg transition-colors"
              >
                <X className="w-5 h-5 text-neutral-400" />
              </button>
            </div>

            {tableSchema.length > 0 ? (
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead className="bg-neutral-700/50">
                    <tr>
                      <th className="text-left py-2 px-3 text-neutral-400 font-medium">Column</th>
                      <th className="text-left py-2 px-3 text-neutral-400 font-medium">Type</th>
                      <th className="text-left py-2 px-3 text-neutral-400 font-medium">Not Null</th>
                      <th className="text-left py-2 px-3 text-neutral-400 font-medium">Default</th>
                      <th className="text-left py-2 px-3 text-neutral-400 font-medium">PK</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-neutral-700/50">
                    {tableSchema.map((col, i) => (
                      <tr key={i} className="hover:bg-neutral-700/30">
                        <td className="py-2 px-3 text-white font-mono">{col.name}</td>
                        <td className="py-2 px-3 text-blue-400 font-mono">{col.type}</td>
                        <td className="py-2 px-3">
                          {col.notnull ? (
                            <span className="text-green-400">Yes</span>
                          ) : (
                            <span className="text-neutral-500">No</span>
                          )}
                        </td>
                        <td className="py-2 px-3 text-neutral-400 font-mono">
                          {col.dflt_value || <span className="text-neutral-500">-</span>}
                        </td>
                        <td className="py-2 px-3">
                          {col.pk ? (
                            <span className="px-2 py-0.5 bg-yellow-500/20 text-yellow-400 rounded text-xs">PK</span>
                          ) : (
                            <span className="text-neutral-500">-</span>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : (
              <div className="flex items-center justify-center py-8">
                <Loader2 className="w-6 h-6 animate-spin text-neutral-400" />
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
