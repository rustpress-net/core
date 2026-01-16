/**
 * ThemeDataService - Provides live-like data context for theme previews
 * Injects realistic mock data that mimics production environment
 */

// ============================================
// TYPES
// ============================================

export interface SiteConfig {
  name: string;
  tagline: string;
  description: string;
  url: string;
  logo: string;
  favicon: string;
  language: string;
  timezone: string;
  dateFormat: string;
  copyright: string;
  social: {
    twitter?: string;
    facebook?: string;
    instagram?: string;
    linkedin?: string;
    github?: string;
    youtube?: string;
  };
  analytics?: {
    googleAnalyticsId?: string;
    plausibleDomain?: string;
  };
  features: {
    comments: boolean;
    search: boolean;
    newsletter: boolean;
    darkMode: boolean;
    multiLanguage: boolean;
  };
}

export interface Author {
  id: string;
  name: string;
  slug: string;
  email: string;
  bio: string;
  avatar: string;
  role: 'admin' | 'editor' | 'author' | 'contributor';
  social?: {
    twitter?: string;
    website?: string;
  };
}

export interface Category {
  id: string;
  name: string;
  slug: string;
  description: string;
  count: number;
  parent?: string;
  image?: string;
}

export interface Tag {
  id: string;
  name: string;
  slug: string;
  count: number;
}

export interface Post {
  id: string;
  title: string;
  slug: string;
  excerpt: string;
  content: string;
  html: string;
  featuredImage: string;
  author: Author;
  category: Category;
  tags: Tag[];
  publishedAt: string;
  updatedAt: string;
  readingTime: number;
  views: number;
  likes: number;
  commentsCount: number;
  status: 'published' | 'draft' | 'scheduled';
  featured: boolean;
  meta: {
    title?: string;
    description?: string;
    keywords?: string[];
  };
}

export interface Page {
  id: string;
  title: string;
  slug: string;
  content: string;
  html: string;
  template: string;
  parent?: string;
  order: number;
  meta: {
    title?: string;
    description?: string;
  };
}

export interface MenuItem {
  id: string;
  label: string;
  url: string;
  target?: '_blank' | '_self';
  icon?: string;
  children?: MenuItem[];
  isActive?: boolean;
}

export interface Menu {
  id: string;
  name: string;
  location: 'header' | 'footer' | 'sidebar' | 'mobile';
  items: MenuItem[];
}

export interface Widget {
  id: string;
  type: 'recent-posts' | 'categories' | 'tags' | 'search' | 'newsletter' | 'social' | 'custom';
  title: string;
  content?: string;
  settings?: Record<string, unknown>;
}

export interface Sidebar {
  id: string;
  name: string;
  widgets: Widget[];
}

export interface Comment {
  id: string;
  postId: string;
  author: {
    name: string;
    email: string;
    avatar: string;
    website?: string;
  };
  content: string;
  createdAt: string;
  likes: number;
  replies?: Comment[];
  status: 'approved' | 'pending' | 'spam';
}

