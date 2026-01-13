import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Video,
  Upload,
  Play,
  Trash2,
  Copy,
  RefreshCw,
  Radio,
  Settings,
  Eye,
  Clock,
  HardDrive,
  Search,
  Code,
  Loader2,
  X,
} from 'lucide-react';
import { cloudflareApi } from '../lib/api';
import toast from 'react-hot-toast';
import { clsx } from 'clsx';
import { CardSkeleton, ListSkeleton } from '../components/Skeleton';
import { useCloudflareStore } from '../stores/cloudflareStore';

interface StreamVideo {
  uid: string;
  thumbnail?: string;
  readyToStream: boolean;
  status: { state: string };
  meta?: { name?: string };
  created: string;
  duration: number;
  size: number;
  playback?: { hls?: string; dash?: string };
  preview?: string;
}

interface LiveInput {
  uid: string;
  meta?: { name?: string };
  created: string;
  rtmps?: { url?: string; streamKey?: string };
  srt?: { url?: string; streamId?: string };
  webRTC?: { url?: string };
  status?: { current?: { state?: string } };
}

interface StreamStats {
  total_videos: number;
  total_duration_seconds: number;
  total_storage_bytes: number;
  ready_videos: number;
  processing_videos: number;
  live_inputs: number;
  active_live_streams: number;
}

