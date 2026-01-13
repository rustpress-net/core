import { useState, useCallback } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useDropzone } from 'react-dropzone';
import {
  HardDrive,
  Plus,
  Trash2,
  Download,
  Upload,
  RefreshCw,
  Folder,
  File,
  Image,
  FileText,
  FileCode,
  Film,
  Music,
  Archive,
  ChevronRight,
  ChevronLeft,
  Copy,
  ExternalLink,
  X,
  Search,
  Grid,
  List,
  MoreVertical,
} from 'lucide-react';
import { cloudflareApi } from '../lib/api';
import toast from 'react-hot-toast';
import { clsx } from 'clsx';
import { ListSkeleton, CardSkeleton } from '../components/Skeleton';

interface Bucket {
  name: string;
  creation_date: string;
}

interface R2Object {
  key: string;
  size: number;
  last_modified: string;
  etag: string;
}

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

const getFileIcon = (key: string) => {
  const ext = key.split('.').pop()?.toLowerCase();
  if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg'].includes(ext || '')) return Image;
  if (['mp4', 'mov', 'avi', 'webm'].includes(ext || '')) return Film;
  if (['mp3', 'wav', 'ogg', 'flac'].includes(ext || '')) return Music;
  if (['zip', 'tar', 'gz', 'rar', '7z'].includes(ext || '')) return Archive;
  if (['js', 'ts', 'jsx', 'tsx', 'json', 'html', 'css'].includes(ext || '')) return FileCode;
  if (['txt', 'md', 'doc', 'docx', 'pdf'].includes(ext || '')) return FileText;
  return File;
};

