#!/usr/bin/env python3
"""Test ALL RustPress API endpoints"""
import requests
import json
import time

BASE_URL = "http://127.0.0.1:8080"
TOKEN = None
CREATED_IDS = {}
TIMESTAMP = str(int(time.time()))  # Unique timestamp for test data

def test(method, endpoint, auth=False, data=None, expected=[200, 201, 204]):
    """Test an endpoint and return status"""
    url = f"{BASE_URL}{endpoint}"
    headers = {"Content-Type": "application/json"}
    if auth and TOKEN:
        headers["Authorization"] = f"Bearer {TOKEN}"

    try:
        if method == "GET":
            r = requests.get(url, headers=headers, timeout=10)
        elif method == "POST":
            r = requests.post(url, headers=headers, json=data, timeout=10)
        elif method == "PUT":
            r = requests.put(url, headers=headers, json=data, timeout=10)
        elif method == "PATCH":
            r = requests.patch(url, headers=headers, json=data, timeout=10)
        elif method == "DELETE":
            r = requests.delete(url, headers=headers, timeout=10)
        else:
            return "UNKNOWN METHOD", None

        if isinstance(expected, list):
            status = "OK" if r.status_code in expected else "FAIL"
        else:
            status = "OK" if r.status_code == expected else "FAIL"

        return f"{r.status_code} {status}", r
    except Exception as e:
        return f"ERR: {str(e)[:30]}", None

def p(method, endpoint, result):
    """Print result"""
    print(f"{method:6} {endpoint:45} {result}")

# ============================================
# LOGIN
# ============================================
print("=" * 70)
print("RUSTPRESS API ENDPOINT TEST")
print("=" * 70)

print("\n[AUTH ENDPOINTS]")
r = requests.post(f"{BASE_URL}/api/v1/auth/login",
                  json={"email": "admin", "password": "admin123"}, timeout=10)
if r.status_code == 200:
    TOKEN = r.json()["access_token"]
    REFRESH = r.json().get("refresh_token")
    p("POST", "/api/v1/auth/login", "200 OK")
else:
    p("POST", "/api/v1/auth/login", f"{r.status_code} FAIL - Cannot continue")
    exit(1)

p("GET", "/api/v1/auth/me", test("GET", "/api/v1/auth/me", auth=True)[0])
p("POST", "/api/v1/auth/refresh", test("POST", "/api/v1/auth/refresh", auth=True, data={"refresh_token": REFRESH})[0])
p("POST", "/api/v1/auth/logout", test("POST", "/api/v1/auth/logout", auth=True)[0])

# Re-login after logout
r = requests.post(f"{BASE_URL}/api/v1/auth/login", json={"email": "admin", "password": "admin123"})
TOKEN = r.json()["access_token"]

# ============================================
# PUBLIC ENDPOINTS
# ============================================
print("\n[PUBLIC ENDPOINTS]")
p("GET", "/health", test("GET", "/health")[0])
p("GET", "/metrics", test("GET", "/metrics")[0])
p("GET", "/api/v1/posts", test("GET", "/api/v1/posts")[0])
p("GET", "/api/v1/categories", test("GET", "/api/v1/categories")[0])
p("GET", "/api/v1/tags", test("GET", "/api/v1/tags")[0])

# ============================================
# POSTS
# ============================================
print("\n[POSTS ENDPOINTS]")
p("GET", "/api/v1/posts", test("GET", "/api/v1/posts", auth=True)[0])

result, r = test("POST", "/api/v1/posts", auth=True, data={
    "title": "Test Post", "content": "<p>Test</p>", "status": "draft", "slug": f"test-post-api-{TIMESTAMP}"
})
p("POST", "/api/v1/posts", result)
if r and r.status_code in [200, 201]:
    post_id = r.json().get("data", {}).get("id")
    CREATED_IDS["post"] = post_id
    p("GET", f"/api/v1/posts/{post_id[:8]}...", test("GET", f"/api/v1/posts/{post_id}", auth=True)[0])
    p("PUT", f"/api/v1/posts/{post_id[:8]}...", test("PUT", f"/api/v1/posts/{post_id}", auth=True, data={"title": "Updated"})[0])

