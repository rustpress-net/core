import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { Database, Trash2, RefreshCw, Tag, Link, Archive }
 from 'lucide-react';
import { cloudflareApi } from '../lib/api';
import toast from 'react-hot-toast';

export function CachePage() {
  const [isPurging, setIsPurging] = useState(false);
  const { register, handleSubmit, reset } = useForm();

  const handlePurgeAll = async () => {
    if (!confirm('Purge entire cache? This cannot be undone.')) return;
    setIsPurging(true);
    try {
      await cloudflareApi.purgeAll();
      toast.success('Entire cache purged successfully');
    } catch (error) {
      toast.error('Failed to purge cache');
    } finally {
      setIsPurging(false);
    }
  };

  const handlePurgeUrls = async (data: any) => {
    const urls = data.urls.split('\n').filter((u: string) => u.trim());
    if (urls.length === 0) return toast.error('Enter at least one URL');

    setIsPurging(true);
    try {
      await cloudflareApi.purgeUrls(urls);
      toast.success(`${urls.length} URLs purged successfully`);
      reset();
    } catch (error) {
      toast.error('Failed to purge URLs');
    } finally {
      setIsPurging(false);
    }
  };

  const handlePurgeTags = async (data: any) => {
    const tags = data.tags.split(',').map((t: string) => t.trim()).filter(Boolean);
    if (tags.length === 0) return toast.error('Enter at least one tag');

    setIsPurging(true);
    try {
      await cloudflareApi.purgeTags(tags);
      toast.success(`Cache purged for ${tags.length} tags`);
      reset();
    } catch (error) {
      toast.error('Failed to purge by tags');
    } finally {
      setIsPurging(false);
    }
  };

  return (
    <div className="p-6">
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-neutral-100 flex items-center gap-3"><Archive className="w-7 h-7 text-orange-400" />Cache Management</h1>
        <p className="text-neutral-400">Manage Cloudflare CDN cache</p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Purge Everything */}
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 bg-danger-500/10 border border-danger-500/20 rounded-lg">
              <Trash2 className="w-5 h-5 text-danger-400" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-neutral-100">Purge Everything</h2>
              <p className="text-sm text-neutral-500">Clear the entire CDN cache</p>
            </div>
          </div>
          <p className="text-sm text-neutral-400 mb-4">
            This will remove all cached files from Cloudflare's edge. Use with caution as it may temporarily increase load on your origin server.
          </p>
          <button
            onClick={handlePurgeAll}
            disabled={isPurging}
            className="w-full px-4 py-2.5 bg-danger-600 hover:bg-danger-700 text-white rounded-lg font-medium disabled:opacity-50 flex items-center justify-center gap-2 transition-colors"
          >
            {isPurging ? <RefreshCw className="w-4 h-4 animate-spin" /> : <Trash2 className="w-4 h-4" />}
            Purge Everything
          </button>
        </div>

        {/* Purge by URL */}
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 bg-primary-500/10 border border-primary-500/20 rounded-lg">
              <Link className="w-5 h-5 text-primary-400" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-neutral-100">Purge by URL</h2>
              <p className="text-sm text-neutral-500">Clear specific URLs from cache</p>
            </div>
          </div>
          <form onSubmit={handleSubmit(handlePurgeUrls)}>
            <textarea
              {...register('urls')}
              placeholder="https://example.com/page1&#10;https://example.com/page2"
              className="w-full h-32 px-3 py-2 border border-neutral-700 rounded-lg bg-neutral-900 text-neutral-100 placeholder-neutral-500 resize-none mb-4 focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
            />
            <button
              type="submit"
              disabled={isPurging}
              className="w-full px-4 py-2.5 bg-primary-600 hover:bg-primary-700 text-white rounded-lg font-medium disabled:opacity-50 transition-colors"
            >
              Purge URLs
            </button>
          </form>
        </div>

        {/* Purge by Tags */}
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 bg-accent-500/10 border border-accent-500/20 rounded-lg">
              <Tag className="w-5 h-5 text-accent-400" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-neutral-100">Purge by Cache Tags</h2>
              <p className="text-sm text-neutral-500">Clear cache by tag (Enterprise)</p>
            </div>
          </div>
          <form onSubmit={handleSubmit(handlePurgeTags)}>
            <input
              {...register('tags')}
              placeholder="tag1, tag2, tag3"
              className="w-full px-3 py-2 border border-neutral-700 rounded-lg bg-neutral-900 text-neutral-100 placeholder-neutral-500 mb-4 focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
            />
            <button
              type="submit"
              disabled={isPurging}
              className="w-full px-4 py-2.5 bg-accent-600 hover:bg-accent-700 text-white rounded-lg font-medium disabled:opacity-50 transition-colors"
            >
              Purge by Tags
            </button>
          </form>
        </div>

        {/* Cache Status */}
        <div className="bg-neutral-800 rounded-xl border border-neutral-700/50 p-6">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 bg-success-500/10 border border-success-500/20 rounded-lg">
              <Database className="w-5 h-5 text-success-400" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-neutral-100">Cache Statistics</h2>
              <p className="text-sm text-neutral-500">Current cache performance</p>
            </div>
          </div>
          <div className="space-y-3">
            <div className="flex justify-between py-2 border-b border-neutral-700/50">
              <span className="text-neutral-400">Cache Level</span>
              <span className="font-medium text-neutral-200">Aggressive</span>
            </div>
            <div className="flex justify-between py-2 border-b border-neutral-700/50">
              <span className="text-neutral-400">Browser TTL</span>
              <span className="font-medium text-neutral-200">4 hours</span>
            </div>
            <div className="flex justify-between py-2">
              <span className="text-neutral-400">Edge TTL</span>
              <span className="font-medium text-neutral-200">24 hours</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