export function R2Page() {
  const queryClient = useQueryClient();
  const [selectedBucket, setSelectedBucket] = useState<string | null>(null);
  const [currentPrefix, setCurrentPrefix] = useState('');
  const [showCreateBucketModal, setShowCreateBucketModal] = useState(false);
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('list');
  const [searchTerm, setSearchTerm] = useState('');

  const { data: buckets, isLoading: bucketsLoading } = useQuery({
    queryKey: ['r2-buckets'],
    queryFn: () => cloudflareApi.listR2Buckets(),
  });

  const { data: objects, isLoading: objectsLoading } = useQuery({
    queryKey: ['r2-objects', selectedBucket, currentPrefix],
    queryFn: () => cloudflareApi.listR2Objects(selectedBucket!, currentPrefix),
    enabled: !!selectedBucket,
  });

  const deleteBucketMutation = useMutation({
    mutationFn: (name: string) => cloudflareApi.deleteR2Bucket(name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['r2-buckets'] });
      toast.success('Bucket deleted');
      if (selectedBucket) setSelectedBucket(null);
    },
    onError: () => toast.error('Failed to delete bucket'),
  });

  const uploadMutation = useMutation({
    mutationFn: ({ file, key }: { file: File; key: string }) =>
      cloudflareApi.uploadR2Object(selectedBucket!, key, file),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['r2-objects'] });
      toast.success('File uploaded');
    },
    onError: () => toast.error('Failed to upload file'),
  });

  const deleteObjectMutation = useMutation({
    mutationFn: (key: string) => cloudflareApi.deleteR2Object(selectedBucket!, key),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['r2-objects'] });
      toast.success('Object deleted');
    },
    onError: () => toast.error('Failed to delete object'),
  });

  const onDrop = useCallback(
    (acceptedFiles: File[]) => {
      if (!selectedBucket) return;
      acceptedFiles.forEach((file) => {
        const key = currentPrefix + file.name;
        uploadMutation.mutate({ file, key });
      });
    },
    [selectedBucket, currentPrefix]
  );

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    noClick: true,
  });

  const bucketsList: Bucket[] = buckets?.data?.data || [];
  const objectsList: R2Object[] = objects?.data?.data || [];

  // Filter objects by search
  const filteredObjects = objectsList.filter((obj) =>
    obj.key.toLowerCase().includes(searchTerm.toLowerCase())
  );

  // Separate folders and files
  const folders = new Set<string>();
  const files: R2Object[] = [];

  filteredObjects.forEach((obj) => {
    const relativePath = obj.key.replace(currentPrefix, '');
    if (relativePath.includes('/')) {
      const folderName = relativePath.split('/')[0];
      folders.add(folderName);
    } else {
      files.push(obj);
    }
  });

  const navigateToFolder = (folder: string) => {
    setCurrentPrefix(currentPrefix + folder + '/');
  };

  const navigateUp = () => {
    const parts = currentPrefix.split('/').filter(Boolean);
    parts.pop();
    setCurrentPrefix(parts.length > 0 ? parts.join('/') + '/' : '');
  };

  const breadcrumbs = currentPrefix.split('/').filter(Boolean);

  return (
    <div className="p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-neutral-100 flex items-center gap-3"><HardDrive className="w-7 h-7 text-orange-400" />R2 Storage</h1>
          <p className="text-neutral-400">
            S3-compatible object storage at the edge
          </p>
        </div>
        <button
          onClick={() => setShowCreateBucketModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-orange-600 hover:bg-orange-700 text-white rounded-lg"
        >
          <Plus className="w-4 h-4" />
          Create Bucket
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        {/* Buckets Sidebar */}
        <div className="lg:col-span-1">
          <h3 className="text-sm font-medium text-neutral-300 mb-3">Buckets</h3>
          <div className="bg-neutral-800 rounded-xl border border-neutral-700/50">
            {bucketsLoading ? (
              <div className="p-4">
                <ListSkeleton items={3} />
              </div>
            ) : bucketsList.length === 0 ? (
              <div className="p-8 text-center text-neutral-500">
                <HardDrive className="w-12 h-12 mx-auto mb-3 opacity-50" />
                <p className="text-sm">No buckets</p>
              </div>
            ) : (
              <div className="divide-y divide-neutral-700/50">
                {bucketsList.map((bucket) => (
                  <div
                    key={bucket.name}
                    className={clsx(
                      'flex items-center justify-between p-4 cursor-pointer hover:bg-neutral-700/50',
                      selectedBucket === bucket.name && 'bg-orange-500/10 border-l-2 border-orange-500'
                    )}
                  >
                    <button
                      onClick={() => {
                        setSelectedBucket(bucket.name);
                        setCurrentPrefix('');
                      }}
                      className="flex items-center gap-3 flex-1"
                    >
                      <HardDrive className="w-4 h-4 text-orange-500" />
                      <div className="text-left">
                        <p className="font-medium text-sm text-neutral-100">{bucket.name}</p>
                        <p className="text-xs text-neutral-500">
                          {new Date(bucket.creation_date).toLocaleDateString()}
                        </p>
                      </div>
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        if (confirm('Delete this bucket?')) {
                          deleteBucketMutation.mutate(bucket.name);
                        }
                      }}
                      className="p-1 text-neutral-500 hover:text-red-500"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* Objects Browser */}
        <div className="lg:col-span-3">
          {!selectedBucket ? (
            <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 h-[600px] flex items-center justify-center">
              <div className="text-center text-neutral-500">
                <HardDrive className="w-16 h-16 mx-auto mb-4 opacity-50" />
                <p className="text-lg font-medium text-neutral-300">Select a bucket</p>
                <p className="text-sm">Choose a bucket from the sidebar to browse objects</p>
              </div>
            </div>
          ) : (
            <div
              {...getRootProps()}
              className={clsx(
                'bg-neutral-800 rounded-xl border border-neutral-700/50 min-h-[600px]',
                isDragActive && 'ring-2 ring-orange-500 ring-offset-2 ring-offset-neutral-900'
              )}
            >
              <input {...getInputProps()} />

              {/* Toolbar */}
              <div className="flex items-center justify-between p-4 border-b border-neutral-700/50">
                <div className="flex items-center gap-2">
                  {/* Breadcrumbs */}
                  <button
                    onClick={() => setCurrentPrefix('')}
                    className={clsx(
                      'px-2 py-1 text-sm rounded hover:bg-neutral-700/50 text-neutral-300',
                      !currentPrefix && 'font-semibold text-neutral-100'
                    )}
                  >
                    {selectedBucket}
                  </button>
                  {breadcrumbs.map((crumb, index) => (
                    <div key={index} className="flex items-center">
                      <ChevronRight className="w-4 h-4 text-neutral-500" />
                      <button
                        onClick={() => {
                          const newPrefix = breadcrumbs.slice(0, index + 1).join('/') + '/';
                          setCurrentPrefix(newPrefix);
                        }}
                        className="px-2 py-1 text-sm rounded hover:bg-neutral-700/50 text-neutral-300"
                      >
                        {crumb}
                      </button>
                    </div>
                  ))}
                </div>

                <div className="flex items-center gap-2">
                  <div className="relative">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-neutral-500" />
                    <input
                      type="text"
                      placeholder="Search..."
                      value={searchTerm}
                      onChange={(e) => setSearchTerm(e.target.value)}
                      className="pl-9 pr-4 py-1.5 text-sm border border-neutral-600 rounded-lg bg-neutral-700/50 text-neutral-100 placeholder-neutral-500 w-48"
                    />
                  </div>
                  <button
                    onClick={() => setViewMode(viewMode === 'list' ? 'grid' : 'list')}
                    className="p-2 text-neutral-400 hover:text-neutral-200"
                  >
                    {viewMode === 'list' ? <Grid className="w-4 h-4" /> : <List className="w-4 h-4" />}
                  </button>
                  <label className="flex items-center gap-2 px-4 py-2 bg-orange-600 hover:bg-orange-700 text-white rounded-lg cursor-pointer">
                    <Upload className="w-4 h-4" />
                    Upload
                    <input
                      type="file"
                      multiple
                      className="hidden"
                      onChange={(e) => {
                        if (e.target.files) {
                          Array.from(e.target.files).forEach((file) => {
                            const key = currentPrefix + file.name;
                            uploadMutation.mutate({ file, key });
                          });
                        }
                      }}
                    />
                  </label>
                </div>
              </div>

              {/* Objects */}
              <div className="p-4">
                {isDragActive && (
                  <div className="absolute inset-0 bg-orange-500/20 flex items-center justify-center z-10 rounded-xl border-2 border-dashed border-orange-500">
                    <div className="text-center">
                      <Upload className="w-12 h-12 text-orange-500 mx-auto mb-2" />
                      <p className="text-lg font-medium text-orange-400">
                        Drop files to upload
                      </p>
                    </div>
                  </div>
                )}

                {objectsLoading ? (
                  <div className="py-20 text-center text-neutral-500">
                    <RefreshCw className="w-8 h-8 animate-spin mx-auto mb-3" />
                    Loading objects...
                  </div>
                ) : filteredObjects.length === 0 && folders.size === 0 ? (
                  <div className="py-20 text-center text-neutral-500">
                    <Folder className="w-16 h-16 mx-auto mb-4 opacity-50" />
                    <p className="text-lg font-medium text-neutral-300">Empty folder</p>
                    <p className="text-sm">Drag and drop files here or click Upload</p>
                  </div>
                ) : viewMode === 'list' ? (
                  <table className="w-full">
                    <thead>
                      <tr className="text-left text-xs font-medium text-neutral-500 uppercase">
                        <th className="pb-3">Name</th>
                        <th className="pb-3">Size</th>
                        <th className="pb-3">Modified</th>
                        <th className="pb-3 text-right">Actions</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-neutral-700/50">
                      {currentPrefix && (
                        <tr
                          className="hover:bg-neutral-700/50 cursor-pointer"
                          onClick={navigateUp}
                        >
                          <td className="py-3">
                            <div className="flex items-center gap-3">
                              <ChevronLeft className="w-4 h-4 text-neutral-500" />
                              <span className="text-neutral-500">..</span>
                            </div>
                          </td>
                          <td></td>
                          <td></td>
                          <td></td>
                        </tr>
                      )}
                      {Array.from(folders).map((folder) => (
                        <tr
                          key={folder}
                          className="hover:bg-neutral-700/50 cursor-pointer"
                          onClick={() => navigateToFolder(folder)}
                        >
                          <td className="py-3">
                            <div className="flex items-center gap-3">
                              <Folder className="w-5 h-5 text-orange-500" />
                              <span className="font-medium text-neutral-100">{folder}/</span>
                            </div>
                          </td>
                          <td className="text-neutral-500">-</td>
                          <td className="text-neutral-500">-</td>
                          <td></td>
                        </tr>
                      ))}
                      {files.map((obj) => {
                        const FileIcon = getFileIcon(obj.key);
                        const fileName = obj.key.replace(currentPrefix, '');
                        return (
                          <tr key={obj.key} className="hover:bg-neutral-700/50">
                            <td className="py-3">
                              <div className="flex items-center gap-3">
                                <FileIcon className="w-5 h-5 text-neutral-400" />
                                <span className="text-neutral-100">{fileName}</span>
                              </div>
                            </td>
                            <td className="text-sm text-neutral-500">{formatBytes(obj.size)}</td>
                            <td className="text-sm text-neutral-500">
                              {new Date(obj.last_modified).toLocaleDateString()}
                            </td>
                            <td className="text-right">
                              <button className="p-1 text-neutral-500 hover:text-primary-400 mr-1">
                                <Download className="w-4 h-4" />
                              </button>
                              <button className="p-1 text-neutral-500 hover:text-neutral-200 mr-1">
                                <Copy className="w-4 h-4" />
                              </button>
                              <button
                                onClick={() => {
                                  if (confirm('Delete this object?')) {
                                    deleteObjectMutation.mutate(obj.key);
                                  }
                                }}
                                className="p-1 text-neutral-500 hover:text-red-500"
                              >
                                <Trash2 className="w-4 h-4" />
                              </button>
                            </td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                ) : (
                  <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
                    {currentPrefix && (
                      <button
                        onClick={navigateUp}
                        className="p-4 border border-neutral-700/50 rounded-lg hover:border-orange-500/50 hover:bg-neutral-700/30 text-center"
                      >
                        <ChevronLeft className="w-8 h-8 mx-auto text-neutral-500 mb-2" />
                        <p className="text-sm text-neutral-500">..</p>
                      </button>
                    )}
                    {Array.from(folders).map((folder) => (
                      <button
                        key={folder}
                        onClick={() => navigateToFolder(folder)}
                        className="p-4 border border-neutral-700/50 rounded-lg hover:border-orange-500/50 hover:bg-neutral-700/30 text-center"
                      >
                        <Folder className="w-8 h-8 mx-auto text-orange-500 mb-2" />
                        <p className="text-sm font-medium truncate text-neutral-100">{folder}</p>
                      </button>
                    ))}
                    {files.map((obj) => {
                      const FileIcon = getFileIcon(obj.key);
                      const fileName = obj.key.replace(currentPrefix, '');
                      return (
                        <div
                          key={obj.key}
                          className="p-4 border border-neutral-700/50 rounded-lg hover:border-orange-500/50 hover:bg-neutral-700/30 text-center group relative"
                        >
                          <FileIcon className="w-8 h-8 mx-auto text-neutral-400 mb-2" />
                          <p className="text-sm font-medium truncate text-neutral-100" title={fileName}>
                            {fileName}
                          </p>
                          <p className="text-xs text-neutral-500">{formatBytes(obj.size)}</p>
                          <div className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity">
                            <button
                              onClick={() => {
                                if (confirm('Delete this object?')) {
                                  deleteObjectMutation.mutate(obj.key);
                                }
                              }}
                              className="p-1 text-neutral-400 hover:text-red-500 bg-neutral-800 rounded shadow-lg border border-neutral-700/50"
                            >
                              <Trash2 className="w-3 h-3" />
                            </button>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>

              {/* Upload Progress */}
              {uploadMutation.isPending && (
                <div className="fixed bottom-4 right-4 bg-neutral-800 border border-neutral-700/50 rounded-lg shadow-xl p-4 flex items-center gap-3">
                  <RefreshCw className="w-5 h-5 animate-spin text-orange-500" />
                  <span className="text-neutral-100">Uploading...</span>
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Create Bucket Modal */}
      {showCreateBucketModal && (
        <CreateBucketModal
          onClose={() => setShowCreateBucketModal(false)}
          onSuccess={() => {
            queryClient.invalidateQueries({ queryKey: ['r2-buckets'] });
            setShowCreateBucketModal(false);
          }}
        />
      )}
    </div>
  );
}

function CreateBucketModal({
  onClose,
  onSuccess,
}: {
  onClose: () => void;
  onSuccess: () => void;
}) {
  const [name, setName] = useState('');

  const createMutation = useMutation({
    mutationFn: () => cloudflareApi.createR2Bucket(name),
    onSuccess: () => {
      toast.success('Bucket created');
      onSuccess();
    },
    onError: () => toast.error('Failed to create bucket'),
  });

  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
      <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 shadow-xl w-full max-w-md mx-4">
        <div className="flex items-center justify-between px-6 py-4 border-b border-neutral-700/50">
          <h2 className="text-lg font-semibold text-neutral-100">Create R2 Bucket</h2>
          <button onClick={onClose} className="text-neutral-500 hover:text-neutral-300">
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="p-6 space-y-4">
          <div>
            <label className="block text-sm font-medium text-neutral-300 mb-1">
              Bucket Name
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="my-bucket"
              className="w-full px-3 py-2 border border-neutral-600 rounded-lg bg-neutral-700/50 text-neutral-100 placeholder-neutral-500"
            />
            <p className="text-xs text-neutral-500 mt-1">
              Lowercase letters, numbers, and hyphens only
            </p>
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
              disabled={!name || createMutation.isPending}
              className="flex-1 px-4 py-2 bg-orange-600 hover:bg-orange-700 text-white rounded-lg disabled:opacity-50 flex items-center justify-center gap-2"
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
