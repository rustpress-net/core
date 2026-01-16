/**
 * HtmlCssPreview - Real-time preview for HTML, CSS, and SVG files
 * Shows live preview with hot reload as you type
 * Injects real-time theme data (posts, products, site config, etc.)
 */

import React, { useEffect, useRef, useState, useMemo, useCallback } from 'react';
import { motion } from 'framer-motion';
import {
  Monitor, Tablet, Smartphone, RefreshCw,
  Maximize2, Minimize2, Sun, Moon, ZoomIn, ZoomOut,
  RotateCcw, Database, AlertCircle, CheckCircle
} from 'lucide-react';
import {
  realTimeThemeDataService,
  generateRealTimeContextScript,
  processTemplateVariables,
  type ThemeDataContext
} from '../../services/themeDataService';

// ============================================
// TYPES
// ============================================

interface HtmlCssPreviewProps {
  content: string;
  language: 'html' | 'css' | 'svg';
  filePath?: string;
  /** Enable real-time data injection from backend */
  enableLiveData?: boolean;
  /** Page type for context (affects which data is prioritized) */
  pageType?: 'home' | 'post' | 'category' | 'product' | 'page';
}

type DeviceMode = 'desktop' | 'tablet' | 'mobile';
type ThemeMode = 'light' | 'dark';

interface DevicePreset {
  name: string;
  width: number;
  height: number;
  icon: React.ElementType;
}

const DEVICE_PRESETS: Record<DeviceMode, DevicePreset> = {
  desktop: { name: 'Desktop', width: 1920, height: 1080, icon: Monitor },
  tablet: { name: 'Tablet', width: 768, height: 1024, icon: Tablet },
  mobile: { name: 'Mobile', width: 375, height: 667, icon: Smartphone }
};

// ============================================
// MAIN COMPONENT
// ============================================