const formatDuration = (seconds: number): string => {
  const hrs = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);
  if (hrs > 0) {
    return `${hrs}:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }
  return `${mins}:${secs.toString().padStart(2, '0')}`;
};

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

export function StreamPage() {
  const { isConnected } = useCloudflareStore();
  const queryClient = useQueryClient();
  const [showLiveModal, setShowLiveModal] = useState(false);
  const [showEmbedModal, setShowEmbedModal] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [newLiveInput, setNewLiveInput] = useState({
    name: '',
    recordingMode: 'automatic',
    timeoutSeconds: 0,
  });

  // Fetch stream statistics
  const { data: stats } = useQuery({
    queryKey: ['stream-stats'],
    queryFn: async () => {
      const response = await cloudflareApi.getStreamStats();
      // API returns { data: stats } or { data: { data: stats } }
      const data = response.data?.data ?? response.data;
      return data as StreamStats;
    },
    enabled: isConnected,
  });

  // Fetch videos
  const { data: videosData, isLoading: videosLoading } = useQuery({
    queryKey: ['stream-videos', searchQuery],
    queryFn: async () => {
      if (searchQuery.trim()) {
        const response = await cloudflareApi.searchStreamVideos(searchQuery);
        const data = response.data?.data ?? response.data;
        return data;
      }
      const response = await cloudflareApi.listStreamVideos();
      // API returns { data: { videos: [...] } }
      const data = response.data?.data ?? response.data;
      return data;
    },
    enabled: isConnected,
  });

  // Fetch live inputs
  const { data: liveInputsData, isLoading: liveInputsLoading } = useQuery({
    queryKey: ['stream-live-inputs'],
    queryFn: async () => {
      const response = await cloudflareApi.listLiveInputs();
      // API returns { data: { live_inputs: [...] } }
      const data = response.data?.data ?? response.data;
      return data;
    },
    enabled: isConnected,
  });

  // Delete video mutation
  const deleteVideoMutation = useMutation({
    mutationFn: async (videoId: string) => {
      await cloudflareApi.deleteStreamVideo(videoId);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['stream-videos'] });
      queryClient.invalidateQueries({ queryKey: ['stream-stats'] });
      toast.success('Video deleted successfully');
    },
    onError: () => {
      toast.error('Failed to delete video');
    },
  });

  // Create live input mutation
  const createLiveInputMutation = useMutation({
    mutationFn: async (options: { name?: string; recordingMode?: string; timeoutSeconds?: number }) => {
      const response = await cloudflareApi.createLiveInput(options);
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['stream-live-inputs'] });
      queryClient.invalidateQueries({ queryKey: ['stream-stats'] });
      toast.success('Live input created successfully');
      setShowLiveModal(false);
      setNewLiveInput({ name: '', recordingMode: 'automatic', timeoutSeconds: 0 });
    },
    onError: () => {
      toast.error('Failed to create live input');
    },
  });

  // Delete live input mutation
  const deleteLiveInputMutation = useMutation({
    mutationFn: async (inputId: string) => {
      await cloudflareApi.deleteLiveInput(inputId);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['stream-live-inputs'] });
      queryClient.invalidateQueries({ queryKey: ['stream-stats'] });
      toast.success('Live input deleted successfully');
    },
    onError: () => {
      toast.error('Failed to delete live input');
    },
  });

  const videosList: StreamVideo[] = videosData?.videos || [];
  const liveInputsList: LiveInput[] = liveInputsData?.live_inputs || [];

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    toast.success('Copied to clipboard');
  };

  const handleDeleteVideo = (videoId: string) => {
    if (confirm('Are you sure you want to delete this video?')) {
      deleteVideoMutation.mutate(videoId);
    }
  };

  const handleDeleteLiveInput = (inputId: string) => {
    if (confirm('Are you sure you want to delete this live input?')) {
      deleteLiveInputMutation.mutate(inputId);
    }
  };

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white flex items-center gap-3"><Video className="w-7 h-7 text-orange-400" />Stream</h1>
          <p className="text-neutral-400 mt-1">
            Video hosting and live streaming at the edge
          </p>
        </div>
        <div className="flex gap-3">
          <button
            onClick={() => setShowLiveModal(true)}
            className="flex items-center gap-2 px-4 py-2 border border-neutral-600 rounded-lg hover:bg-neutral-700/50 text-neutral-300 transition-colors"
          >
            <Radio className="w-4 h-4 text-red-500" />
            Create Live Input
          </button>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary-500/10 rounded-lg">
              <Video className="w-5 h-5 text-primary-400" />
            </div>
            <div>
              <p className="text-sm text-neutral-400">Total Videos</p>
              <p className="text-xl font-bold text-white">{stats?.total_videos || 0}</p>
            </div>
          </div>
        </div>
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-red-500/10 rounded-lg">
              <Radio className="w-5 h-5 text-red-400" />
            </div>
            <div>
              <p className="text-sm text-neutral-400">Live Inputs</p>
              <p className="text-xl font-bold text-white">{stats?.live_inputs || 0}</p>
            </div>
          </div>
        </div>
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-success-500/10 rounded-lg">
              <Clock className="w-5 h-5 text-success-400" />
            </div>
            <div>
              <p className="text-sm text-neutral-400">Total Duration</p>
              <p className="text-xl font-bold text-white">
                {formatDuration(stats?.total_duration_seconds || 0)}
              </p>
            </div>
          </div>
        </div>
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-warning-500/10 rounded-lg">
              <HardDrive className="w-5 h-5 text-warning-400" />
            </div>
            <div>
              <p className="text-sm text-neutral-400">Storage Used</p>
              <p className="text-xl font-bold text-white">
                {formatBytes(stats?.total_storage_bytes || 0)}
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Search */}
      <div className="relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-neutral-500" />
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          placeholder="Search videos..."
          className="w-full pl-10 pr-4 py-2.5 bg-neutral-800 border border-neutral-700 rounded-lg text-white placeholder-neutral-500 focus:outline-none focus:border-primary-500"
        />
      </div>

      {/* Videos Grid */}
      <div className="bg-neutral-800 rounded-xl border border-neutral-700/50">
        <div className="px-6 py-4 border-b border-neutral-700/50 flex items-center justify-between">
          <h2 className="text-lg font-semibold text-white">Videos</h2>
          {stats && (
            <div className="text-sm text-neutral-400">
              {stats.ready_videos} ready, {stats.processing_videos} processing
            </div>
          )}
        </div>

        {videosLoading ? (
          <div className="p-6">
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {[1, 2, 3].map((i) => (
                <CardSkeleton key={i} />
              ))}
            </div>
          </div>
        ) : videosList.length === 0 ? (
          <div className="p-12 text-center">
            <Video className="w-16 h-16 mx-auto mb-4 text-neutral-600" />
            <p className="text-lg font-medium mb-2 text-neutral-300">No videos uploaded</p>
            <p className="text-sm text-neutral-500 mb-4">
              Upload videos via the Cloudflare Dashboard or API
            </p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 p-6">
            {videosList.map((video) => (
              <div
                key={video.uid}
                className="bg-neutral-900/50 border border-neutral-700/50 rounded-lg overflow-hidden hover:border-neutral-600 transition-colors"
              >
                <div className="relative aspect-video bg-neutral-900">
                  {video.thumbnail ? (
                    <img
                      src={video.thumbnail}
                      alt={video.meta?.name || 'Video'}
                      className="w-full h-full object-cover"
                    />
                  ) : (
                    <div className="w-full h-full flex items-center justify-center">
                      <Video className="w-12 h-12 text-neutral-700" />
                    </div>
                  )}
                  <div className="absolute bottom-2 right-2 px-2 py-1 bg-black/70 text-white text-xs rounded">
                    {formatDuration(video.duration)}
                  </div>
                  {!video.readyToStream && (
                    <div className="absolute inset-0 bg-black/50 flex items-center justify-center">
                      <div className="text-center text-white">
                        <RefreshCw className="w-8 h-8 animate-spin mx-auto mb-2" />
                        <p className="text-sm">Processing...</p>
                      </div>
                    </div>
                  )}
                </div>
                <div className="p-4">
                  <h3 className="font-medium text-white truncate">
                    {video.meta?.name || 'Untitled'}
                  </h3>
                  <div className="flex items-center gap-4 mt-2 text-sm text-neutral-500">
                    <span className="flex items-center gap-1">
                      <Clock className="w-3 h-3" />
                      {new Date(video.created).toLocaleDateString()}
                    </span>
                    <span>{formatBytes(video.size)}</span>
                  </div>
                  <div className="flex items-center gap-2 mt-4">
                    {video.preview && (
                      <button
                        onClick={() => window.open(video.preview, '_blank')}
                        className="flex-1 flex items-center justify-center gap-1 px-3 py-1.5 text-sm border border-neutral-600 rounded hover:bg-neutral-700/50 text-neutral-300 transition-colors"
                      >
                        <Play className="w-3 h-3" />
                        Preview
                      </button>
                    )}
                    <button
                      onClick={() => setShowEmbedModal(video.uid)}
                      className="p-1.5 text-neutral-400 hover:text-primary-400 border border-neutral-600 rounded transition-colors"
                      title="Get embed code"
                    >
                      <Code className="w-4 h-4" />
                    </button>
                    {video.playback?.hls && (
                      <button
                        onClick={() => copyToClipboard(video.playback!.hls!)}
                        className="p-1.5 text-neutral-400 hover:text-primary-400 border border-neutral-600 rounded transition-colors"
                        title="Copy HLS URL"
                      >
                        <Copy className="w-4 h-4" />
                      </button>
                    )}
                    <button
                      onClick={() => handleDeleteVideo(video.uid)}
                      disabled={deleteVideoMutation.isPending}
                      className="p-1.5 text-neutral-400 hover:text-red-400 border border-neutral-600 rounded transition-colors disabled:opacity-50"
                      title="Delete video"
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

      {/* Live Inputs Section */}
      <div className="bg-neutral-800 rounded-xl border border-neutral-700/50">
        <div className="px-6 py-4 border-b border-neutral-700/50 flex items-center justify-between">
          <h2 className="text-lg font-semibold text-white flex items-center gap-2">
            <Radio className="w-5 h-5 text-red-400" />
            Live Inputs
          </h2>
          {stats && stats.active_live_streams > 0 && (
            <span className="px-2 py-1 text-xs rounded-full bg-red-500/10 text-red-400 border border-red-500/20">
              {stats.active_live_streams} active
            </span>
          )}
        </div>

        {liveInputsLoading ? (
          <div className="p-6">
            <ListSkeleton items={3} />
          </div>
        ) : liveInputsList.length === 0 ? (
          <div className="p-12 text-center">
            <Radio className="w-16 h-16 mx-auto mb-4 text-neutral-600" />
            <p className="text-lg font-medium mb-2 text-neutral-300">No live inputs configured</p>
            <p className="text-sm text-neutral-500 mb-4">Create a live input to start streaming</p>
            <button
              onClick={() => setShowLiveModal(true)}
              className="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors"
            >
              Create Live Input
            </button>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="text-left text-xs font-medium text-neutral-500 uppercase border-b border-neutral-700/50">
                  <th className="px-6 py-3">Name</th>
                  <th className="px-6 py-3">Status</th>
                  <th className="px-6 py-3">RTMP URL</th>
                  <th className="px-6 py-3">Created</th>
                  <th className="px-6 py-3 text-right">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-neutral-700/50">
                {liveInputsList.map((input) => (
                  <tr key={input.uid} className="hover:bg-neutral-700/20">
                    <td className="px-6 py-4 font-medium text-white">
                      {input.meta?.name || 'Unnamed'}
                    </td>
                    <td className="px-6 py-4">
                      <span
                        className={clsx(
                          'px-2 py-1 text-xs rounded-full',
                          input.status?.current?.state === 'connected'
                            ? 'bg-green-500/10 text-green-400 border border-green-500/20'
                            : 'bg-neutral-700/50 text-neutral-400 border border-neutral-600'
                        )}
                      >
                        {input.status?.current?.state || 'idle'}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      {input.rtmps?.url && (
                        <button
                          onClick={() => copyToClipboard(`${input.rtmps!.url}/${input.rtmps!.streamKey}`)}
                          className="flex items-center gap-2 text-xs bg-neutral-700/50 hover:bg-neutral-700 px-2 py-1 rounded text-neutral-300 transition-colors"
                        >
                          <code className="truncate max-w-[200px]">
                            {input.rtmps.url}
                          </code>
                          <Copy className="w-3 h-3 flex-shrink-0" />
                        </button>
                      )}
                    </td>
                    <td className="px-6 py-4 text-sm text-neutral-500">
                      {new Date(input.created).toLocaleDateString()}
                    </td>
                    <td className="px-6 py-4 text-right">
                      <button
                        onClick={() => handleDeleteLiveInput(input.uid)}
                        disabled={deleteLiveInputMutation.isPending}
                        className="p-1 text-neutral-500 hover:text-red-400 transition-colors disabled:opacity-50"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Create Live Input Modal */}
      {showLiveModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-neutral-800 rounded-xl border border-neutral-700 p-6 w-full max-w-md">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold text-white">Create Live Input</h2>
              <button
                onClick={() => setShowLiveModal(false)}
                className="text-neutral-400 hover:text-white transition-colors"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-neutral-300 mb-1">Name</label>
                <input
                  type="text"
                  value={newLiveInput.name}
                  onChange={(e) => setNewLiveInput({ ...newLiveInput, name: e.target.value })}
                  placeholder="My Live Stream"
                  className="w-full bg-neutral-700 border border-neutral-600 rounded-lg px-4 py-2 text-white placeholder-neutral-400 focus:outline-none focus:border-primary-500"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-neutral-300 mb-1">Recording Mode</label>
                <select
                  value={newLiveInput.recordingMode}
                  onChange={(e) => setNewLiveInput({ ...newLiveInput, recordingMode: e.target.value })}
                  className="w-full bg-neutral-700 border border-neutral-600 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-primary-500"
                >
                  <option value="off">Off</option>
                  <option value="automatic">Automatic</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-neutral-300 mb-1">
                  Timeout (seconds)
                </label>
                <input
                  type="number"
                  value={newLiveInput.timeoutSeconds}
                  onChange={(e) => setNewLiveInput({ ...newLiveInput, timeoutSeconds: parseInt(e.target.value) || 0 })}
                  placeholder="0 (no timeout)"
                  className="w-full bg-neutral-700 border border-neutral-600 rounded-lg px-4 py-2 text-white placeholder-neutral-400 focus:outline-none focus:border-primary-500"
                />
                <p className="text-xs text-neutral-500 mt-1">
                  0 means no timeout. The stream will stay active until manually stopped.
                </p>
              </div>
            </div>

            <div className="flex justify-end gap-3 mt-6">
              <button
                onClick={() => setShowLiveModal(false)}
                className="px-4 py-2 text-neutral-400 hover:text-white transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={() => createLiveInputMutation.mutate(newLiveInput)}
                disabled={createLiveInputMutation.isPending}
                className="flex items-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-700 disabled:bg-neutral-600 text-white rounded-lg transition-colors"
              >
                {createLiveInputMutation.isPending && <Loader2 className="w-4 h-4 animate-spin" />}
                Create
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Embed Code Modal */}
      {showEmbedModal && (
        <EmbedCodeModal
          videoId={showEmbedModal}
          onClose={() => setShowEmbedModal(null)}
        />
      )}
    </div>
  );
}

// Embed Code Modal Component
function EmbedCodeModal({ videoId, onClose }: { videoId: string; onClose: () => void }) {
  const [embedOptions, setEmbedOptions] = useState({
    autoplay: false,
    muted: false,
    loop: false,
    controls: true,
  });

  const { data: embedData, isLoading } = useQuery({
    queryKey: ['stream-embed', videoId, embedOptions],
    queryFn: async () => {
      const response = await cloudflareApi.getStreamEmbedCode(videoId, {
        autoplay: embedOptions.autoplay,
        muted: embedOptions.muted,
        loop: embedOptions.loop,
        controls: embedOptions.controls,
      });
      // API returns { data: { html: "..." } }
      const data = response.data?.data ?? response.data;
      return data;
    },
  });

  const copyEmbed = () => {
    const html = embedData?.html;
    if (html) {
      navigator.clipboard.writeText(html);
      toast.success('Embed code copied to clipboard');
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-neutral-800 rounded-xl border border-neutral-700 p-6 w-full max-w-lg">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-white">Embed Code</h2>
          <button
            onClick={onClose}
            className="text-neutral-400 hover:text-white transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="space-y-4">
          <div className="grid grid-cols-2 gap-3">
            <label className="flex items-center gap-2 text-sm text-neutral-300">
              <input
                type="checkbox"
                checked={embedOptions.autoplay}
                onChange={(e) => setEmbedOptions({ ...embedOptions, autoplay: e.target.checked })}
                className="rounded border-neutral-600 bg-neutral-700 text-primary-500 focus:ring-primary-500"
              />
              Autoplay
            </label>
            <label className="flex items-center gap-2 text-sm text-neutral-300">
              <input
                type="checkbox"
                checked={embedOptions.muted}
                onChange={(e) => setEmbedOptions({ ...embedOptions, muted: e.target.checked })}
                className="rounded border-neutral-600 bg-neutral-700 text-primary-500 focus:ring-primary-500"
              />
              Muted
            </label>
            <label className="flex items-center gap-2 text-sm text-neutral-300">
              <input
                type="checkbox"
                checked={embedOptions.loop}
                onChange={(e) => setEmbedOptions({ ...embedOptions, loop: e.target.checked })}
                className="rounded border-neutral-600 bg-neutral-700 text-primary-500 focus:ring-primary-500"
              />
              Loop
            </label>
            <label className="flex items-center gap-2 text-sm text-neutral-300">
              <input
                type="checkbox"
                checked={embedOptions.controls}
                onChange={(e) => setEmbedOptions({ ...embedOptions, controls: e.target.checked })}
                className="rounded border-neutral-600 bg-neutral-700 text-primary-500 focus:ring-primary-500"
              />
              Show Controls
            </label>
          </div>

          <div>
            <label className="block text-sm font-medium text-neutral-300 mb-1">HTML Code</label>
            {isLoading ? (
              <div className="h-32 bg-neutral-900 rounded-lg flex items-center justify-center">
                <Loader2 className="w-6 h-6 animate-spin text-neutral-500" />
              </div>
            ) : (
              <pre className="bg-neutral-900 border border-neutral-700 rounded-lg p-3 text-sm text-neutral-300 overflow-x-auto max-h-40">
                {embedData?.html || 'Loading...'}
              </pre>
            )}
          </div>
        </div>

        <div className="flex justify-end gap-3 mt-6">
          <button
            onClick={onClose}
            className="px-4 py-2 text-neutral-400 hover:text-white transition-colors"
          >
            Close
          </button>
          <button
            onClick={copyEmbed}
            disabled={!embedData?.html}
            className="flex items-center gap-2 px-4 py-2 bg-primary-500 hover:bg-primary-600 disabled:bg-neutral-600 text-white rounded-lg transition-colors"
          >
            <Copy className="w-4 h-4" />
            Copy Code
          </button>
        </div>
      </div>
    </div>
  );
}
