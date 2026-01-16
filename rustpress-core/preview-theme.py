#!/usr/bin/env python3
"""
Simple theme preview server for RustPress themes.
This allows previewing the theme templates without running the full RustPress server.
"""

import http.server
import socketserver
import os
import sys
from pathlib import Path

# Configuration
PORT = 8888
THEME_DIR = Path(__file__).parent / "themes" / "rustpress-enterprise"
TEMPLATES_DIR = THEME_DIR / "templates"

class ThemeHandler(http.server.SimpleHTTPRequestHandler):
    """Custom handler for serving theme templates."""

    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(THEME_DIR), **kwargs)

    def do_GET(self):
        # Map paths to templates
        path_map = {
            '/': 'home.html',
            '/home': 'home.html',
            '/features': 'features.html',
            '/pricing': 'pricing.html',
            '/about': 'about.html',
            '/contact': 'contact.html',
            '/blog': 'blog.html',
            '/post': 'post.html',
            '/team': 'team.html',
            '/integrations': 'integrations.html',
            '/use-cases': 'use-cases.html',
            '/customers': 'customers.html',
            '/security': 'security.html',
            '/enterprise': 'enterprise.html',
            '/api': 'api.html',
            '/docs': 'docs.html',
            '/demo': 'demo.html',
            '/changelog': 'changelog.html',
            '/careers': 'careers.html',
            '/privacy': 'privacy.html',
            '/terms': 'terms.html',
            '/404': '404.html',
            '/500': '500.html',
        }

        # Clean path
        clean_path = self.path.split('?')[0].rstrip('/')
        if not clean_path:
            clean_path = '/'

        # Check if it's a template route
        if clean_path in path_map:
            template_file = TEMPLATES_DIR / path_map[clean_path]
            if template_file.exists():
                self.send_response(200)
                self.send_header('Content-type', 'text/html; charset=utf-8')
                self.end_headers()

                content = template_file.read_text(encoding='utf-8')

                # Basic Tera/Jinja template processing
                # Replace common template tags with empty strings for preview
                import re
                content = re.sub(r'\{\%.*?\%\}', '', content)
                content = re.sub(r'\{\{.*?\}\}', '', content)

                self.wfile.write(content.encode('utf-8'))
                return

        # Fall back to normal file serving
        super().do_GET()

    def log_message(self, format, *args):
        print(f"[{self.address_string()}] {format % args}")

def main():
    print(f"\n{'='*60}")
    print("RustPress Theme Preview Server")
    print(f"{'='*60}")
    print(f"\nServing theme from: {THEME_DIR}")
    print(f"Templates directory: {TEMPLATES_DIR}")
    print(f"\nServer running at: http://localhost:{PORT}")
    print("\nAvailable pages:")
    pages = ['/', '/features', '/pricing', '/about', '/contact', '/blog',
             '/team', '/integrations', '/use-cases', '/customers', '/security',
             '/enterprise', '/api', '/docs', '/demo', '/changelog', '/careers',
             '/privacy', '/terms']
    for page in pages:
        print(f"  - http://localhost:{PORT}{page}")
    print(f"\nPress Ctrl+C to stop the server.\n")

    with socketserver.TCPServer(("", PORT), ThemeHandler) as httpd:
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\nServer stopped.")

if __name__ == "__main__":
    main()