export const HtmlCssPreview: React.FC<HtmlCssPreviewProps> = ({
  content,
  language,
  filePath = '',
  enableLiveData = true,
  pageType = 'home'
}) => {
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const [deviceMode, setDeviceMode] = useState<DeviceMode>('desktop');
  const [themeMode, setThemeMode] = useState<ThemeMode>('light');
  const [zoom, setZoom] = useState(100);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());
  const [error, setError] = useState<string | null>(null);

  // Real-time data state
  const [themeData, setThemeData] = useState<ThemeDataContext | null>(null);
  const [dataLoading, setDataLoading] = useState(false);
  const [dataError, setDataError] = useState<string | null>(null);
  const [dataSource, setDataSource] = useState<'cache' | 'backend' | 'mock'>('mock');

  const fileName = filePath.split('/').pop() || 'preview';

  // Fetch theme data on mount (with caching - only one DB call)
  useEffect(() => {
    if (!enableLiveData) return;

    let mounted = true;
    const fetchData = async () => {
      setDataLoading(true);
      setDataError(null);

      try {
        const data = await realTimeThemeDataService.fetchAllThemeData();
        if (mounted) {
          setThemeData(data);
          // Determine data source from logs (simplified check)
          setDataSource('cache');
        }
      } catch (err) {
        if (mounted) {
          setDataError(err instanceof Error ? err.message : 'Failed to load data');
          setDataSource('mock');
        }
      } finally {
        if (mounted) {
          setDataLoading(false);
        }
      }
    };

    fetchData();

    // Subscribe to real-time updates
    const unsubscribe = realTimeThemeDataService.subscribeToUpdates((type, data) => {
      console.log('[HtmlCssPreview] Received real-time update:', type);
      // Refetch data when updates occur
      fetchData();
    });

    return () => {
      mounted = false;
      unsubscribe();
    };
  }, [enableLiveData]);

  // Force refresh data
  const refreshData = useCallback(async () => {
    realTimeThemeDataService.invalidateCache();
    setDataLoading(true);
    try {
      const data = await realTimeThemeDataService.fetchAllThemeData();
      setThemeData(data);
      setDataError(null);
      setLastUpdate(new Date());
    } catch (err) {
      setDataError(err instanceof Error ? err.message : 'Failed to refresh data');
    } finally {
      setDataLoading(false);
    }
  }, []);

  // Generate the full HTML document for preview
  const previewHtml = useMemo(() => {
    // Generate context script if data is available
    const contextScript = themeData && enableLiveData
      ? generateRealTimeContextScript(themeData as ThemeDataContext & { products?: any[] })
      : '';

    // Process template variables in content
    let processedContent = content;
    if (themeData && enableLiveData) {
      processedContent = processTemplateVariables(content, themeData);
    }

    // Base styles for preview
    const baseStyles = `
      <style>
        * { box-sizing: border-box; }
        body {
          margin: 0;
          padding: 16px;
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
          background: ${themeMode === 'dark' ? '#1a1a2e' : '#ffffff'};
          color: ${themeMode === 'dark' ? '#e0e0e0' : '#333333'};
          min-height: 100vh;
        }
        /* CSS Preview Mode - Show styled elements */
        .css-preview-container {
          display: flex;
          flex-direction: column;
          gap: 24px;
        }
        .css-preview-section {
          padding: 16px;
          border: 1px solid ${themeMode === 'dark' ? '#333' : '#e0e0e0'};
          border-radius: 8px;
        }
        .css-preview-section h3 {
          margin: 0 0 12px 0;
          font-size: 14px;
          color: ${themeMode === 'dark' ? '#888' : '#666'};
          text-transform: uppercase;
          letter-spacing: 0.5px;
        }
        /* SVG Preview container */
        .svg-preview-container {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          min-height: calc(100vh - 32px);
          gap: 16px;
        }
        .svg-preview-wrapper {
          padding: 24px;
          background: ${themeMode === 'dark' ? '#2a2a3e' : '#f5f5f5'};
          border-radius: 12px;
          display: flex;
          align-items: center;
          justify-content: center;
        }
        .svg-preview-wrapper svg {
          max-width: 100%;
          height: auto;
        }
        .svg-info {
          font-size: 12px;
          color: ${themeMode === 'dark' ? '#888' : '#666'};
        }
      </style>
    `;

    if (language === 'html') {
      // Check if it's a complete HTML document
      if (processedContent.includes('<html') || processedContent.includes('<!DOCTYPE')) {
        // Inject our styles and context script into the head
        let html = processedContent;
        if (html.includes('</head>')) {
          html = html.replace('</head>', `${baseStyles}${contextScript}</head>`);
        } else if (html.includes('<body')) {
          html = html.replace('<body', `<head>${baseStyles}${contextScript}</head><body`);
        } else {
          // No head or body, inject at the beginning
          html = `${baseStyles}${contextScript}${html}`;
        }
        return html;
      } else {
        // It's an HTML fragment, wrap it in a full document with live data
        return `
          <!DOCTYPE html>
          <html lang="en">
          <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Preview - ${fileName}</title>
            ${baseStyles}
            ${contextScript}
          </head>
          <body>
            ${processedContent}
          </body>
          </html>
        `;
      }
    } else if (language === 'svg') {
      // SVG Preview - render the SVG directly
      return `
        <!DOCTYPE html>
        <html lang="en">
        <head>
          <meta charset="UTF-8">
          <meta name="viewport" content="width=device-width, initial-scale=1.0">
          <title>SVG Preview - ${fileName}</title>
          ${baseStyles}
        </head>
        <body>
          <div class="svg-preview-container">
            <div class="svg-preview-wrapper">
              ${processedContent}
            </div>
            <div class="svg-info">
              ${fileName}
            </div>
          </div>
        </body>
        </html>
      `;
    } else if (language === 'css') {
      // For CSS files, create a demo page with various elements
      return `
        <!DOCTYPE html>
        <html lang="en">
        <head>
          <meta charset="UTF-8">
          <meta name="viewport" content="width=device-width, initial-scale=1.0">
          <title>CSS Preview - ${fileName}</title>
          ${baseStyles}
          <style>${content}</style>
        </head>
        <body>
          <div class="css-preview-container">
            <div class="css-preview-section">
              <h3>Typography</h3>
              <h1>Heading 1</h1>
              <h2>Heading 2</h2>
              <h3>Heading 3</h3>
              <p>This is a paragraph with some <strong>bold text</strong> and <em>italic text</em>.
                 You can also have <a href="#">links</a> and <code>inline code</code>.</p>
              <blockquote>This is a blockquote. It's often used for quotes or highlighted content.</blockquote>
            </div>

            <div class="css-preview-section">
              <h3>Buttons</h3>
              <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                <button>Default Button</button>
                <button class="primary">Primary Button</button>
                <button class="secondary">Secondary Button</button>
                <button class="success">Success Button</button>
                <button class="danger">Danger Button</button>
                <button disabled>Disabled Button</button>
              </div>
            </div>

            <div class="css-preview-section">
              <h3>Form Elements</h3>
              <form style="display: flex; flex-direction: column; gap: 12px; max-width: 400px;">
                <div>
                  <label for="input">Text Input</label>
                  <input type="text" id="input" placeholder="Enter text..." style="width: 100%;">
                </div>
                <div>
                  <label for="email">Email Input</label>
                  <input type="email" id="email" placeholder="email@example.com" style="width: 100%;">
                </div>
                <div>
                  <label for="select">Select</label>
                  <select id="select" style="width: 100%;">
                    <option>Option 1</option>
                    <option>Option 2</option>
                    <option>Option 3</option>
                  </select>
                </div>
                <div>
                  <label for="textarea">Textarea</label>
                  <textarea id="textarea" rows="3" placeholder="Enter message..." style="width: 100%;"></textarea>
                </div>
                <div style="display: flex; gap: 16px;">
                  <label><input type="checkbox"> Checkbox</label>
                  <label><input type="radio" name="radio"> Radio 1</label>
                  <label><input type="radio" name="radio"> Radio 2</label>
                </div>
              </form>
            </div>

            <div class="css-preview-section">
              <h3>Lists</h3>
              <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 24px;">
                <div>
                  <h4>Unordered List</h4>
                  <ul>
                    <li>List item one</li>
                    <li>List item two</li>
                    <li>List item three
                      <ul>
                        <li>Nested item</li>
                        <li>Nested item</li>
                      </ul>
                    </li>
                  </ul>
                </div>
                <div>
                  <h4>Ordered List</h4>
                  <ol>
                    <li>First item</li>
                    <li>Second item</li>
                    <li>Third item</li>
                  </ol>
                </div>
              </div>
            </div>

            <div class="css-preview-section">
              <h3>Cards & Containers</h3>
              <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 16px;">
                <div class="card">
                  <h4>Card Title</h4>
                  <p>Card content goes here. This is a simple card component.</p>
                </div>
                <div class="card">
                  <h4>Another Card</h4>
                  <p>More card content. Cards are useful for organizing content.</p>
                </div>
                <div class="card">
                  <h4>Third Card</h4>
                  <p>Even more content in this third card example.</p>
                </div>
              </div>
            </div>

            <div class="css-preview-section">
              <h3>Table</h3>
              <table style="width: 100%; border-collapse: collapse;">
                <thead>
                  <tr>
                    <th>Name</th>
                    <th>Email</th>
                    <th>Role</th>
                    <th>Status</th>
                  </tr>
                </thead>
                <tbody>
                  <tr>
                    <td>John Doe</td>
                    <td>john@example.com</td>
                    <td>Admin</td>
                    <td><span class="badge success">Active</span></td>
                  </tr>
                  <tr>
                    <td>Jane Smith</td>
                    <td>jane@example.com</td>
                    <td>Editor</td>
                    <td><span class="badge warning">Pending</span></td>
                  </tr>
                  <tr>
                    <td>Bob Wilson</td>
                    <td>bob@example.com</td>
                    <td>Viewer</td>
                    <td><span class="badge danger">Inactive</span></td>
                  </tr>
                </tbody>
              </table>
            </div>

            <div class="css-preview-section">
              <h3>Alerts & Badges</h3>
              <div style="display: flex; flex-direction: column; gap: 8px;">
                <div class="alert info">This is an info alert message.</div>
                <div class="alert success">This is a success alert message.</div>
                <div class="alert warning">This is a warning alert message.</div>
                <div class="alert danger">This is a danger alert message.</div>
              </div>
              <div style="margin-top: 16px; display: flex; gap: 8px;">
                <span class="badge">Default</span>
                <span class="badge primary">Primary</span>
                <span class="badge success">Success</span>
                <span class="badge warning">Warning</span>
                <span class="badge danger">Danger</span>
              </div>
            </div>
          </div>
        </body>
        </html>
      `;
    }

    return '';
  }, [content, language, fileName, themeMode, themeData, enableLiveData]);

  // Update iframe content
  useEffect(() => {
    if (iframeRef.current && previewHtml) {
      try {
        const iframe = iframeRef.current;
        const doc = iframe.contentDocument || iframe.contentWindow?.document;

        if (doc) {
          doc.open();
          doc.write(previewHtml);
          doc.close();
          setError(null);
          setLastUpdate(new Date());
        }
      } catch (err) {
        setError('Failed to update preview');
        console.error('Preview update error:', err);
      }
    }
  }, [previewHtml]);

  // Handle zoom
  const handleZoom = (delta: number) => {
    setZoom(prev => Math.min(200, Math.max(25, prev + delta)));
  };

  // Reset zoom
  const resetZoom = () => setZoom(100);

  // Get device dimensions
  const device = DEVICE_PRESETS[deviceMode];
  const DeviceIcon = device.icon;

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      className={`flex flex-col bg-gray-900 h-full ${
        isFullscreen ? 'fixed inset-0 z-50' : ''
      }`}
    >
      {/* Toolbar */}
      <div className="flex items-center justify-between px-3 py-2 bg-gray-800 border-b border-gray-700">
        <div className="flex items-center gap-2">
          <span className="text-xs font-medium text-gray-300">
            {language.toUpperCase()} Preview
          </span>
          <span className="text-xs text-gray-500">
            {lastUpdate.toLocaleTimeString()}
          </span>

          {/* Live Data Status Indicator */}
          {enableLiveData && (
            <div className="flex items-center gap-1.5 ml-2 pl-2 border-l border-gray-700">
              {dataLoading ? (
                <span className="flex items-center gap-1 text-xs text-blue-400">
                  <div className="w-2 h-2 border border-blue-400 border-t-transparent rounded-full animate-spin" />
                  Loading data...
                </span>
              ) : dataError ? (
                <span className="flex items-center gap-1 text-xs text-yellow-400" title={dataError}>
                  <AlertCircle className="w-3 h-3" />
                  Using fallback
                </span>
              ) : (
                <span className="flex items-center gap-1 text-xs text-green-400">
                  <CheckCircle className="w-3 h-3" />
                  Live data
                </span>
              )}
              <button
                onClick={refreshData}
                disabled={dataLoading}
                className="p-1 text-gray-400 hover:text-white hover:bg-gray-700 rounded disabled:opacity-50"
                title="Refresh live data (clears cache)"
              >
                <Database className={`w-3 h-3 ${dataLoading ? 'animate-pulse' : ''}`} />
              </button>
            </div>
          )}
        </div>

        <div className="flex items-center gap-1">
          {/* Device Mode Buttons */}
          {Object.entries(DEVICE_PRESETS).map(([mode, preset]) => {
            const Icon = preset.icon;
            return (
              <button
                key={mode}
                onClick={() => setDeviceMode(mode as DeviceMode)}
                className={`p-1.5 rounded transition-colors ${
                  deviceMode === mode
                    ? 'bg-blue-600 text-white'
                    : 'text-gray-400 hover:text-white hover:bg-gray-700'
                }`}
                title={preset.name}
              >
                <Icon className="w-3.5 h-3.5" />
              </button>
            );
          })}

          <div className="w-px h-4 bg-gray-700 mx-1" />

          {/* Theme Toggle */}
          <button
            onClick={() => setThemeMode(prev => prev === 'light' ? 'dark' : 'light')}
            className="p-1.5 rounded text-gray-400 hover:text-white hover:bg-gray-700 transition-colors"
            title={`Switch to ${themeMode === 'light' ? 'dark' : 'light'} mode`}
          >
            {themeMode === 'light' ? <Moon className="w-3.5 h-3.5" /> : <Sun className="w-3.5 h-3.5" />}
          </button>

          <div className="w-px h-4 bg-gray-700 mx-1" />

          {/* Zoom Controls */}
          <button
            onClick={() => handleZoom(-25)}
            className="p-1.5 rounded text-gray-400 hover:text-white hover:bg-gray-700 transition-colors"
            title="Zoom out"
          >
            <ZoomOut className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={resetZoom}
            className="px-1.5 py-1 text-xs text-gray-400 hover:text-white rounded hover:bg-gray-700 transition-colors"
            title="Reset zoom"
          >
            {zoom}%
          </button>
          <button
            onClick={() => handleZoom(25)}
            className="p-1.5 rounded text-gray-400 hover:text-white hover:bg-gray-700 transition-colors"
            title="Zoom in"
          >
            <ZoomIn className="w-3.5 h-3.5" />
          </button>

          <div className="w-px h-4 bg-gray-700 mx-1" />

          {/* Refresh */}
          <button
            onClick={() => setLastUpdate(new Date())}
            className="p-1.5 rounded text-gray-400 hover:text-white hover:bg-gray-700 transition-colors"
            title="Refresh preview"
          >
            <RefreshCw className="w-3.5 h-3.5" />
          </button>

          {/* Fullscreen Toggle */}
          <button
            onClick={() => setIsFullscreen(!isFullscreen)}
            className="p-1.5 rounded text-gray-400 hover:text-white hover:bg-gray-700 transition-colors"
            title={isFullscreen ? 'Exit fullscreen' : 'Fullscreen'}
          >
            {isFullscreen ? <Minimize2 className="w-3.5 h-3.5" /> : <Maximize2 className="w-3.5 h-3.5" />}
          </button>
        </div>
      </div>

      {/* Device Info Bar */}
      <div className="flex items-center justify-center gap-3 px-3 py-1 bg-gray-850 border-b border-gray-700 text-xs" style={{ backgroundColor: '#1e1e2e' }}>
        <div className="flex items-center gap-1.5 text-gray-400">
          <DeviceIcon className="w-3 h-3" />
          <span>{device.name}</span>
        </div>
        <span className="text-gray-600">|</span>
        <span className="text-gray-400">
          {deviceMode === 'desktop' ? 'Responsive' : `${device.width} x ${device.height}`}
        </span>
        <span className="text-gray-600">|</span>
        <span className={`${themeMode === 'dark' ? 'text-purple-400' : 'text-yellow-400'}`}>
          {themeMode === 'dark' ? 'Dark' : 'Light'}
        </span>
      </div>

      {/* Preview Container */}
      <div className="flex-1 overflow-auto bg-gray-950 p-4 flex items-start justify-center">
        {error ? (
          <div className="flex flex-col items-center justify-center h-full text-center p-8">
            <div className="w-12 h-12 rounded-full bg-red-500/20 flex items-center justify-center mb-3">
              <RotateCcw className="w-6 h-6 text-red-400" />
            </div>
            <h3 className="text-sm font-medium text-white mb-2">Preview Error</h3>
            <p className="text-gray-400 text-xs mb-3">{error}</p>
            <button
              onClick={() => setLastUpdate(new Date())}
              className="px-3 py-1.5 text-xs bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
            >
              Retry
            </button>
          </div>
        ) : (
          <div
            className={`bg-white shadow-xl rounded-lg overflow-hidden transition-all duration-300 ${
              deviceMode === 'desktop' ? 'w-full h-full' : ''
            }`}
            style={{
              width: deviceMode === 'desktop' ? '100%' : device.width * (zoom / 100),
              height: deviceMode === 'desktop' ? '100%' : device.height * (zoom / 100),
              transform: deviceMode === 'desktop' ? `scale(${zoom / 100})` : 'none',
              transformOrigin: 'top left',
              maxWidth: deviceMode === 'desktop' ? `${100 / (zoom / 100)}%` : 'none',
              maxHeight: deviceMode === 'desktop' ? `${100 / (zoom / 100)}%` : 'none'
            }}
          >
            <iframe
              ref={iframeRef}
              title={`${language.toUpperCase()} Preview`}
              className="w-full h-full border-0"
              sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
            />
          </div>
        )}
      </div>

      {/* Status Bar */}
      <div className="flex items-center justify-between px-3 py-1 bg-gray-800 border-t border-gray-700 text-xs">
        <div className="flex items-center gap-2 text-gray-400">
          <span className="flex items-center gap-1">
            <span className="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse" />
            Live
          </span>
          <span className="text-gray-600">|</span>
          <span>{fileName}</span>
          {/* Data Stats */}
          {enableLiveData && themeData && (
            <>
              <span className="text-gray-600">|</span>
              <span className="text-gray-500">
                {themeData.posts?.length || 0} posts,{' '}
                {themeData.categories?.length || 0} categories,{' '}
                {(themeData as any).products?.length || 0} products
              </span>
            </>
          )}
        </div>
        <div className="flex items-center gap-2 text-gray-500">
          {enableLiveData && (
            <span className="text-blue-400">Data cached (2min TTL)</span>
          )}
          <span className="text-gray-600">|</span>
          <span>Auto-refresh enabled</span>
        </div>
      </div>
    </motion.div>
  );
};

export default HtmlCssPreview;
