//! Pre-built Workers templates for RustPress integration

/// Cache Worker template - intelligent caching with RustPress awareness
pub const CACHE_WORKER: &str = r#"
addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request))
})

async function handleRequest(request) {
  const url = new URL(request.url)

  // Skip cache for admin and API routes
  if (url.pathname.startsWith('/admin') || url.pathname.startsWith('/api/')) {
    return fetch(request)
  }

  // Check cache first
  const cache = caches.default
  let response = await cache.match(request)

  if (!response) {
    response = await fetch(request)

    // Cache successful GET responses
    if (request.method === 'GET' && response.status === 200) {
      const headers = new Headers(response.headers)
      headers.set('Cache-Control', 'public, max-age=3600')

      const cachedResponse = new Response(response.clone().body, {
        status: response.status,
        statusText: response.statusText,
        headers: headers
      })

      event.waitUntil(cache.put(request, cachedResponse))
    }
  }

  return response
}
"#;

/// Security Worker template - additional security headers and protections
pub const SECURITY_WORKER: &str = r#"
addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request))
})

async function handleRequest(request) {
  const response = await fetch(request)
  const headers = new Headers(response.headers)

  // Security headers
  headers.set('X-Content-Type-Options', 'nosniff')
  headers.set('X-Frame-Options', 'SAMEORIGIN')
  headers.set('X-XSS-Protection', '1; mode=block')
  headers.set('Referrer-Policy', 'strict-origin-when-cross-origin')
  headers.set('Permissions-Policy', 'geolocation=(), microphone=(), camera=()')

  return new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers: headers
  })
}
"#;

/// Image optimization Worker template
pub const IMAGE_WORKER: &str = r#"
addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request))
})

async function handleRequest(request) {
  const url = new URL(request.url)

  // Only process image requests
  if (!isImagePath(url.pathname)) {
    return fetch(request)
  }

  // Get optimization parameters
  const width = url.searchParams.get('w')
  const quality = url.searchParams.get('q') || '80'
  const format = url.searchParams.get('f') || 'auto'

  const imageURL = url.origin + url.pathname

  const options = {
    cf: {
      image: {
        quality: parseInt(quality),
        format: format === 'auto' ? 'webp' : format
      }
    }
  }

  if (width) {
    options.cf.image.width = parseInt(width)
  }

  return fetch(imageURL, options)
}

function isImagePath(pathname) {
  return /\.(jpg|jpeg|png|gif|webp|svg)$/i.test(pathname)
}
"#;

/// Redirect Worker template for URL management
pub const REDIRECT_WORKER: &str = r#"
const redirects = {
  // Add your redirects here
  // '/old-path': '/new-path',
}

addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request))
})

async function handleRequest(request) {
  const url = new URL(request.url)

  if (redirects[url.pathname]) {
    return Response.redirect(url.origin + redirects[url.pathname], 301)
  }

  return fetch(request)
}
"#;

/// Analytics Worker template
pub const ANALYTICS_WORKER: &str = r#"
addEventListener('fetch', event => {
  event.passThroughOnException()
  event.respondWith(handleRequest(event))
})

async function handleRequest(event) {
  const request = event.request
  const response = await fetch(request)

  // Log analytics data asynchronously
  event.waitUntil(logAnalytics(request, response))

  return response
}

async function logAnalytics(request, response) {
  const url = new URL(request.url)
  const data = {
    timestamp: Date.now(),
    path: url.pathname,
    method: request.method,
    status: response.status,
    country: request.cf?.country || 'unknown',
    device: request.headers.get('user-agent'),
  }

  // Send to analytics endpoint
  // await fetch('https://your-analytics-endpoint.com', {
  //   method: 'POST',
  //   body: JSON.stringify(data)
  // })
}
"#;

/// Get all available worker templates
pub fn get_templates() -> Vec<WorkerTemplate> {
    vec![
        WorkerTemplate {
            id: "cache".to_string(),
            name: "Intelligent Cache".to_string(),
            description: "Smart caching with RustPress route awareness".to_string(),
            script: CACHE_WORKER.to_string(),
        },
        WorkerTemplate {
            id: "security".to_string(),
            name: "Security Headers".to_string(),
            description: "Add security headers to all responses".to_string(),
            script: SECURITY_WORKER.to_string(),
        },
        WorkerTemplate {
            id: "image".to_string(),
            name: "Image Optimization".to_string(),
            description: "On-the-fly image optimization and resizing".to_string(),
            script: IMAGE_WORKER.to_string(),
        },
        WorkerTemplate {
            id: "redirect".to_string(),
            name: "URL Redirects".to_string(),
            description: "Manage URL redirections at the edge".to_string(),
            script: REDIRECT_WORKER.to_string(),
        },
        WorkerTemplate {
            id: "analytics".to_string(),
            name: "Edge Analytics".to_string(),
            description: "Collect analytics data at the edge".to_string(),
            script: ANALYTICS_WORKER.to_string(),
        },
    ]
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkerTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub script: String,
}