p("GET", "/api/v1/posts/revisions (if any)", test("GET", f"/api/v1/posts/{CREATED_IDS.get('post', 'x')}/revisions", auth=True)[0])

# ============================================
# PAGES
# ============================================
print("\n[PAGES ENDPOINTS]")
p("GET", "/api/v1/pages", test("GET", "/api/v1/pages", auth=True)[0])

result, r = test("POST", "/api/v1/pages", auth=True, data={
    "title": "Test Page", "content": "<p>Page</p>", "status": "draft", "slug": f"test-page-api-{TIMESTAMP}"
})
p("POST", "/api/v1/pages", result)
if r and r.status_code in [200, 201]:
    page_id = r.json().get("data", {}).get("id")
    CREATED_IDS["page"] = page_id
    p("GET", f"/api/v1/pages/{page_id[:8]}...", test("GET", f"/api/v1/pages/{page_id}", auth=True)[0])
    p("PUT", f"/api/v1/pages/{page_id[:8]}...", test("PUT", f"/api/v1/pages/{page_id}", auth=True, data={"title": "Updated Page"})[0])

# ============================================
# USERS
# ============================================
print("\n[USERS ENDPOINTS]")
p("GET", "/api/v1/users", test("GET", "/api/v1/users", auth=True)[0])
p("GET", "/api/v1/users/me", test("GET", "/api/v1/users/me", auth=True)[0])

result, r = test("POST", "/api/v1/users", auth=True, data={
    "email": f"test{TIMESTAMP}@example.com", "username": f"testuser{TIMESTAMP}", "password": "TestPass123", "role": "subscriber"
})
p("POST", "/api/v1/users", result)
if r and r.status_code in [200, 201]:
    user_id = r.json().get("data", {}).get("id") or r.json().get("id")
    if user_id:
        CREATED_IDS["user"] = user_id
        p("GET", f"/api/v1/users/{user_id[:8]}...", test("GET", f"/api/v1/users/{user_id}", auth=True)[0])
        p("PUT", f"/api/v1/users/{user_id[:8]}...", test("PUT", f"/api/v1/users/{user_id}", auth=True, data={"display_name": "Test User"})[0])

# ============================================
# MEDIA
# ============================================
print("\n[MEDIA ENDPOINTS]")
p("GET", "/api/v1/media", test("GET", "/api/v1/media", auth=True)[0])
p("GET", "/api/v1/media/folders", test("GET", "/api/v1/media/folders", auth=True)[0])

# ============================================
# COMMENTS
# ============================================
print("\n[COMMENTS ENDPOINTS]")
p("GET", "/api/v1/comments", test("GET", "/api/v1/comments", auth=True)[0])

if CREATED_IDS.get("post"):
    result, r = test("POST", "/api/v1/comments", auth=True, data={
        "post_id": CREATED_IDS["post"], "content": "Test comment", "author_name": "Tester", "author_email": "test@test.com"
    })
    p("POST", "/api/v1/comments", result)
    if r and r.status_code in [200, 201]:
        comment_id = r.json().get("data", {}).get("id") or r.json().get("id")
        if comment_id:
            CREATED_IDS["comment"] = comment_id
            p("GET", f"/api/v1/comments/{comment_id[:8]}...", test("GET", f"/api/v1/comments/{comment_id}", auth=True)[0])

# ============================================
# CATEGORIES
# ============================================
print("\n[CATEGORIES ENDPOINTS]")
p("GET", "/api/v1/categories", test("GET", "/api/v1/categories", auth=True)[0])
p("GET", "/api/v1/taxonomies/categories", test("GET", "/api/v1/taxonomies/categories", auth=True)[0])