export interface ThemeDataContext {
  site: SiteConfig;
  currentUser: Author | null;
  authors: Author[];
  categories: Category[];
  tags: Tag[];
  posts: Post[];
  recentPosts: Post[];
  featuredPosts: Post[];
  popularPosts: Post[];
  pages: Page[];
  menus: Menu[];
  sidebars: Sidebar[];
  currentPage?: {
    type: 'home' | 'post' | 'page' | 'category' | 'tag' | 'author' | 'search' | 'archive';
    data?: Post | Page | Category | Tag | Author;
  };
  pagination?: {
    currentPage: number;
    totalPages: number;
    totalItems: number;
    perPage: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
  breadcrumbs: Array<{ label: string; url: string }>;
  comments?: Comment[];
}

// ============================================
// MOCK DATA GENERATORS
// ============================================

const generateId = () => Math.random().toString(36).substring(2, 11);

const loremIpsum = `Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.`;

const sampleContent = `
<p>Welcome to this article! Today we'll explore the fascinating world of web development and how modern frameworks are changing the way we build applications.</p>

<h2>Getting Started</h2>
<p>The first step in any development journey is understanding the fundamentals. Let's dive into what makes modern web applications tick.</p>

<blockquote>
  <p>"The best way to predict the future is to create it." - Peter Drucker</p>
</blockquote>

<h3>Key Concepts</h3>
<ul>
  <li>Component-based architecture</li>
  <li>State management</li>
  <li>Server-side rendering</li>
  <li>API integration</li>
</ul>

<p>Understanding these concepts will give you a solid foundation for building scalable applications.</p>

<h2>Code Example</h2>
<pre><code class="language-javascript">
function greet(name) {
  return \`Hello, \${name}!\`;
}

console.log(greet('World'));
</code></pre>

<p>This simple example demonstrates the power of modern JavaScript syntax.</p>

<h2>Conclusion</h2>
<p>We've only scratched the surface of what's possible with modern web development. Stay tuned for more in-depth articles!</p>
`;

// ============================================
// DEFAULT MOCK DATA
// ============================================

export const defaultSiteConfig: SiteConfig = {
  name: 'RustPress Demo',
  tagline: 'Build blazingly fast websites',
  description: 'A modern content management system built with Rust and React',
  url: 'https://demo.rustpress.dev',
  logo: '/assets/logo.svg',
  favicon: '/assets/favicon.ico',
  language: 'en',
  timezone: 'UTC',
  dateFormat: 'MMMM D, YYYY',
  copyright: '© 2024 RustPress. All rights reserved.',
  social: {
    twitter: 'rustpress',
    github: 'rustpress',
    linkedin: 'rustpress',
    youtube: '@rustpress',
  },
  analytics: {
    googleAnalyticsId: 'G-XXXXXXXXXX',
  },
  features: {
    comments: true,
    search: true,
    newsletter: true,
    darkMode: true,
    multiLanguage: false,
  },
};

export const defaultAuthors: Author[] = [
  {
    id: 'author-1',
    name: 'Sarah Johnson',
    slug: 'sarah-johnson',
    email: 'sarah@rustpress.dev',
    bio: 'Senior developer and tech writer. Passionate about building great software and sharing knowledge.',
    avatar: 'https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=200&h=200&fit=crop',
    role: 'admin',
    social: {
      twitter: 'sarahjdev',
      website: 'https://sarahjohnson.dev',
    },
  },
  {
    id: 'author-2',
    name: 'Mike Chen',
    slug: 'mike-chen',
    email: 'mike@rustpress.dev',
    bio: 'Full-stack developer specializing in Rust and TypeScript. Open source contributor.',
    avatar: 'https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=200&h=200&fit=crop',
    role: 'editor',
    social: {
      twitter: 'mikechendev',
      website: 'https://mikechen.io',
    },
  },
  {
    id: 'author-3',
    name: 'Emily Davis',
    slug: 'emily-davis',
    email: 'emily@rustpress.dev',
    bio: 'UX designer turned developer. Creating beautiful and functional web experiences.',
    avatar: 'https://images.unsplash.com/photo-1438761681033-6461ffad8d80?w=200&h=200&fit=crop',
    role: 'author',
  },
];

export const defaultCategories: Category[] = [
  {
    id: 'cat-1',
    name: 'Technology',
    slug: 'technology',
    description: 'Latest tech news, tutorials, and insights',
    count: 24,
    image: 'https://images.unsplash.com/photo-1518770660439-4636190af475?w=400&h=300&fit=crop',
  },
  {
    id: 'cat-2',
    name: 'Design',
    slug: 'design',
    description: 'UI/UX design principles and best practices',
    count: 18,
    image: 'https://images.unsplash.com/photo-1561070791-2526d30994b5?w=400&h=300&fit=crop',
  },
  {
    id: 'cat-3',
    name: 'Development',
    slug: 'development',
    description: 'Programming tutorials and development guides',
    count: 32,
    image: 'https://images.unsplash.com/photo-1461749280684-dccba630e2f6?w=400&h=300&fit=crop',
  },
  {
    id: 'cat-4',
    name: 'Business',
    slug: 'business',
    description: 'Business strategies and entrepreneurship',
    count: 15,
    image: 'https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=400&h=300&fit=crop',
  },
  {
    id: 'cat-5',
    name: 'Tutorials',
    slug: 'tutorials',
    description: 'Step-by-step guides and how-tos',
    count: 28,
    image: 'https://images.unsplash.com/photo-1516321318423-f06f85e504b3?w=400&h=300&fit=crop',
  },
];

export const defaultTags: Tag[] = [
  { id: 'tag-1', name: 'Rust', slug: 'rust', count: 15 },
  { id: 'tag-2', name: 'React', slug: 'react', count: 22 },
  { id: 'tag-3', name: 'TypeScript', slug: 'typescript', count: 18 },
  { id: 'tag-4', name: 'Web Development', slug: 'web-development', count: 30 },
  { id: 'tag-5', name: 'CSS', slug: 'css', count: 12 },
  { id: 'tag-6', name: 'JavaScript', slug: 'javascript', count: 25 },
  { id: 'tag-7', name: 'Performance', slug: 'performance', count: 8 },
  { id: 'tag-8', name: 'SEO', slug: 'seo', count: 10 },
  { id: 'tag-9', name: 'API', slug: 'api', count: 14 },
  { id: 'tag-10', name: 'Database', slug: 'database', count: 11 },
];

export const generateMockPosts = (count: number = 10): Post[] => {
  const titles = [
    'Getting Started with Rust for Web Development',
    'Building Blazingly Fast APIs with Actix-web',
    'Modern CSS Techniques Every Developer Should Know',
    'The Complete Guide to React Server Components',
    'TypeScript Best Practices for Large Applications',
    'Optimizing Database Performance in Production',
    'Creating Beautiful UI Components with Tailwind CSS',
    'Understanding WebAssembly and Its Applications',
    'Mastering Git Workflows for Team Collaboration',
    'Building Real-time Applications with WebSockets',
    'Introduction to Microservices Architecture',
    'Securing Your Web Applications: A Complete Guide',
    'The Future of JavaScript: ES2024 and Beyond',
    'Building Progressive Web Apps from Scratch',
    'Docker and Kubernetes for Developers',
  ];

  const images = [
    'https://images.unsplash.com/photo-1555066931-4365d14bab8c?w=800&h=450&fit=crop',
    'https://images.unsplash.com/photo-1461749280684-dccba630e2f6?w=800&h=450&fit=crop',
    'https://images.unsplash.com/photo-1517694712202-14dd9538aa97?w=800&h=450&fit=crop',
    'https://images.unsplash.com/photo-1504639725590-34d0984388bd?w=800&h=450&fit=crop',
    'https://images.unsplash.com/photo-1516321318423-f06f85e504b3?w=800&h=450&fit=crop',
    'https://images.unsplash.com/photo-1518770660439-4636190af475?w=800&h=450&fit=crop',
    'https://images.unsplash.com/photo-1550439062-609e1531270e?w=800&h=450&fit=crop',
    'https://images.unsplash.com/photo-1537432376149-e84978e51575?w=800&h=450&fit=crop',
  ];

  return Array.from({ length: count }, (_, i) => {
    const title = titles[i % titles.length];
    const author = defaultAuthors[i % defaultAuthors.length];
    const category = defaultCategories[i % defaultCategories.length];
    const publishDate = new Date();
    publishDate.setDate(publishDate.getDate() - i * 3);

    return {
      id: `post-${i + 1}`,
      title,
      slug: title.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, ''),
      excerpt: loremIpsum.substring(0, 150) + '...',
      content: sampleContent,
      html: sampleContent,
      featuredImage: images[i % images.length],
      author,
      category,
      tags: defaultTags.slice(0, 3 + (i % 3)),
      publishedAt: publishDate.toISOString(),
      updatedAt: publishDate.toISOString(),
      readingTime: 5 + (i % 10),
      views: 100 + Math.floor(Math.random() * 5000),
      likes: 10 + Math.floor(Math.random() * 500),
      commentsCount: Math.floor(Math.random() * 50),
      status: 'published',
      featured: i < 3,
      meta: {
        title: title,
        description: loremIpsum.substring(0, 160),
        keywords: defaultTags.slice(0, 5).map(t => t.name),
      },
    };
  });
};

