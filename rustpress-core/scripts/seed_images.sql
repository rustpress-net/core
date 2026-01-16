-- Seed script: Add featured images to articles
-- Run after seed_articles.sql

DO $$
DECLARE
    admin_id UUID := '019b4752-488d-76d2-b859-2ae7d094864d';
    new_id UUID;
BEGIN
    -- Image 1: Rust/Memory Safety
    INSERT INTO media (uploader_id, filename, original_filename, mime_type, media_type, file_size, storage_path, url, width, height, title, alt_text)
    VALUES (admin_id, 'rust-memory-safety.jpg', 'rust-memory-safety.jpg', 'image/jpeg', 'image', 150000, 'external', 'https://images.unsplash.com/photo-1518770660439-4636190af475?w=1200&h=630&fit=crop', 1200, 630, 'Rust Memory Safety', 'Circuit board representing memory safety')
    RETURNING id INTO new_id;
    UPDATE posts SET featured_image_id = new_id WHERE slug = 'rust-memory-safety-enterprise-cms';
    RAISE NOTICE 'Added image for rust-memory-safety-enterprise-cms';

    -- Image 2: React Server Components
    INSERT INTO media (uploader_id, filename, original_filename, mime_type, media_type, file_size, storage_path, url, width, height, title, alt_text)
    VALUES (admin_id, 'react-server-components.jpg', 'react-server-components.jpg', 'image/jpeg', 'image', 150000, 'external', 'https://images.unsplash.com/photo-1633356122544-f134324a6cee?w=1200&h=630&fit=crop', 1200, 630, 'React Server Components', 'React code on computer screen')
    RETURNING id INTO new_id;
    UPDATE posts SET featured_image_id = new_id WHERE slug = 'react-server-components-cms-admin';
    RAISE NOTICE 'Added image for react-server-components-cms-admin';

    -- Image 3: AI Content
    INSERT INTO media (uploader_id, filename, original_filename, mime_type, media_type, file_size, storage_path, url, width, height, title, alt_text)
    VALUES (admin_id, 'ai-content-creation.jpg', 'ai-content-creation.jpg', 'image/jpeg', 'image', 150000, 'external', 'https://images.unsplash.com/photo-1677442136019-21780ecad995?w=1200&h=630&fit=crop', 1200, 630, 'AI Content Creation', 'AI brain visualization')
    RETURNING id INTO new_id;
    UPDATE posts SET featured_image_id = new_id WHERE slug = 'ai-powered-content-creation-rustpress';
    RAISE NOTICE 'Added image for ai-powered-content-creation-rustpress';

    -- Image 4: CRM Integration
    INSERT INTO media (uploader_id, filename, original_filename, mime_type, media_type, file_size, storage_path, url, width, height, title, alt_text)
    VALUES (admin_id, 'crm-integration.jpg', 'crm-integration.jpg', 'image/jpeg', 'image', 150000, 'external', 'https://images.unsplash.com/photo-1553877522-43269d4ea984?w=1200&h=630&fit=crop', 1200, 630, 'CRM Integration', 'Business handshake representing CRM')
    RETURNING id INTO new_id;
    UPDATE posts SET featured_image_id = new_id WHERE slug = 'cms-crm-integration-unified-experience';
    RAISE NOTICE 'Added image for cms-crm-integration-unified-experience';

    -- Image 5: Async Rust
    INSERT INTO media (uploader_id, filename, original_filename, mime_type, media_type, file_size, storage_path, url, width, height, title, alt_text)
    VALUES (admin_id, 'async-rust-tokio.jpg', 'async-rust-tokio.jpg', 'image/jpeg', 'image', 150000, 'external', 'https://images.unsplash.com/photo-1558494949-ef010cbdcc31?w=1200&h=630&fit=crop', 1200, 630, 'Async Rust Tokio', 'Server infrastructure')
    RETURNING id INTO new_id;
    UPDATE posts SET featured_image_id = new_id WHERE slug = 'async-rust-tokio-concurrent-requests';
    RAISE NOTICE 'Added image for async-rust-tokio-concurrent-requests';

    -- Image 6: React Hooks
    INSERT INTO media (uploader_id, filename, original_filename, mime_type, media_type, file_size, storage_path, url, width, height, title, alt_text)
    VALUES (admin_id, 'react-hooks-cms.jpg', 'react-hooks-cms.jpg', 'image/jpeg', 'image', 150000, 'external', 'https://images.unsplash.com/photo-1587620962725-abab7fe55159?w=1200&h=630&fit=crop', 1200, 630, 'React Hooks CMS', 'Developer coding with React')
    RETURNING id INTO new_id;
    UPDATE posts SET featured_image_id = new_id WHERE slug = 'react-hooks-cms-admin-interfaces';
    RAISE NOTICE 'Added image for react-hooks-cms-admin-interfaces';

    -- Image 7: AI SEO
    INSERT INTO media (uploader_id, filename, original_filename, mime_type, media_type, file_size, storage_path, url, width, height, title, alt_text)
    VALUES (admin_id, 'ai-seo-optimization.jpg', 'ai-seo-optimization.jpg', 'image/jpeg', 'image', 150000, 'external', 'https://images.unsplash.com/photo-1460925895917-afdab827c52f?w=1200&h=630&fit=crop', 1200, 630, 'AI SEO Optimization', 'Analytics dashboard showing growth')
    RETURNING id INTO new_id;
    UPDATE posts SET featured_image_id = new_id WHERE slug = 'ai-seo-optimization-content';
    RAISE NOTICE 'Added image for ai-seo-optimization-content';

    -- Image 8: CRM Lead Management
    INSERT INTO media (uploader_id, filename, original_filename, mime_type, media_type, file_size, storage_path, url, width, height, title, alt_text)
    VALUES (admin_id, 'crm-lead-management.jpg', 'crm-lead-management.jpg', 'image/jpeg', 'image', 150000, 'external', 'https://images.unsplash.com/photo-1552664730-d307ca884978?w=1200&h=630&fit=crop', 1200, 630, 'CRM Lead Management', 'Team collaborating on sales strategy')
    RETURNING id INTO new_id;
    UPDATE posts SET featured_image_id = new_id WHERE slug = 'crm-lead-management-cms';
    RAISE NOTICE 'Added image for crm-lead-management-cms';

    -- Image 9: Rust Error Handling
    INSERT INTO media (uploader_id, filename, original_filename, mime_type, media_type, file_size, storage_path, url, width, height, title, alt_text)
    VALUES (admin_id, 'rust-error-handling.jpg', 'rust-error-handling.jpg', 'image/jpeg', 'image', 150000, 'external', 'https://images.unsplash.com/photo-1555066931-4365d14bab8c?w=1200&h=630&fit=crop', 1200, 630, 'Rust Error Handling', 'Code on dark screen')
    RETURNING id INTO new_id;
    UPDATE posts SET featured_image_id = new_id WHERE slug = 'rust-error-handling-resilient-cms';
    RAISE NOTICE 'Added image for rust-error-handling-resilient-cms';

    -- Image 10: React Performance
    INSERT INTO media (uploader_id, filename, original_filename, mime_type, media_type, file_size, storage_path, url, width, height, title, alt_text)
    VALUES (admin_id, 'react-performance.jpg', 'react-performance.jpg', 'image/jpeg', 'image', 150000, 'external', 'https://images.unsplash.com/photo-1504868584819-f8e8b4b6d7e3?w=1200&h=630&fit=crop', 1200, 630, 'React Performance', 'Performance analytics dashboard')
    RETURNING id INTO new_id;
    UPDATE posts SET featured_image_id = new_id WHERE slug = 'react-performance-optimization-cms-dashboard';
    RAISE NOTICE 'Added image for react-performance-optimization-cms-dashboard';

    RAISE NOTICE 'Successfully added featured images to all 10 articles!';
END $$;
