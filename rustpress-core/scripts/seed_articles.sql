-- Seed script: 10 articles about Rust, React, AI, and CRM
-- Run with: docker exec rustpress-postgres-dev psql -U rustpress -d rustpress_dev -f /scripts/seed_articles.sql

-- Get the admin user ID
DO $$
DECLARE
    admin_id UUID;
BEGIN
    SELECT id INTO admin_id FROM users WHERE email = 'admin@rustpress.dev' LIMIT 1;

    IF admin_id IS NULL THEN
        RAISE EXCEPTION 'Admin user not found. Please create an admin user first.';
    END IF;

    -- Article 1: Rust Memory Safety
    INSERT INTO posts (author_id, post_type, title, slug, content, excerpt, status, published_at)
    VALUES (
        admin_id,
        'post',
        'Why Rust Memory Safety Makes It Perfect for Enterprise CMS Development',
        'rust-memory-safety-enterprise-cms',
        '<h2>The Power of Memory Safety in Modern Web Applications</h2>
<p>When building enterprise-grade content management systems, memory safety isn''t just a nice-to-have feature—it''s essential. Rust''s ownership model provides compile-time guarantees that eliminate entire classes of bugs that have plagued traditional CMS platforms built in languages like PHP or JavaScript.</p>

<h3>Understanding Rust''s Ownership Model</h3>
<p>At its core, Rust enforces strict rules about how memory is accessed and modified. Every value in Rust has a single owner, and when that owner goes out of scope, the value is automatically deallocated. This prevents:</p>
<ul>
<li>Memory leaks from forgotten deallocations</li>
<li>Use-after-free vulnerabilities</li>
<li>Double-free errors</li>
<li>Data races in concurrent code</li>
</ul>

<h3>Real-World Impact on CMS Security</h3>
<p>Traditional CMS platforms like WordPress have suffered from numerous security vulnerabilities over the years, many stemming from memory-related bugs. By choosing Rust for RustPress, we''ve eliminated these concerns at the language level.</p>

<p>The borrow checker ensures that references to data are always valid, preventing the dangling pointer issues that have caused countless security patches in other platforms. This means fewer emergency updates and more stable deployments for your business.</p>

<h3>Performance Benefits</h3>
<p>Beyond security, Rust''s memory management approach delivers exceptional performance. Without garbage collection pauses and with precise control over memory layout, RustPress achieves response times that compete with static site generators while maintaining full dynamic CMS capabilities.</p>

<blockquote>
<p>"Rust gives us the performance of C with the safety guarantees of managed languages—the best of both worlds for enterprise software."</p>
</blockquote>

<h3>Conclusion</h3>
<p>Memory safety isn''t just about preventing crashes—it''s about building systems that enterprises can trust with their most critical content. Rust''s approach to memory management makes it the ideal foundation for the next generation of content management systems.</p>',
        'Discover how Rust''s memory safety features make it the ideal choice for building secure, high-performance enterprise CMS platforms like RustPress.',
        'published',
        NOW() - INTERVAL '2 days'
    );

    -- Article 2: React Server Components
    INSERT INTO posts (author_id, post_type, title, slug, content, excerpt, status, published_at)
    VALUES (
        admin_id,
        'post',
        'React Server Components: Revolutionizing How We Build CMS Admin Interfaces',
        'react-server-components-cms-admin',
        '<h2>The Evolution of React in Enterprise Applications</h2>
<p>React has come a long way since its introduction in 2013. With the advent of React Server Components (RSC), we''re witnessing a paradigm shift in how we build complex admin interfaces for content management systems.</p>

<h3>What Are React Server Components?</h3>
<p>React Server Components allow components to render on the server while maintaining the interactive, component-based architecture React is known for. This hybrid approach combines the best of server-side rendering with client-side interactivity.</p>

<h3>Benefits for CMS Development</h3>
<p>For RustPress''s admin interface, React Server Components provide several key advantages:</p>
<ul>
<li><strong>Reduced Bundle Size:</strong> Server components don''t ship JavaScript to the client</li>
<li><strong>Direct Database Access:</strong> Server components can query the database directly</li>
<li><strong>Improved Initial Load:</strong> Content renders on the server, appearing instantly</li>
<li><strong>Seamless Integration:</strong> Mix server and client components as needed</li>
</ul>

<h3>Building the RustPress Admin Panel</h3>
<p>Our admin interface leverages RSC for data-heavy operations like listing posts, displaying analytics, and managing media. Interactive elements like form inputs and drag-and-drop functionality use client components, creating a responsive experience without compromising on performance.</p>

<pre><code>// Server Component - fetches data on the server
async function PostsList() {
  const posts = await db.posts.findMany();
  return (
    &lt;div&gt;
      {posts.map(post =&gt; &lt;PostCard key={post.id} post={post} /&gt;)}
    &lt;/div&gt;
  );
}</code></pre>

<h3>The Future of React in CMS</h3>
<p>As React Server Components mature, we expect to see even more powerful patterns emerge. The ability to stream content, handle suspense boundaries, and progressively enhance pages opens new possibilities for content management interfaces.</p>',
        'Learn how React Server Components are transforming CMS admin interface development, enabling faster load times and better user experiences in RustPress.',
        'published',
        NOW() - INTERVAL '4 days'
    );

    -- Article 3: AI Content Generation
    INSERT INTO posts (author_id, post_type, title, slug, content, excerpt, status, published_at)
    VALUES (
        admin_id,
        'post',
        'AI-Powered Content Creation: How RustPress Integrates with Modern LLMs',
        'ai-powered-content-creation-rustpress',
        '<h2>The Rise of AI in Content Management</h2>
<p>Artificial Intelligence is transforming how we create, edit, and optimize content. RustPress embraces this revolution by integrating cutting-edge AI capabilities directly into the content creation workflow.</p>

<h3>Built-in AI Enhancement Tools</h3>
<p>RustPress includes a comprehensive suite of AI-powered tools designed to streamline your content workflow:</p>

<h4>Title Generator</h4>
<p>Struggling with headlines? Our AI title generator analyzes your content and suggests compelling, SEO-optimized titles that capture reader attention and improve click-through rates.</p>

<h4>Content Summarizer</h4>
<p>Create perfect excerpts and meta descriptions automatically. The summarizer distills your content into concise, engaging summaries that respect character limits while maintaining key messaging.</p>

<h4>SEO Optimizer</h4>
<p>Real-time suggestions for improving your content''s search engine visibility. From keyword density to readability scores, the SEO optimizer ensures your content ranks.</p>

<h4>Grammar and Tone Adjuster</h4>
<p>Polish your prose with AI-powered grammar checking and tone adjustment. Whether you need formal business communication or casual blog posts, the tone adjuster adapts your content accordingly.</p>

<h3>Privacy-First AI Implementation</h3>
<p>Unlike cloud-dependent solutions, RustPress offers options for local AI processing. Run smaller models on your own infrastructure for sensitive content, or leverage cloud APIs for maximum capability—the choice is yours.</p>

<h3>The Human-AI Collaboration</h3>
<p>AI doesn''t replace human creativity—it amplifies it. RustPress positions AI as a collaborative tool that handles routine tasks, allowing content creators to focus on strategy, storytelling, and audience engagement.</p>

<blockquote>
<p>"The best content combines human insight with AI efficiency. RustPress makes this collaboration seamless."</p>
</blockquote>',
        'Explore how RustPress integrates AI tools for title generation, content summarization, SEO optimization, and more to supercharge your content workflow.',
        'published',
        NOW() - INTERVAL '1 day'
    );

    -- Article 4: CRM Integration
    INSERT INTO posts (author_id, post_type, title, slug, content, excerpt, status, published_at)
    VALUES (
        admin_id,
        'post',
        'Connecting Your CMS to CRM: Building Unified Customer Experiences',
        'cms-crm-integration-unified-experience',
        '<h2>Breaking Down the CMS-CRM Divide</h2>
<p>Too often, content management and customer relationship management exist in separate silos. This disconnect leads to fragmented customer experiences and missed opportunities for personalization. RustPress bridges this gap.</p>

<h3>Why Integration Matters</h3>
<p>When your CMS and CRM communicate seamlessly, you unlock powerful capabilities:</p>
<ul>
<li><strong>Personalized Content:</strong> Display content based on customer segment, purchase history, or engagement level</li>
<li><strong>Lead Capture:</strong> Form submissions flow directly into your CRM pipeline</li>
<li><strong>Behavioral Tracking:</strong> Content engagement feeds into customer profiles</li>
<li><strong>Automated Nurturing:</strong> Trigger email sequences based on content interactions</li>
</ul>

<h3>RustPress Integration Architecture</h3>
<p>RustPress provides flexible integration options for popular CRM platforms:</p>

<h4>Native Integrations</h4>
<p>Out-of-the-box connectors for Salesforce, HubSpot, Pipedrive, and Zoho CRM. Configure once and enjoy automatic data synchronization.</p>

<h4>Webhook System</h4>
<p>For custom CRM solutions, our robust webhook system triggers on content events—new posts, form submissions, user registrations, and more.</p>

<h4>API-First Design</h4>
<p>Every RustPress feature is accessible via REST API, enabling deep custom integrations with any CRM system.</p>

<h3>Real-World Use Case: Lead Scoring</h3>
<p>Imagine a prospect visiting your pricing page, downloading a whitepaper, and watching a product video. With CRM integration, each touchpoint updates their lead score automatically, signaling to your sales team when they''re ready for outreach.</p>

<h3>Implementation Best Practices</h3>
<ol>
<li>Map your customer journey before integrating</li>
<li>Define clear data ownership between systems</li>
<li>Implement proper consent management for GDPR compliance</li>
<li>Test thoroughly in staging before production deployment</li>
</ol>',
        'Learn how to integrate RustPress with your CRM system to create unified, personalized customer experiences that drive conversions.',
        'published',
        NOW() - INTERVAL '5 days'
    );

    -- Article 5: Rust Async Performance
    INSERT INTO posts (author_id, post_type, title, slug, content, excerpt, status, published_at)
    VALUES (
        admin_id,
        'post',
        'Async Rust: How Tokio Powers RustPress''s Concurrent Request Handling',
        'async-rust-tokio-concurrent-requests',
        '<h2>Understanding Async in Modern Web Servers</h2>
<p>Handling thousands of concurrent connections efficiently is crucial for any modern CMS. RustPress achieves this through Rust''s async/await system, powered by the Tokio runtime.</p>

<h3>The Tokio Advantage</h3>
<p>Tokio is Rust''s most popular async runtime, providing:</p>
<ul>
<li>Work-stealing scheduler for optimal CPU utilization</li>
<li>Non-blocking I/O for network and file operations</li>
<li>Timers and intervals for scheduled tasks</li>
<li>Channels for inter-task communication</li>
</ul>

<h3>How RustPress Leverages Async</h3>
<p>Every HTTP request in RustPress is handled asynchronously. When a request comes in, it doesn''t block a thread waiting for database queries or file reads. Instead, the runtime efficiently manages thousands of in-flight requests across a small pool of threads.</p>

<pre><code>async fn get_post(Path(id): Path&lt;Uuid&gt;, State(db): State&lt;Pool&gt;) -&gt; Result&lt;Json&lt;Post&gt;&gt; {
    // This await doesn''t block the thread
    let post = sqlx::query_as!(&quot;SELECT * FROM posts WHERE id = $1&quot;, id)
        .fetch_one(&amp;db)
        .await?;

    Ok(Json(post))
}</code></pre>

<h3>Benchmarks: RustPress vs Traditional CMS</h3>
<p>In our internal benchmarks, RustPress handles 10x more concurrent requests than WordPress on equivalent hardware. More importantly, response times remain consistent under load, without the performance degradation seen in blocking architectures.</p>

<table>
<tr><th>Metric</th><th>RustPress</th><th>WordPress</th></tr>
<tr><td>Requests/sec</td><td>15,000</td><td>1,500</td></tr>
<tr><td>P99 Latency</td><td>12ms</td><td>180ms</td></tr>
<tr><td>Memory Usage</td><td>50MB</td><td>256MB</td></tr>
</table>

<h3>Practical Implications</h3>
<p>This performance means lower hosting costs, better user experience during traffic spikes, and the ability to handle viral content without emergency scaling.</p>',
        'Dive deep into how RustPress uses Tokio and Rust''s async/await to handle massive concurrent loads with minimal resource usage.',
        'published',
        NOW() - INTERVAL '7 days'
    );

    -- Article 6: React Hooks for CMS
    INSERT INTO posts (author_id, post_type, title, slug, content, excerpt, status, published_at)
    VALUES (
        admin_id,
        'post',
        'Essential React Hooks for Building CMS Admin Interfaces',
        'react-hooks-cms-admin-interfaces',
        '<h2>React Hooks: The Foundation of Modern CMS UIs</h2>
<p>Since their introduction in React 16.8, hooks have revolutionized how we build component logic. For CMS admin interfaces, the right hooks can dramatically simplify complex workflows.</p>

<h3>Custom Hooks for Content Management</h3>

<h4>useDebounce for Search</h4>
<p>When implementing search in a post listing, you don''t want to fire API requests on every keystroke. A debounce hook delays execution until the user stops typing:</p>

<pre><code>function useDebounce(value, delay) {
  const [debouncedValue, setDebouncedValue] = useState(value);

  useEffect(() =&gt; {
    const handler = setTimeout(() =&gt; setDebouncedValue(value), delay);
    return () =&gt; clearTimeout(handler);
  }, [value, delay]);

  return debouncedValue;
}</code></pre>

<h4>useAutoSave for Draft Preservation</h4>
<p>Nothing frustrates content creators more than losing work. An auto-save hook ensures drafts are preserved:</p>

<pre><code>function useAutoSave(content, saveFunction) {
  const [lastSaved, setLastSaved] = useState(null);
  const debouncedContent = useDebounce(content, 2000);

  useEffect(() =&gt; {
    if (debouncedContent) {
      saveFunction(debouncedContent);
      setLastSaved(new Date());
    }
  }, [debouncedContent]);

  return lastSaved;
}</code></pre>

<h4>useMediaLibrary for Asset Management</h4>
<p>Managing media in a CMS requires state for selection, filtering, and upload progress. A comprehensive hook encapsulates this complexity.</p>

<h3>Performance Optimization Hooks</h3>
<p>useMemo and useCallback prevent unnecessary re-renders in complex admin interfaces. When rendering large post lists or complex forms, these optimizations are essential.</p>

<h3>State Management Considerations</h3>
<p>For simpler admin interfaces, useState and useContext may suffice. As complexity grows, consider Zustand (which RustPress uses) or similar lightweight state management solutions that integrate well with hooks.</p>',
        'Master the essential React hooks that power modern CMS admin interfaces, from debounced search to auto-save functionality.',
        'published',
        NOW() - INTERVAL '3 days'
    );

    -- Article 7: AI for SEO
    INSERT INTO posts (author_id, post_type, title, slug, content, excerpt, status, published_at)
    VALUES (
        admin_id,
        'post',
        'Using AI to Supercharge Your Content''s SEO Performance',
        'ai-seo-optimization-content',
        '<h2>The AI-SEO Revolution</h2>
<p>Search engine optimization has evolved from keyword stuffing to sophisticated content quality assessment. Modern AI tools help you create content that ranks by focusing on what actually matters: relevance, depth, and user satisfaction.</p>

<h3>AI-Powered SEO Features in RustPress</h3>

<h4>Semantic Keyword Analysis</h4>
<p>Rather than matching exact keywords, AI understands semantic relationships. Our analyzer suggests related terms and concepts that strengthen topical authority without awkward keyword insertion.</p>

<h4>Content Gap Identification</h4>
<p>AI compares your content against top-ranking pages for your target keywords, identifying topics and questions you haven''t addressed. Fill these gaps to create more comprehensive content.</p>

<h4>Readability Optimization</h4>
<p>Search engines favor content that users engage with. AI readability tools ensure your content matches your audience''s reading level while maintaining necessary technical depth.</p>

<h4>Schema Markup Generation</h4>
<p>Structured data helps search engines understand your content. RustPress AI automatically generates appropriate schema markup for articles, products, FAQs, and more.</p>

<h3>Beyond On-Page SEO</h3>
<p>AI assists with:</p>
<ul>
<li><strong>Internal Linking:</strong> Suggest relevant internal links based on content analysis</li>
<li><strong>Image Optimization:</strong> Generate descriptive alt text for accessibility and SEO</li>
<li><strong>Meta Description Writing:</strong> Create compelling descriptions that improve CTR</li>
</ul>

<h3>Measuring AI SEO Impact</h3>
<p>RustPress integrates with Google Search Console to show before/after metrics for AI-optimized content. Track impressions, clicks, and ranking changes to quantify the value of AI assistance.</p>

<blockquote>
<p>"AI doesn''t game SEO—it helps you create the genuinely valuable content that modern search engines reward."</p>
</blockquote>',
        'Discover how RustPress''s AI-powered SEO tools help you create content that ranks by focusing on quality, relevance, and user intent.',
        'published',
        NOW() - INTERVAL '6 days'
    );

    -- Article 8: CRM Lead Management
    INSERT INTO posts (author_id, post_type, title, slug, content, excerpt, status, published_at)
    VALUES (
        admin_id,
        'post',
        'From Visitor to Customer: CRM-Driven Lead Management in Your CMS',
        'crm-lead-management-cms',
        '<h2>The Content-to-Customer Pipeline</h2>
<p>Every website visitor represents a potential customer. Effective CRM integration transforms your CMS from a content delivery system into a lead generation and nurturing machine.</p>

<h3>Capturing Leads Through Content</h3>

<h4>Progressive Profiling</h4>
<p>Don''t ask for everything upfront. Progressive profiling gathers information over multiple interactions, reducing form friction while building rich customer profiles.</p>

<h4>Content Gates</h4>
<p>RustPress supports gated content with customizable forms. Gate your highest-value content (whitepapers, reports, tools) to capture qualified leads.</p>

<h4>Exit Intent Popups</h4>
<p>Intelligent exit intent detection offers value propositions to departing visitors without disrupting engaged readers.</p>

<h3>Lead Scoring Based on Content Engagement</h3>
<p>Not all engagement is equal. Configure scoring rules based on:</p>
<ul>
<li>Page views (with different weights per page type)</li>
<li>Time on page (engaged reading vs. bounce)</li>
<li>Content downloads</li>
<li>Video watch completion</li>
<li>Return visits</li>
</ul>

<h3>Automated Nurturing Workflows</h3>
<p>When a lead reaches a score threshold, trigger automated sequences:</p>
<ol>
<li>Welcome email with relevant content recommendations</li>
<li>Educational content series based on interests</li>
<li>Case study delivery after sufficient engagement</li>
<li>Sales team notification for high-intent signals</li>
</ol>

<h3>Measuring Content ROI</h3>
<p>Track which content pieces generate leads and which leads convert. This data informs future content strategy, focusing resources on what actually drives revenue.</p>

<h3>Privacy Compliance</h3>
<p>RustPress includes built-in GDPR and CCPA compliance tools. Consent management, data access requests, and deletion workflows are handled automatically.</p>',
        'Learn how to transform your CMS into a lead generation powerhouse with CRM integration, scoring, and automated nurturing workflows.',
        'published',
        NOW() - INTERVAL '8 days'
    );

    -- Article 9: Rust Error Handling
    INSERT INTO posts (author_id, post_type, title, slug, content, excerpt, status, published_at)
    VALUES (
        admin_id,
        'post',
        'Rust''s Error Handling: Building Resilient CMS Applications',
        'rust-error-handling-resilient-cms',
        '<h2>Errors as Values: The Rust Philosophy</h2>
<p>In Rust, errors aren''t exceptions that can crash your application unexpectedly—they''re values that must be handled explicitly. This philosophy creates more resilient applications by making error handling a first-class concern.</p>

<h3>The Result Type</h3>
<p>Rust''s Result type represents operations that can fail:</p>

<pre><code>enum Result&lt;T, E&gt; {
    Ok(T),
    Err(E),
}

// Example: fetching a post
fn get_post(id: Uuid) -&gt; Result&lt;Post, PostError&gt; {
    let post = db.query(&quot;SELECT * FROM posts WHERE id = $1&quot;, &amp;[&amp;id])?;
    Ok(post)
}</code></pre>

<h3>Graceful Degradation in RustPress</h3>
<p>When errors occur, RustPress degrades gracefully rather than crashing:</p>

<h4>Database Connectivity</h4>
<p>If the database becomes temporarily unavailable, cached content continues serving while connection retry logic works in the background.</p>

<h4>External Services</h4>
<p>AI features, email sending, and CRM integrations have circuit breakers. If a service fails repeatedly, RustPress stops calling it temporarily, preventing cascade failures.</p>

<h4>User-Facing Errors</h4>
<p>Detailed error information logs for debugging, but users see friendly, actionable messages without sensitive system details.</p>

<h3>Custom Error Types</h3>
<p>RustPress defines domain-specific error types that provide context:</p>

<pre><code>pub enum ContentError {
    NotFound { id: Uuid },
    PermissionDenied { user: Uuid, resource: String },
    ValidationFailed { fields: Vec&lt;FieldError&gt; },
    DatabaseError(sqlx::Error),
}</code></pre>

<h3>Panic Safety</h3>
<p>Unlike exceptions, panics in Rust don''t unwind through arbitrary code. RustPress catches panics at request boundaries, ensuring one bad request can''t crash the server.</p>

<h3>Monitoring and Alerting</h3>
<p>All errors are structured and logged with context. Integration with services like Sentry provides real-time alerting when error rates spike.</p>',
        'Understand how Rust''s error-as-values approach creates more resilient CMS applications that handle failures gracefully.',
        'published',
        NOW() - INTERVAL '10 days'
    );

    -- Article 10: React Performance Optimization
    INSERT INTO posts (author_id, post_type, title, slug, content, excerpt, status, published_at)
    VALUES (
        admin_id,
        'post',
        'React Performance Optimization for Complex CMS Admin Dashboards',
        'react-performance-optimization-cms-dashboard',
        '<h2>When Admin Interfaces Get Slow</h2>
<p>Content management dashboards can become sluggish as they display more data and offer more features. React''s declarative model is powerful, but without optimization, re-renders can become a bottleneck.</p>

<h3>Identifying Performance Issues</h3>

<h4>React DevTools Profiler</h4>
<p>The Profiler tab in React DevTools shows exactly which components re-render and how long they take. Look for components that render frequently or take more than 16ms.</p>

<h4>Why Did You Render</h4>
<p>This library logs unnecessary re-renders caused by new object/array references or unchanged props.</p>

<h3>Optimization Techniques</h3>

<h4>Memoization</h4>
<p>React.memo prevents re-renders when props haven''t changed:</p>

<pre><code>const PostCard = React.memo(function PostCard({ post }) {
  return (
    &lt;div className=&quot;card&quot;&gt;
      &lt;h3&gt;{post.title}&lt;/h3&gt;
      &lt;p&gt;{post.excerpt}&lt;/p&gt;
    &lt;/div&gt;
  );
});</code></pre>

<h4>Virtual Scrolling</h4>
<p>For long lists (posts, media, users), virtual scrolling renders only visible items. Libraries like react-window or react-virtualized handle this efficiently.</p>

<h4>Code Splitting</h4>
<p>Don''t load the entire admin bundle upfront. Lazy load routes and heavy components:</p>

<pre><code>const MediaLibrary = lazy(() =&gt; import(''./MediaLibrary''));
const Analytics = lazy(() =&gt; import(''./Analytics''));</code></pre>

<h4>Debouncing State Updates</h4>
<p>User input often triggers state updates. Debounce expensive operations:</p>

<pre><code>const debouncedSearch = useMemo(
  () =&gt; debounce((query) =&gt; fetchResults(query), 300),
  []
);</code></pre>

<h3>State Management Optimization</h3>
<p>Choose state management carefully. Global state that changes frequently causes widespread re-renders. Use local state for UI concerns and global state only for truly shared data.</p>

<h3>Measuring Improvement</h3>
<p>Use Lighthouse and Web Vitals to measure improvement. Focus on Interaction to Next Paint (INP) for admin interface responsiveness.</p>',
        'Master React performance optimization techniques to keep your CMS admin dashboard fast and responsive, even with large datasets.',
        'published',
        NOW() - INTERVAL '9 days'
    );

    RAISE NOTICE 'Successfully inserted 10 articles!';
END $$;