export const defaultPages: Page[] = [
  {
    id: 'page-1',
    title: 'About Us',
    slug: 'about',
    content: '<p>Learn more about our team and mission...</p>',
    html: '<p>Learn more about our team and mission...</p>',
    template: 'default',
    order: 1,
    meta: {
      title: 'About Us - RustPress',
      description: 'Learn more about RustPress and our mission',
    },
  },
  {
    id: 'page-2',
    title: 'Contact',
    slug: 'contact',
    content: '<p>Get in touch with us...</p>',
    html: '<p>Get in touch with us...</p>',
    template: 'contact',
    order: 2,
    meta: {
      title: 'Contact Us - RustPress',
      description: 'Contact the RustPress team',
    },
  },
  {
    id: 'page-3',
    title: 'Privacy Policy',
    slug: 'privacy',
    content: '<p>Our privacy policy...</p>',
    html: '<p>Our privacy policy...</p>',
    template: 'legal',
    order: 3,
    meta: {},
  },
  {
    id: 'page-4',
    title: 'Terms of Service',
    slug: 'terms',
    content: '<p>Our terms of service...</p>',
    html: '<p>Our terms of service...</p>',
    template: 'legal',
    order: 4,
    meta: {},
  },
];

export const defaultMenus: Menu[] = [
  {
    id: 'menu-header',
    name: 'Main Navigation',
    location: 'header',
    items: [
      { id: 'nav-1', label: 'Home', url: '/', isActive: true },
      { id: 'nav-2', label: 'Blog', url: '/blog' },
      {
        id: 'nav-3',
        label: 'Categories',
        url: '/categories',
        children: defaultCategories.map(cat => ({
          id: `nav-cat-${cat.id}`,
          label: cat.name,
          url: `/category/${cat.slug}`,
        })),
      },
      { id: 'nav-4', label: 'About', url: '/about' },
      { id: 'nav-5', label: 'Contact', url: '/contact' },
    ],
  },
  {
    id: 'menu-footer',
    name: 'Footer Navigation',
    location: 'footer',
    items: [
      { id: 'footer-1', label: 'About', url: '/about' },
      { id: 'footer-2', label: 'Privacy Policy', url: '/privacy' },
      { id: 'footer-3', label: 'Terms of Service', url: '/terms' },
      { id: 'footer-4', label: 'Contact', url: '/contact' },
    ],
  },
  {
    id: 'menu-mobile',
    name: 'Mobile Navigation',
    location: 'mobile',
    items: [
      { id: 'mobile-1', label: 'Home', url: '/' },
      { id: 'mobile-2', label: 'Blog', url: '/blog' },
      { id: 'mobile-3', label: 'Categories', url: '/categories' },
      { id: 'mobile-4', label: 'About', url: '/about' },
      { id: 'mobile-5', label: 'Contact', url: '/contact' },
    ],
  },
];

export const defaultSidebars: Sidebar[] = [
  {
    id: 'sidebar-main',
    name: 'Main Sidebar',
    widgets: [
      {
        id: 'widget-search',
        type: 'search',
        title: 'Search',
      },
      {
        id: 'widget-recent',
        type: 'recent-posts',
        title: 'Recent Posts',
        settings: { count: 5 },
      },
      {
        id: 'widget-categories',
        type: 'categories',
        title: 'Categories',
      },
      {
        id: 'widget-tags',
        type: 'tags',
        title: 'Popular Tags',
        settings: { count: 10 },
      },
      {
        id: 'widget-newsletter',
        type: 'newsletter',
        title: 'Subscribe',
        content: 'Get the latest posts delivered to your inbox.',
      },
      {
        id: 'widget-social',
        type: 'social',
        title: 'Follow Us',
      },
    ],
  },
];

export const generateMockComments = (postId: string, count: number = 5): Comment[] => {
  const names = ['John Doe', 'Jane Smith', 'Alex Turner', 'Maria Garcia', 'David Kim'];
  const contents = [
    'Great article! This really helped me understand the concept better.',
    'Thanks for sharing this. I have been looking for a clear explanation.',
    'Very well written. Looking forward to more content like this!',
    'This is exactly what I needed. Bookmarked for future reference.',
    'Excellent post! Could you elaborate more on the performance aspects?',
  ];

  return Array.from({ length: count }, (_, i) => {
    const date = new Date();
    date.setHours(date.getHours() - i * 24);

    return {
      id: `comment-${postId}-${i + 1}`,
      postId,
      author: {
        name: names[i % names.length],
        email: `user${i + 1}@example.com`,
        avatar: `https://i.pravatar.cc/100?u=${i}`,
      },
      content: contents[i % contents.length],
      createdAt: date.toISOString(),
      likes: Math.floor(Math.random() * 20),
      status: 'approved',
      replies: i === 0 ? [
        {
          id: `comment-${postId}-reply-1`,
          postId,
          author: {
            name: defaultAuthors[0].name,
            email: defaultAuthors[0].email,
            avatar: defaultAuthors[0].avatar,
          },
          content: 'Thank you for the kind words! Happy to help.',
          createdAt: new Date().toISOString(),
          likes: 5,
          status: 'approved',
        },
      ] : undefined,
    };
  });
};

