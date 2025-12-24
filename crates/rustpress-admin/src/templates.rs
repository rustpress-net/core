//! Admin dashboard HTML templates

/// Dashboard HTML template
pub const DASHBOARD_HTML: &str = r##"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RustPress Admin Dashboard</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://unpkg.com/alpinejs@3.x.x/dist/cdn.min.js" defer></script>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        [x-cloak] { display: none !important; }
    </style>
</head>
<body class="bg-gray-100 dark:bg-gray-900" x-data="dashboard()">
    <div class="min-h-screen flex">
        <!-- Sidebar -->
        <aside class="w-64 bg-white dark:bg-gray-800 shadow-md">
            <div class="p-4 border-b dark:border-gray-700">
                <h1 class="text-xl font-bold text-gray-800 dark:text-white">RustPress</h1>
                <p class="text-sm text-gray-500 dark:text-gray-400">Admin Dashboard</p>
            </div>
            <nav class="p-4">
                <ul class="space-y-2">
                    <li><a href="#dashboard" @click="currentPage = 'dashboard'" class="flex items-center p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"><span class="mr-3">Dashboard</span></a></li>
                    <li><a href="#system" @click="currentPage = 'system'" class="flex items-center p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"><span class="mr-3">System</span></a></li>
                    <li><a href="#database" @click="currentPage = 'database'" class="flex items-center p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"><span class="mr-3">Database</span></a></li>
                    <li><a href="#cache" @click="currentPage = 'cache'" class="flex items-center p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"><span class="mr-3">Cache</span></a></li>
                    <li><a href="#cdn" @click="currentPage = 'cdn'" class="flex items-center p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"><span class="mr-3">CDN</span></a></li>
                    <li><a href="#backups" @click="currentPage = 'backups'" class="flex items-center p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"><span class="mr-3">Backups</span></a></li>
                    <li><a href="#logs" @click="currentPage = 'logs'" class="flex items-center p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"><span class="mr-3">Logs</span></a></li>
                    <li><a href="#settings" @click="currentPage = 'settings'" class="flex items-center p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"><span class="mr-3">Settings</span></a></li>
                </ul>
            </nav>
        </aside>

        <!-- Main Content -->
        <main class="flex-1 p-8">
            <header class="mb-8 flex justify-between items-center">
                <div>
                    <h2 class="text-2xl font-bold text-gray-800 dark:text-white" x-text="pageTitle"></h2>
                    <p class="text-gray-500 dark:text-gray-400" x-text="pageDescription"></p>
                </div>
                <div class="flex items-center space-x-4">
                    <span class="text-sm text-gray-500" x-text="'Last updated: ' + lastUpdated"></span>
                    <button @click="refreshData()" class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700">Refresh</button>
                </div>
            </header>

            <!-- Dashboard Page -->
            <div x-show="currentPage === 'dashboard'" x-cloak>
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                        <div class="flex items-center">
                            <div class="p-3 bg-blue-100 dark:bg-blue-900 rounded-full"><span class="text-2xl">Posts</span></div>
                            <div class="ml-4">
                                <p class="text-sm text-gray-500 dark:text-gray-400">Total Posts</p>
                                <p class="text-2xl font-bold text-gray-800 dark:text-white" x-text="data.stats?.total_posts || 0"></p>
                            </div>
                        </div>
                    </div>
                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                        <div class="flex items-center">
                            <div class="p-3 bg-green-100 dark:bg-green-900 rounded-full"><span class="text-2xl">Users</span></div>
                            <div class="ml-4">
                                <p class="text-sm text-gray-500 dark:text-gray-400">Total Users</p>
                                <p class="text-2xl font-bold text-gray-800 dark:text-white" x-text="data.stats?.total_users || 0"></p>
                            </div>
                        </div>
                    </div>
                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                        <div class="flex items-center">
                            <div class="p-3 bg-yellow-100 dark:bg-yellow-900 rounded-full"><span class="text-2xl">Views</span></div>
                            <div class="ml-4">
                                <p class="text-sm text-gray-500 dark:text-gray-400">Views Today</p>
                                <p class="text-2xl font-bold text-gray-800 dark:text-white" x-text="data.stats?.views_today || 0"></p>
                            </div>
                        </div>
                    </div>
                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                        <div class="flex items-center">
                            <div class="p-3 bg-purple-100 dark:bg-purple-900 rounded-full"><span class="text-2xl">Comments</span></div>
                            <div class="ml-4">
                                <p class="text-sm text-gray-500 dark:text-gray-400">Comments</p>
                                <p class="text-2xl font-bold text-gray-800 dark:text-white" x-text="data.stats?.total_comments || 0"></p>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- System Status -->
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                        <h3 class="text-lg font-semibold text-gray-800 dark:text-white mb-4">System Resources</h3>
                        <div class="space-y-4">
                            <div>
                                <div class="flex justify-between mb-1">
                                    <span class="text-sm text-gray-600 dark:text-gray-400">CPU Usage</span>
                                    <span class="text-sm font-medium text-gray-800 dark:text-white" x-text="(data.system?.cpu_usage || 0).toFixed(1) + '%'"></span>
                                </div>
                                <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                                    <div class="bg-blue-600 h-2 rounded-full" x-bind:style="'width: ' + (data.system?.cpu_usage || 0) + '%'"></div>
                                </div>
                            </div>
                            <div>
                                <div class="flex justify-between mb-1">
                                    <span class="text-sm text-gray-600 dark:text-gray-400">Memory Usage</span>
                                    <span class="text-sm font-medium text-gray-800 dark:text-white" x-text="(data.system?.memory_usage || 0).toFixed(1) + '%'"></span>
                                </div>
                                <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                                    <div class="bg-green-600 h-2 rounded-full" x-bind:style="'width: ' + (data.system?.memory_usage || 0) + '%'"></div>
                                </div>
                            </div>
                            <div>
                                <div class="flex justify-between mb-1">
                                    <span class="text-sm text-gray-600 dark:text-gray-400">Disk Usage</span>
                                    <span class="text-sm font-medium text-gray-800 dark:text-white" x-text="(data.system?.disk_usage || 0).toFixed(1) + '%'"></span>
                                </div>
                                <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                                    <div class="bg-yellow-600 h-2 rounded-full" x-bind:style="'width: ' + (data.system?.disk_usage || 0) + '%'"></div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                        <h3 class="text-lg font-semibold text-gray-800 dark:text-white mb-4">Service Status</h3>
                        <div class="space-y-3">
                            <div class="flex items-center justify-between">
                                <span class="text-gray-600 dark:text-gray-400">Application</span>
                                <span class="px-2 py-1 text-xs rounded-full bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300">Running</span>
                            </div>
                            <div class="flex items-center justify-between">
                                <span class="text-gray-600 dark:text-gray-400">Database</span>
                                <span class="px-2 py-1 text-xs rounded-full bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300" x-text="data.database?.connected ? 'Connected' : 'Disconnected'"></span>
                            </div>
                            <div class="flex items-center justify-between">
                                <span class="text-gray-600 dark:text-gray-400">Cache</span>
                                <span class="px-2 py-1 text-xs rounded-full bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300" x-text="data.cache?.connected ? 'Connected' : 'Memory Mode'"></span>
                            </div>
                            <div class="flex items-center justify-between">
                                <span class="text-gray-600 dark:text-gray-400">CDN</span>
                                <span class="px-2 py-1 text-xs rounded-full bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300" x-text="data.cdn?.connected ? 'Active' : 'Not Configured'"></span>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Recent Activity -->
                <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                    <h3 class="text-lg font-semibold text-gray-800 dark:text-white mb-4">Recent Activity</h3>
                    <div class="space-y-4">
                        <template x-for="activity in data.activity?.slice(0, 5) || []" x-bind:key="activity.id">
                            <div class="flex items-center p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
                                <div class="flex-1">
                                    <p class="text-sm text-gray-800 dark:text-white" x-text="activity.description"></p>
                                    <p class="text-xs text-gray-500 dark:text-gray-400" x-text="new Date(activity.timestamp).toLocaleString()"></p>
                                </div>
                            </div>
                        </template>
                        <p x-show="!data.activity?.length" class="text-gray-500 dark:text-gray-400 text-center py-4">No recent activity</p>
                    </div>
                </div>
            </div>

            <!-- System Page -->
            <div x-show="currentPage === 'system'" x-cloak>
                <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                    <h3 class="text-lg font-semibold mb-4">System Information</h3>
                    <div class="grid grid-cols-2 gap-4">
                        <div><span class="text-gray-500">OS:</span> <span x-text="data.system?.os_name"></span></div>
                        <div><span class="text-gray-500">Version:</span> <span x-text="data.system?.os_version"></span></div>
                        <div><span class="text-gray-500">Hostname:</span> <span x-text="data.system?.hostname"></span></div>
                        <div><span class="text-gray-500">Uptime:</span> <span x-text="formatDuration(data.system?.uptime)"></span></div>
                    </div>
                </div>
            </div>

            <!-- Cache Page -->
            <div x-show="currentPage === 'cache'" x-cloak>
                <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                    <div class="flex justify-between items-center mb-4">
                        <h3 class="text-lg font-semibold">Cache Management</h3>
                        <button @click="purgeCache()" class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700">Purge All Cache</button>
                    </div>
                    <div class="grid grid-cols-2 gap-4">
                        <div><span class="text-gray-500">Driver:</span> <span x-text="data.cache?.driver"></span></div>
                        <div><span class="text-gray-500">Hit Rate:</span> <span x-text="(data.cache?.hit_rate || 0).toFixed(1) + '%'"></span></div>
                        <div><span class="text-gray-500">Total Keys:</span> <span x-text="data.cache?.total_keys"></span></div>
                        <div><span class="text-gray-500">Memory Used:</span> <span x-text="formatBytes(data.cache?.memory_used)"></span></div>
                    </div>
                </div>
            </div>
        </main>
    </div>

    <script>
        function dashboard() {
            return {
                currentPage: 'dashboard',
                data: {},
                lastUpdated: 'Loading...',

                get pageTitle() {
                    const titles = {
                        dashboard: 'Dashboard',
                        system: 'System Status',
                        database: 'Database Management',
                        cache: 'Cache Management',
                        cdn: 'CDN Configuration',
                        backups: 'Backups',
                        logs: 'System Logs',
                        settings: 'Settings'
                    };
                    return titles[this.currentPage] || 'Dashboard';
                },

                get pageDescription() {
                    const descriptions = {
                        dashboard: 'Overview of your RustPress installation',
                        system: 'System resources and health',
                        database: 'Database status and management',
                        cache: 'Cache statistics and management',
                        cdn: 'Content delivery network settings',
                        backups: 'Backup and restore operations',
                        logs: 'View and manage system logs',
                        settings: 'Application settings'
                    };
                    return descriptions[this.currentPage] || '';
                },

                async init() {
                    await this.refreshData();
                    setInterval(() => this.refreshData(), 30000);
                },

                async refreshData() {
                    try {
                        const response = await fetch('/admin/api/dashboard');
                        const result = await response.json();
                        if (result.success) {
                            this.data = result.data;
                            this.lastUpdated = new Date().toLocaleTimeString();
                        }
                    } catch (error) {
                        console.error('Failed to refresh data:', error);
                    }
                },

                async purgeCache() {
                    if (!confirm('Are you sure you want to purge all cache?')) return;
                    try {
                        await fetch('/admin/api/cache/purge', { method: 'POST' });
                        alert('Cache purged successfully!');
                        await this.refreshData();
                    } catch (error) {
                        alert('Failed to purge cache');
                    }
                },

                formatBytes(bytes) {
                    if (!bytes) return '0 B';
                    const k = 1024;
                    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
                    const i = Math.floor(Math.log(bytes) / Math.log(k));
                    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
                },

                formatDuration(seconds) {
                    if (!seconds) return '0m';
                    const d = Math.floor(seconds / 86400);
                    const h = Math.floor((seconds % 86400) / 3600);
                    const m = Math.floor((seconds % 3600) / 60);
                    if (d > 0) return d + 'd ' + h + 'h ' + m + 'm';
                    if (h > 0) return h + 'h ' + m + 'm';
                    return m + 'm';
                }
            }
        }
    </script>
</body>
</html>
"##;