result, r = test("POST", "/api/v1/categories", auth=True, data={
    "name": f"Test Category {TIMESTAMP}", "slug": f"test-category-{TIMESTAMP}", "description": "A test category"
})
p("POST", "/api/v1/categories", result)
if r and r.status_code in [200, 201]:
    cat_id = r.json().get("data", {}).get("id") or r.json().get("id")
    if cat_id:
        CREATED_IDS["category"] = cat_id
        p("GET", f"/api/v1/categories/{cat_id[:8]}...", test("GET", f"/api/v1/categories/{cat_id}", auth=True)[0])
        p("PUT", f"/api/v1/categories/{cat_id[:8]}...", test("PUT", f"/api/v1/categories/{cat_id}", auth=True, data={"name": "Updated Cat"})[0])

# ============================================
# TAGS
# ============================================
print("\n[TAGS ENDPOINTS]")
p("GET", "/api/v1/tags", test("GET", "/api/v1/tags", auth=True)[0])
p("GET", "/api/v1/taxonomies/tags", test("GET", "/api/v1/taxonomies/tags", auth=True)[0])

result, r = test("POST", "/api/v1/tags", auth=True, data={
    "name": f"Test Tag {TIMESTAMP}", "slug": f"test-tag-{TIMESTAMP}"
})
p("POST", "/api/v1/tags", result)
if r and r.status_code in [200, 201]:
    tag_id = r.json().get("data", {}).get("id") or r.json().get("id")
    if tag_id:
        CREATED_IDS["tag"] = tag_id
        p("GET", f"/api/v1/tags/{tag_id[:8]}...", test("GET", f"/api/v1/tags/{tag_id}", auth=True)[0])

# ============================================
# THEMES
# ============================================
print("\n[THEMES ENDPOINTS]")
p("GET", "/api/v1/themes", test("GET", "/api/v1/themes", auth=True)[0])
p("POST", "/api/v1/themes (scan)", test("POST", "/api/v1/themes", auth=True)[0])
p("GET", "/api/v1/themes/active", test("GET", "/api/v1/themes/active", auth=True)[0])

# ============================================
# SETTINGS
# ============================================
print("\n[SETTINGS ENDPOINTS]")
p("GET", "/api/v1/settings", test("GET", "/api/v1/settings", auth=True)[0])
p("GET", "/api/v1/settings/general", test("GET", "/api/v1/settings/general", auth=True)[0])
p("GET", "/api/v1/settings/reading", test("GET", "/api/v1/settings/reading", auth=True)[0])
p("GET", "/api/v1/settings/writing", test("GET", "/api/v1/settings/writing", auth=True)[0])
p("GET", "/api/v1/settings/discussion", test("GET", "/api/v1/settings/discussion", auth=True)[0])
p("GET", "/api/v1/settings/permalinks", test("GET", "/api/v1/settings/permalinks", auth=True)[0])

# ============================================
# MENUS
# ============================================
print("\n[MENUS ENDPOINTS]")
p("GET", "/api/v1/menus", test("GET", "/api/v1/menus", auth=True)[0])
p("GET", "/api/v1/menus/locations", test("GET", "/api/v1/menus/locations", auth=True)[0])

result, r = test("POST", "/api/v1/menus", auth=True, data={
    "name": f"Test Menu {TIMESTAMP}", "location": "primary"
})
p("POST", "/api/v1/menus", result)
if r and r.status_code in [200, 201]:
    menu_id = r.json().get("data", {}).get("id") or r.json().get("id")
    if menu_id:
        CREATED_IDS["menu"] = menu_id
        p("GET", f"/api/v1/menus/{menu_id[:8]}...", test("GET", f"/api/v1/menus/{menu_id}", auth=True)[0])