// ============================================
// MAIN SERVICE
// ============================================

export class ThemeDataService {
  private data: ThemeDataContext;

  constructor(customData?: Partial<ThemeDataContext>) {
    const posts = generateMockPosts(15);

    this.data = {
      site: defaultSiteConfig,
      currentUser: defaultAuthors[0],
      authors: defaultAuthors,
      categories: defaultCategories,
      tags: defaultTags,
      posts,
      recentPosts: posts.slice(0, 5),
      featuredPosts: posts.filter(p => p.featured),
      popularPosts: [...posts].sort((a, b) => b.views - a.views).slice(0, 5),
      pages: defaultPages,
      menus: defaultMenus,
      sidebars: defaultSidebars,
      currentPage: {
        type: 'home',
      },
      pagination: {
        currentPage: 1,
        totalPages: 5,
        totalItems: posts.length,
        perPage: 10,
        hasNext: true,
        hasPrev: false,
      },
      breadcrumbs: [
        { label: 'Home', url: '/' },
      ],
      ...customData,
    };
  }

  getContext(): ThemeDataContext {
    return this.data;
  }

  getPost(slug: string): Post | undefined {
    return this.data.posts.find(p => p.slug === slug);
  }

  getPage(slug: string): Page | undefined {
    return this.data.pages.find(p => p.slug === slug);
  }

  getCategory(slug: string): Category | undefined {
    return this.data.categories.find(c => c.slug === slug);
  }

  getTag(slug: string): Tag | undefined {
    return this.data.tags.find(t => t.slug === slug);
  }

  getAuthor(slug: string): Author | undefined {
    return this.data.authors.find(a => a.slug === slug);
  }

  getMenu(location: string): Menu | undefined {
    return this.data.menus.find(m => m.location === location);
  }

  getSidebar(id: string): Sidebar | undefined {
    return this.data.sidebars.find(s => s.id === id);
  }

  getPostsByCategory(categorySlug: string): Post[] {
    return this.data.posts.filter(p => p.category.slug === categorySlug);
  }

  getPostsByTag(tagSlug: string): Post[] {
    return this.data.posts.filter(p => p.tags.some(t => t.slug === tagSlug));
  }

  getPostsByAuthor(authorSlug: string): Post[] {
    return this.data.posts.filter(p => p.author.slug === authorSlug);
  }

  updateSiteConfig(config: Partial<SiteConfig>): void {
    this.data.site = { ...this.data.site, ...config };
  }

  setCurrentPage(type: ThemeDataContext['currentPage']['type'], data?: Post | Page | Category | Tag | Author): void {
    this.data.currentPage = { type, data };

    // Update breadcrumbs based on page type
    switch (type) {
      case 'post':
        const post = data as Post;
        this.data.breadcrumbs = [
          { label: 'Home', url: '/' },
          { label: 'Blog', url: '/blog' },
          { label: post.category.name, url: `/category/${post.category.slug}` },
          { label: post.title, url: `/post/${post.slug}` },
        ];
        this.data.comments = generateMockComments(post.id);
        break;
      case 'category':
        const cat = data as Category;
        this.data.breadcrumbs = [
          { label: 'Home', url: '/' },
          { label: 'Categories', url: '/categories' },
          { label: cat.name, url: `/category/${cat.slug}` },
        ];
        break;
      case 'tag':
        const tag = data as Tag;
        this.data.breadcrumbs = [
          { label: 'Home', url: '/' },
          { label: 'Tags', url: '/tags' },
          { label: tag.name, url: `/tag/${tag.slug}` },
        ];
        break;
      case 'author':
        const author = data as Author;
        this.data.breadcrumbs = [
          { label: 'Home', url: '/' },
          { label: 'Authors', url: '/authors' },
          { label: author.name, url: `/author/${author.slug}` },
        ];
        break;
      case 'page':
        const page = data as Page;
        this.data.breadcrumbs = [
          { label: 'Home', url: '/' },
          { label: page.title, url: `/${page.slug}` },
        ];
        break;
      default:
        this.data.breadcrumbs = [{ label: 'Home', url: '/' }];
    }
  }
}

// ============================================
// TEMPLATE ENGINE
// ============================================

/**
 * Process template variables in HTML content
 * Supports multiple template syntaxes:
 * - Handlebars: {{variable}}, {{#each}}, {{#if}}
 * - Liquid: {{ variable }}, {% for %}, {% if %}
 * - Simple: ${variable}
 */
export function processTemplateVariables(
  html: string,
  context: ThemeDataContext
): string {
  let processed = html;

  // Helper to safely get nested properties
  const getNestedValue = (obj: any, path: string): any => {
    return path.split('.').reduce((current, key) => {
      return current && current[key] !== undefined ? current[key] : undefined;
    }, obj);
  };

  // Process simple variable replacements: {{variable}} or {{ variable }}
  processed = processed.replace(/\{\{\s*([^#/}]+?)\s*\}\}/g, (match, path) => {
    const value = getNestedValue(context, path.trim());
    if (value === undefined) return match;
    if (typeof value === 'object') return JSON.stringify(value);
    return String(value);
  });

  // Process ${variable} syntax
  processed = processed.replace(/\$\{([^}]+)\}/g, (match, path) => {
    const value = getNestedValue(context, path.trim());
    if (value === undefined) return match;
    if (typeof value === 'object') return JSON.stringify(value);
    return String(value);
  });

  // Process Liquid-style variables: {{ variable }}
  processed = processed.replace(/\{\{\s*([a-zA-Z_][a-zA-Z0-9_.]*)\s*\}\}/g, (match, path) => {
    const value = getNestedValue(context, path.trim());
    if (value === undefined) return match;
    if (typeof value === 'object') return JSON.stringify(value);
    return String(value);
  });

  return processed;
}

/**
 * Generate JavaScript code to inject context into the preview
 */
export function generateContextScript(context: ThemeDataContext): string {
  return `
<script>
  // RustPress Theme Data Context - Injected for preview
  window.RustPress = ${JSON.stringify(context, null, 2)};

  // Shorthand aliases
  window.site = window.RustPress.site;
  window.posts = window.RustPress.posts;
  window.pages = window.RustPress.pages;
  window.categories = window.RustPress.categories;
  window.tags = window.RustPress.tags;
  window.authors = window.RustPress.authors;
  window.menus = window.RustPress.menus;
  window.currentUser = window.RustPress.currentUser;
  window.currentPage = window.RustPress.currentPage;
  window.pagination = window.RustPress.pagination;
  window.breadcrumbs = window.RustPress.breadcrumbs;
  window.sidebars = window.RustPress.sidebars;
  window.recentPosts = window.RustPress.recentPosts;
  window.featuredPosts = window.RustPress.featuredPosts;
  window.popularPosts = window.RustPress.popularPosts;
  window.comments = window.RustPress.comments;

  // Helper functions
  window.RustPress.helpers = {
    formatDate: (date, format) => {
      const d = new Date(date);
      const options = { year: 'numeric', month: 'long', day: 'numeric' };
      return d.toLocaleDateString('en-US', options);
    },
    truncate: (str, length = 100) => {
      if (str.length <= length) return str;
      return str.substring(0, length) + '...';
    },
    slugify: (str) => {
      return str.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '');
    },
    getPostUrl: (post) => '/post/' + post.slug,
    getCategoryUrl: (cat) => '/category/' + cat.slug,
    getTagUrl: (tag) => '/tag/' + tag.slug,
    getAuthorUrl: (author) => '/author/' + author.slug,
    getMenu: (location) => window.menus.find(m => m.location === location),
    getSidebar: (id) => window.sidebars.find(s => s.id === id),
  };

  // Console log for debugging
  console.log('[RustPress Preview] Theme data context loaded:', window.RustPress);
</script>
`;
}

// ============================================
// CACHED DATA FETCHER
// ============================================

interface CacheEntry<T> {
  data: T;
  timestamp: number;
  ttl: number; // Time to live in ms
}

class ThemeDataCache {
  private cache: Map<string, CacheEntry<any>> = new Map();
  private fetchPromises: Map<string, Promise<any>> = new Map();
  private static instance: ThemeDataCache;

  static getInstance(): ThemeDataCache {
    if (!ThemeDataCache.instance) {
      ThemeDataCache.instance = new ThemeDataCache();
    }
    return ThemeDataCache.instance;
  }

  private isExpired(entry: CacheEntry<any>): boolean {
    return Date.now() - entry.timestamp > entry.ttl;
  }

  get<T>(key: string): T | null {
    const entry = this.cache.get(key);
    if (!entry || this.isExpired(entry)) {
      return null;
    }
    return entry.data as T;
  }

  set<T>(key: string, data: T, ttl: number = 5 * 60 * 1000): void {
    this.cache.set(key, {
      data,
      timestamp: Date.now(),
      ttl,
    });
  }

  invalidate(key: string): void {
    this.cache.delete(key);
    this.fetchPromises.delete(key);
  }

  invalidateAll(): void {
    this.cache.clear();
    this.fetchPromises.clear();
  }

  // Deduplicate concurrent requests
  async fetchWithDedup<T>(
    key: string,
    fetcher: () => Promise<T>,
    ttl?: number
  ): Promise<T> {
    // Check cache first
    const cached = this.get<T>(key);
    if (cached !== null) {
      console.log(`[ThemeDataCache] Cache hit for: ${key}`);
      return cached;
    }

    // Check if there's already a pending fetch
    const pendingFetch = this.fetchPromises.get(key);
    if (pendingFetch) {
      console.log(`[ThemeDataCache] Reusing pending fetch for: ${key}`);
      return pendingFetch;
    }

    // Create new fetch promise
    console.log(`[ThemeDataCache] Fetching from backend: ${key}`);
    const fetchPromise = fetcher()
      .then((data) => {
        this.set(key, data, ttl);
        this.fetchPromises.delete(key);
        return data;
      })
      .catch((error) => {
        this.fetchPromises.delete(key);
        throw error;
      });

    this.fetchPromises.set(key, fetchPromise);
    return fetchPromise;
  }
}

// ============================================
// REAL-TIME DATA SERVICE (with backend integration)
// ============================================

const API_BASE = '/api';

export interface RealTimeThemeData {
  site: SiteConfig;
  posts: Post[];
  pages: Page[];
  categories: Category[];
  tags: Tag[];
  authors: Author[];
  menus: Menu[];
  products?: any[];
  users?: any[];
  media?: any[];
}

class RealTimeThemeDataService {
  private cache = ThemeDataCache.getInstance();
  private mockService = new ThemeDataService();
  private isBackendAvailable = false;
  private lastCheckTime = 0;
  private checkInterval = 30000; // Check backend availability every 30s

  // Cache TTLs (in milliseconds)
  private readonly CACHE_TTL = {
    site: 10 * 60 * 1000,      // 10 minutes - site config rarely changes
    posts: 2 * 60 * 1000,      // 2 minutes - posts change more frequently
    pages: 5 * 60 * 1000,      // 5 minutes
    categories: 10 * 60 * 1000, // 10 minutes
    tags: 10 * 60 * 1000,      // 10 minutes
    authors: 10 * 60 * 1000,   // 10 minutes
    menus: 10 * 60 * 1000,     // 10 minutes
    products: 2 * 60 * 1000,   // 2 minutes
    media: 5 * 60 * 1000,      // 5 minutes
    all: 2 * 60 * 1000,        // 2 minutes for full context
  };

  private async checkBackendAvailability(): Promise<boolean> {
    if (Date.now() - this.lastCheckTime < this.checkInterval && this.lastCheckTime > 0) {
      return this.isBackendAvailable;
    }

    try {
      const response = await fetch(`${API_BASE}/health`, {
        method: 'GET',
        signal: AbortSignal.timeout(3000),
      });
      this.isBackendAvailable = response.ok;
    } catch {
      this.isBackendAvailable = false;
    }

    this.lastCheckTime = Date.now();
    return this.isBackendAvailable;
  }

  private async fetchFromBackend<T>(endpoint: string): Promise<T> {
    const response = await fetch(`${API_BASE}${endpoint}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
      },
      credentials: 'include',
    });

    if (!response.ok) {
      throw new Error(`API error: ${response.status} ${response.statusText}`);
    }

    return response.json();
  }

  /**
   * Fetch ALL theme data in a single optimized API call
   * This is the most efficient approach - one DB query that returns everything
   */
  async fetchAllThemeData(): Promise<ThemeDataContext> {
    return this.cache.fetchWithDedup(
      'theme-data-all',
      async () => {
        const backendAvailable = await this.checkBackendAvailability();

        if (backendAvailable) {
          try {
            // Single API call that returns all theme data
            const data = await this.fetchFromBackend<RealTimeThemeData>('/theme/preview-context');

            // Transform backend data to ThemeDataContext
            return this.transformToContext(data);
          } catch (error) {
            console.warn('[RealTimeThemeData] Backend fetch failed, using mock data:', error);
            return this.mockService.getContext();
          }
        }

        // Use mock data when backend is not available
        console.log('[RealTimeThemeData] Backend not available, using mock data');
        return this.mockService.getContext();
      },
      this.CACHE_TTL.all
    );
  }

  /**
   * Fetch individual data types (with caching)
   */
  async fetchPosts(): Promise<Post[]> {
    return this.cache.fetchWithDedup(
      'posts',
      async () => {
        const backendAvailable = await this.checkBackendAvailability();
        if (backendAvailable) {
          try {
            return await this.fetchFromBackend<Post[]>('/posts?limit=50&status=published');
          } catch {
            return generateMockPosts(15);
          }
        }
        return generateMockPosts(15);
      },
      this.CACHE_TTL.posts
    );
  }

  async fetchCategories(): Promise<Category[]> {
    return this.cache.fetchWithDedup(
      'categories',
      async () => {
        const backendAvailable = await this.checkBackendAvailability();
        if (backendAvailable) {
          try {
            return await this.fetchFromBackend<Category[]>('/categories');
          } catch {
            return defaultCategories;
          }
        }
        return defaultCategories;
      },
      this.CACHE_TTL.categories
    );
  }

  async fetchProducts(): Promise<any[]> {
    return this.cache.fetchWithDedup(
      'products',
      async () => {
        const backendAvailable = await this.checkBackendAvailability();
        if (backendAvailable) {
          try {
            return await this.fetchFromBackend<any[]>('/products?limit=50&status=active');
          } catch {
            return this.generateMockProducts();
          }
        }
        return this.generateMockProducts();
      },
      this.CACHE_TTL.products
    );
  }

  async fetchSiteConfig(): Promise<SiteConfig> {
    return this.cache.fetchWithDedup(
      'site-config',
      async () => {
        const backendAvailable = await this.checkBackendAvailability();
        if (backendAvailable) {
          try {
            return await this.fetchFromBackend<SiteConfig>('/settings/site');
          } catch {
            return defaultSiteConfig;
          }
        }
        return defaultSiteConfig;
      },
      this.CACHE_TTL.site
    );
  }

  async fetchMenus(): Promise<Menu[]> {
    return this.cache.fetchWithDedup(
      'menus',
      async () => {
        const backendAvailable = await this.checkBackendAvailability();
        if (backendAvailable) {
          try {
            return await this.fetchFromBackend<Menu[]>('/menus');
          } catch {
            return defaultMenus;
          }
        }
        return defaultMenus;
      },
      this.CACHE_TTL.menus
    );
  }

  /**
   * Transform backend response to ThemeDataContext
   */
  private transformToContext(data: RealTimeThemeData): ThemeDataContext {
    const posts = data.posts || generateMockPosts(15);

    return {
      site: data.site || defaultSiteConfig,
      currentUser: data.authors?.[0] || defaultAuthors[0],
      authors: data.authors || defaultAuthors,
      categories: data.categories || defaultCategories,
      tags: data.tags || defaultTags,
      posts,
      recentPosts: posts.slice(0, 5),
      featuredPosts: posts.filter(p => p.featured).slice(0, 3),
      popularPosts: [...posts].sort((a, b) => (b.views || 0) - (a.views || 0)).slice(0, 5),
      pages: data.pages || defaultPages,
      menus: data.menus || defaultMenus,
      sidebars: defaultSidebars,
      currentPage: { type: 'home' },
      pagination: {
        currentPage: 1,
        totalPages: Math.ceil(posts.length / 10),
        totalItems: posts.length,
        perPage: 10,
        hasNext: posts.length > 10,
        hasPrev: false,
      },
      breadcrumbs: [{ label: 'Home', url: '/' }],
      // Extended data
      products: data.products,
      media: data.media,
    } as ThemeDataContext & { products?: any[]; media?: any[] };
  }

  /**
   * Generate mock products for preview
   */
  private generateMockProducts(): any[] {
    return [
      {
        id: 'prod-1',
        name: 'Premium Theme Bundle',
        slug: 'premium-theme-bundle',
        price: 99.00,
        salePrice: 79.00,
        currency: 'USD',
        description: 'A collection of 5 premium themes for your website.',
        image: 'https://images.unsplash.com/photo-1555066931-4365d14bab8c?w=400&h=400&fit=crop',
        category: 'Themes',
        inStock: true,
        rating: 4.8,
        reviews: 124,
      },
      {
        id: 'prod-2',
        name: 'Developer Toolkit',
        slug: 'developer-toolkit',
        price: 149.00,
        currency: 'USD',
        description: 'Essential tools and utilities for developers.',
        image: 'https://images.unsplash.com/photo-1461749280684-dccba630e2f6?w=400&h=400&fit=crop',
        category: 'Tools',
        inStock: true,
        rating: 4.9,
        reviews: 89,
      },
      {
        id: 'prod-3',
        name: 'E-commerce Plugin',
        slug: 'ecommerce-plugin',
        price: 59.00,
        currency: 'USD',
        description: 'Add shopping cart functionality to your site.',
        image: 'https://images.unsplash.com/photo-1556742049-0cfed4f6a45d?w=400&h=400&fit=crop',
        category: 'Plugins',
        inStock: true,
        rating: 4.7,
        reviews: 256,
      },
      {
        id: 'prod-4',
        name: 'SEO Optimization Pack',
        slug: 'seo-optimization-pack',
        price: 39.00,
        salePrice: 29.00,
        currency: 'USD',
        description: 'Boost your search engine rankings.',
        image: 'https://images.unsplash.com/photo-1432888622747-4eb9a8efeb07?w=400&h=400&fit=crop',
        category: 'Marketing',
        inStock: true,
        rating: 4.6,
        reviews: 178,
      },
      {
        id: 'prod-5',
        name: 'Analytics Dashboard',
        slug: 'analytics-dashboard',
        price: 79.00,
        currency: 'USD',
        description: 'Comprehensive analytics for your website.',
        image: 'https://images.unsplash.com/photo-1551288049-bebda4e38f71?w=400&h=400&fit=crop',
        category: 'Analytics',
        inStock: false,
        rating: 4.5,
        reviews: 67,
      },
    ];
  }

  /**
   * Invalidate cache (call when data changes)
   */
  invalidateCache(key?: string): void {
    if (key) {
      this.cache.invalidate(key);
    } else {
      this.cache.invalidateAll();
    }
  }

  /**
   * Subscribe to real-time updates via WebSocket
   */
  subscribeToUpdates(onUpdate: (type: string, data: any) => void): () => void {
    const ws = new WebSocket(`${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/api/ws/theme-updates`);

    ws.onmessage = (event) => {
      try {
        const { type, data } = JSON.parse(event.data);
        // Invalidate cache for the updated data type
        this.invalidateCache(type);
        onUpdate(type, data);
      } catch (error) {
        console.error('[RealTimeThemeData] WebSocket message error:', error);
      }
    };

    ws.onerror = (error) => {
      console.warn('[RealTimeThemeData] WebSocket error:', error);
    };

    // Return unsubscribe function
    return () => {
      ws.close();
    };
  }
}

// ============================================
// EXTENDED CONTEXT SCRIPT GENERATOR
// ============================================

/**
 * Generate JavaScript code with real-time data helpers
 */
export function generateRealTimeContextScript(context: ThemeDataContext & { products?: any[] }): string {
  return `
<script>
  // RustPress Theme Data Context - Live Preview Data
  window.RustPress = ${JSON.stringify(context, null, 2)};

  // Shorthand aliases for common data
  window.site = window.RustPress.site;
  window.posts = window.RustPress.posts;
  window.pages = window.RustPress.pages;
  window.categories = window.RustPress.categories;
  window.tags = window.RustPress.tags;
  window.authors = window.RustPress.authors;
  window.menus = window.RustPress.menus;
  window.currentUser = window.RustPress.currentUser;
  window.currentPage = window.RustPress.currentPage;
  window.pagination = window.RustPress.pagination;
  window.breadcrumbs = window.RustPress.breadcrumbs;
  window.sidebars = window.RustPress.sidebars;
  window.recentPosts = window.RustPress.recentPosts;
  window.featuredPosts = window.RustPress.featuredPosts;
  window.popularPosts = window.RustPress.popularPosts;
  window.comments = window.RustPress.comments;
  window.products = window.RustPress.products || [];

  // Helper functions
  window.RustPress.helpers = {
    // Date formatting
    formatDate: (date, format = 'long') => {
      const d = new Date(date);
      if (format === 'short') {
        return d.toLocaleDateString();
      } else if (format === 'relative') {
        const now = new Date();
        const diff = now - d;
        const days = Math.floor(diff / (1000 * 60 * 60 * 24));
        if (days === 0) return 'Today';
        if (days === 1) return 'Yesterday';
        if (days < 7) return days + ' days ago';
        if (days < 30) return Math.floor(days / 7) + ' weeks ago';
        return d.toLocaleDateString();
      }
      return d.toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' });
    },

    // Text utilities
    truncate: (str, length = 100) => {
      if (!str || str.length <= length) return str;
      return str.substring(0, length).trim() + '...';
    },
    slugify: (str) => {
      return str.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '');
    },
    stripHtml: (html) => {
      const tmp = document.createElement('div');
      tmp.innerHTML = html;
      return tmp.textContent || tmp.innerText || '';
    },

    // URL helpers
    getPostUrl: (post) => '/post/' + post.slug,
    getCategoryUrl: (cat) => '/category/' + cat.slug,
    getTagUrl: (tag) => '/tag/' + tag.slug,
    getAuthorUrl: (author) => '/author/' + author.slug,
    getPageUrl: (page) => '/' + page.slug,
    getProductUrl: (product) => '/product/' + product.slug,

    // Data getters
    getMenu: (location) => window.menus.find(m => m.location === location),
    getSidebar: (id) => window.sidebars.find(s => s.id === id),
    getCategory: (slug) => window.categories.find(c => c.slug === slug),
    getTag: (slug) => window.tags.find(t => t.slug === slug),
    getAuthor: (slug) => window.authors.find(a => a.slug === slug),
    getPost: (slug) => window.posts.find(p => p.slug === slug),
    getPage: (slug) => window.pages.find(p => p.slug === slug),
    getProduct: (slug) => window.products.find(p => p.slug === slug),

    // Filtered data
    getPostsByCategory: (categorySlug) => window.posts.filter(p => p.category?.slug === categorySlug),
    getPostsByTag: (tagSlug) => window.posts.filter(p => p.tags?.some(t => t.slug === tagSlug)),
    getPostsByAuthor: (authorSlug) => window.posts.filter(p => p.author?.slug === authorSlug),
    getProductsByCategory: (category) => window.products.filter(p => p.category === category),
    getFeaturedProducts: () => window.products.filter(p => p.featured),
    getProductsOnSale: () => window.products.filter(p => p.salePrice && p.salePrice < p.price),

    // Price formatting
    formatPrice: (price, currency = 'USD') => {
      return new Intl.NumberFormat('en-US', {
        style: 'currency',
        currency: currency,
      }).format(price);
    },

    // Pagination helpers
    getPaginationRange: (current, total, delta = 2) => {
      const range = [];
      const left = current - delta;
      const right = current + delta + 1;
      let l;
      for (let i = 1; i <= total; i++) {
        if (i === 1 || i === total || (i >= left && i < right)) {
          if (l) {
            if (i - l === 2) range.push(l + 1);
            else if (i - l !== 1) range.push('...');
          }
          range.push(i);
          l = i;
        }
      }
      return range;
    },
  };

  // Template rendering helpers (for Handlebars/Liquid-like templates)
  window.RustPress.render = {
    // Render a list of posts
    posts: (posts, template) => {
      return posts.map(post => {
        let html = template;
        html = html.replace(/\\{\\{\\s*title\\s*\\}\\}/g, post.title);
        html = html.replace(/\\{\\{\\s*excerpt\\s*\\}\\}/g, post.excerpt);
        html = html.replace(/\\{\\{\\s*url\\s*\\}\\}/g, window.RustPress.helpers.getPostUrl(post));
        html = html.replace(/\\{\\{\\s*image\\s*\\}\\}/g, post.featuredImage);
        html = html.replace(/\\{\\{\\s*date\\s*\\}\\}/g, window.RustPress.helpers.formatDate(post.publishedAt));
        html = html.replace(/\\{\\{\\s*author\\.name\\s*\\}\\}/g, post.author?.name || '');
        html = html.replace(/\\{\\{\\s*author\\.avatar\\s*\\}\\}/g, post.author?.avatar || '');
        html = html.replace(/\\{\\{\\s*category\\.name\\s*\\}\\}/g, post.category?.name || '');
        html = html.replace(/\\{\\{\\s*readingTime\\s*\\}\\}/g, post.readingTime + ' min read');
        return html;
      }).join('');
    },

    // Render a menu
    menu: (location, template) => {
      const menu = window.RustPress.helpers.getMenu(location);
      if (!menu) return '';
      return menu.items.map(item => {
        let html = template;
        html = html.replace(/\\{\\{\\s*label\\s*\\}\\}/g, item.label);
        html = html.replace(/\\{\\{\\s*url\\s*\\}\\}/g, item.url);
        html = html.replace(/\\{\\{\\s*active\\s*\\}\\}/g, item.isActive ? 'active' : '');
        return html;
      }).join('');
    },

    // Render products
    products: (products, template) => {
      return products.map(product => {
        let html = template;
        html = html.replace(/\\{\\{\\s*name\\s*\\}\\}/g, product.name);
        html = html.replace(/\\{\\{\\s*description\\s*\\}\\}/g, product.description);
        html = html.replace(/\\{\\{\\s*url\\s*\\}\\}/g, window.RustPress.helpers.getProductUrl(product));
        html = html.replace(/\\{\\{\\s*image\\s*\\}\\}/g, product.image);
        html = html.replace(/\\{\\{\\s*price\\s*\\}\\}/g, window.RustPress.helpers.formatPrice(product.price));
        html = html.replace(/\\{\\{\\s*salePrice\\s*\\}\\}/g, product.salePrice ? window.RustPress.helpers.formatPrice(product.salePrice) : '');
        html = html.replace(/\\{\\{\\s*category\\s*\\}\\}/g, product.category);
        return html;
      }).join('');
    },
  };

  // Dispatch event when data is ready
  window.dispatchEvent(new CustomEvent('rustpress:ready', { detail: window.RustPress }));

  console.log('[RustPress Preview] Live theme data context loaded');
  console.log('[RustPress Preview] Posts:', window.posts.length);
  console.log('[RustPress Preview] Categories:', window.categories.length);
  console.log('[RustPress Preview] Products:', window.products.length);
</script>
`;
}

// ============================================
// DEFAULT INSTANCES
// ============================================

export const themeDataService = new ThemeDataService();
export const realTimeThemeDataService = new RealTimeThemeDataService();

export default themeDataService;