# ============================================
# WIDGETS
# ============================================
print("\n[WIDGETS ENDPOINTS]")
p("GET", "/api/v1/widgets", test("GET", "/api/v1/widgets", auth=True)[0])
p("GET", "/api/v1/widgets/types", test("GET", "/api/v1/widgets/types", auth=True)[0])
p("GET", "/api/v1/widgets/areas", test("GET", "/api/v1/widgets/areas", auth=True)[0])

# ============================================
# STATS / DASHBOARD
# ============================================
print("\n[STATS ENDPOINTS]")
p("GET", "/api/v1/stats/dashboard", test("GET", "/api/v1/stats/dashboard", auth=True)[0])
p("GET", "/api/v1/stats/posts", test("GET", "/api/v1/stats/posts", auth=True)[0])

# ============================================
# PLUGINS
# ============================================
print("\n[PLUGINS ENDPOINTS]")
p("GET", "/api/v1/plugins", test("GET", "/api/v1/plugins", auth=True)[0])

# ============================================
# BACKUPS
# ============================================
print("\n[BACKUPS ENDPOINTS]")
p("GET", "/api/v1/backups", test("GET", "/api/v1/backups", auth=True)[0])

# ============================================
# SEO
# ============================================
print("\n[SEO ENDPOINTS]")
p("GET", "/api/v1/seo/settings", test("GET", "/api/v1/seo/settings", auth=True)[0])

# ============================================
# CACHE
# ============================================
print("\n[CACHE ENDPOINTS]")
p("GET", "/api/v1/cache/stats", test("GET", "/api/v1/cache/stats", auth=True)[0])
p("POST", "/api/v1/cache/clear", test("POST", "/api/v1/cache/clear", auth=True)[0])

# ============================================
# CDN
# ============================================
print("\n[CDN ENDPOINTS]")
p("GET", "/api/v1/cdn/config", test("GET", "/api/v1/cdn/config", auth=True)[0])

# ============================================
# EMAIL
# ============================================
print("\n[EMAIL ENDPOINTS]")
p("GET", "/api/v1/email/templates", test("GET", "/api/v1/email/templates", auth=True)[0])
p("GET", "/api/v1/email/settings", test("GET", "/api/v1/email/settings", auth=True)[0])

# ============================================
# SEARCH
# ============================================
print("\n[SEARCH ENDPOINTS]")
p("GET", "/api/v1/search?q=test", test("GET", "/api/v1/search?q=test", auth=True)[0])

# ============================================
# CLEANUP - Delete created test resources
# ============================================
print("\n[CLEANUP - Deleting test resources]")
if CREATED_IDS.get("post"):
    p("DELETE", f"/api/v1/posts/{CREATED_IDS['post'][:8]}...", test("DELETE", f"/api/v1/posts/{CREATED_IDS['post']}", auth=True)[0])
if CREATED_IDS.get("page"):
    p("DELETE", f"/api/v1/pages/{CREATED_IDS['page'][:8]}...", test("DELETE", f"/api/v1/pages/{CREATED_IDS['page']}", auth=True)[0])
if CREATED_IDS.get("category"):
    p("DELETE", f"/api/v1/categories/{CREATED_IDS['category'][:8]}...", test("DELETE", f"/api/v1/categories/{CREATED_IDS['category']}", auth=True)[0])
if CREATED_IDS.get("tag"):
    p("DELETE", f"/api/v1/tags/{CREATED_IDS['tag'][:8]}...", test("DELETE", f"/api/v1/tags/{CREATED_IDS['tag']}", auth=True)[0])
if CREATED_IDS.get("menu"):
    p("DELETE", f"/api/v1/menus/{CREATED_IDS['menu'][:8]}...", test("DELETE", f"/api/v1/menus/{CREATED_IDS['menu']}", auth=True)[0])
if CREATED_IDS.get("user"):
    p("DELETE", f"/api/v1/users/{CREATED_IDS['user'][:8]}...", test("DELETE", f"/api/v1/users/{CREATED_IDS['user']}", auth=True)[0])

print("\n" + "=" * 70)
print("TEST COMPLETE")
print("=" * 70)
