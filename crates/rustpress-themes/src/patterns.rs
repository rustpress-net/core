//! Theme Block Patterns
//!
//! Reusable block pattern definitions and registration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Block pattern category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternCategory {
    pub slug: String,
    pub label: String,
    pub description: Option<String>,
}

/// Block pattern definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPattern {
    /// Unique pattern name (slug)
    pub name: String,
    /// Human-readable title
    pub title: String,
    /// Description of the pattern
    pub description: Option<String>,
    /// Categories this pattern belongs to
    pub categories: Vec<String>,
    /// Keywords for search
    pub keywords: Vec<String>,
    /// Block content (serialized blocks)
    pub content: String,
    /// Viewport width for preview
    pub viewport_width: Option<u32>,
    /// Block types this pattern is for
    pub block_types: Vec<String>,
    /// Template types this pattern can be used in
    pub template_types: Vec<String>,
    /// Whether to show in inserter
    pub inserter: bool,
    /// Source theme/plugin
    pub source: PatternSource,
}

/// Pattern source information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PatternSource {
    Core,
    Theme { theme_id: String },
    Plugin { plugin_id: String },
    User { user_id: String },
    Remote { url: String },
}

/// Block pattern registry
pub struct PatternRegistry {
    /// Registered patterns
    patterns: Arc<RwLock<HashMap<String, BlockPattern>>>,
    /// Pattern categories
    categories: Arc<RwLock<HashMap<String, PatternCategory>>>,
}

impl PatternRegistry {
    pub fn new() -> Self {
        let registry = Self {
            patterns: Arc::new(RwLock::new(HashMap::new())),
            categories: Arc::new(RwLock::new(HashMap::new())),
        };

        // Register default categories
        registry.register_default_categories();

        registry
    }

    fn register_default_categories(&self) {
        let default_categories = vec![
            PatternCategory {
                slug: "text".to_string(),
                label: "Text".to_string(),
                description: Some("Text patterns".to_string()),
            },
            PatternCategory {
                slug: "buttons".to_string(),
                label: "Buttons".to_string(),
                description: Some("Button patterns".to_string()),
            },
            PatternCategory {
                slug: "columns".to_string(),
                label: "Columns".to_string(),
                description: Some("Column layout patterns".to_string()),
            },
            PatternCategory {
                slug: "header".to_string(),
                label: "Headers".to_string(),
                description: Some("Header patterns".to_string()),
            },
            PatternCategory {
                slug: "footer".to_string(),
                label: "Footers".to_string(),
                description: Some("Footer patterns".to_string()),
            },
            PatternCategory {
                slug: "gallery".to_string(),
                label: "Gallery".to_string(),
                description: Some("Gallery patterns".to_string()),
            },
            PatternCategory {
                slug: "hero".to_string(),
                label: "Hero Sections".to_string(),
                description: Some("Hero section patterns".to_string()),
            },
            PatternCategory {
                slug: "call-to-action".to_string(),
                label: "Call to Action".to_string(),
                description: Some("CTA patterns".to_string()),
            },
            PatternCategory {
                slug: "testimonials".to_string(),
                label: "Testimonials".to_string(),
                description: Some("Testimonial patterns".to_string()),
            },
            PatternCategory {
                slug: "team".to_string(),
                label: "Team".to_string(),
                description: Some("Team member patterns".to_string()),
            },
            PatternCategory {
                slug: "pricing".to_string(),
                label: "Pricing".to_string(),
                description: Some("Pricing table patterns".to_string()),
            },
            PatternCategory {
                slug: "contact".to_string(),
                label: "Contact".to_string(),
                description: Some("Contact form patterns".to_string()),
            },
            PatternCategory {
                slug: "featured".to_string(),
                label: "Featured".to_string(),
                description: Some("Featured content patterns".to_string()),
            },
            PatternCategory {
                slug: "posts".to_string(),
                label: "Posts".to_string(),
                description: Some("Post display patterns".to_string()),
            },
        ];

        let mut categories = self.categories.write();
        for cat in default_categories {
            categories.insert(cat.slug.clone(), cat);
        }
    }

    /// Register a pattern
    pub fn register(&self, pattern: BlockPattern) {
        self.patterns.write().insert(pattern.name.clone(), pattern);
    }

    /// Unregister a pattern
    pub fn unregister(&self, name: &str) {
        self.patterns.write().remove(name);
    }

    /// Register a category
    pub fn register_category(&self, category: PatternCategory) {
        self.categories.write().insert(category.slug.clone(), category);
    }

    /// Get pattern by name
    pub fn get(&self, name: &str) -> Option<BlockPattern> {
        self.patterns.read().get(name).cloned()
    }

    /// Get all patterns
    pub fn get_all(&self) -> Vec<BlockPattern> {
        self.patterns.read().values().cloned().collect()
    }

    /// Get patterns by category
    pub fn get_by_category(&self, category: &str) -> Vec<BlockPattern> {
        self.patterns
            .read()
            .values()
            .filter(|p| p.categories.contains(&category.to_string()))
            .cloned()
            .collect()
    }

    /// Get patterns for block type
    pub fn get_for_block_type(&self, block_type: &str) -> Vec<BlockPattern> {
        self.patterns
            .read()
            .values()
            .filter(|p| p.block_types.is_empty() || p.block_types.contains(&block_type.to_string()))
            .cloned()
            .collect()
    }

    /// Search patterns
    pub fn search(&self, query: &str) -> Vec<BlockPattern> {
        let query_lower = query.to_lowercase();

        self.patterns
            .read()
            .values()
            .filter(|p| {
                p.title.to_lowercase().contains(&query_lower)
                    || p.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&query_lower))
                    || p.keywords.iter().any(|k| k.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }

    /// Get all categories
    pub fn get_categories(&self) -> Vec<PatternCategory> {
        self.categories.read().values().cloned().collect()
    }

    /// Get patterns for inserter (excluding hidden patterns)
    pub fn get_for_inserter(&self) -> Vec<BlockPattern> {
        self.patterns
            .read()
            .values()
            .filter(|p| p.inserter)
            .cloned()
            .collect()
    }

    /// Register patterns from theme manifest
    pub fn register_from_theme(&self, theme_id: &str, patterns: &[crate::manifest::PatternDefinition]) {
        for def in patterns {
            let pattern = BlockPattern {
                name: format!("{}/{}", theme_id, def.name),
                title: def.title.clone(),
                description: def.description.clone(),
                categories: def.categories.clone(),
                keywords: def.keywords.clone(),
                content: def.content.clone(),
                viewport_width: def.viewport_width,
                block_types: def.block_types.clone(),
                template_types: Vec::new(), // PatternDefinition doesn't have this field
                inserter: def.inserter,
                source: PatternSource::Theme { theme_id: theme_id.to_string() },
            };

            self.register(pattern);
        }
    }

    /// Unregister all patterns from a theme
    pub fn unregister_theme(&self, theme_id: &str) {
        let prefix = format!("{}/", theme_id);
        self.patterns.write().retain(|k, _| !k.starts_with(&prefix));
    }
}

impl Default for PatternRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Pattern builder for programmatic creation
pub struct PatternBuilder {
    pattern: BlockPattern,
}

impl PatternBuilder {
    pub fn new(name: &str, title: &str) -> Self {
        Self {
            pattern: BlockPattern {
                name: name.to_string(),
                title: title.to_string(),
                description: None,
                categories: Vec::new(),
                keywords: Vec::new(),
                content: String::new(),
                viewport_width: None,
                block_types: Vec::new(),
                template_types: Vec::new(),
                inserter: true,
                source: PatternSource::Core,
            },
        }
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.pattern.description = Some(desc.to_string());
        self
    }

    pub fn category(mut self, category: &str) -> Self {
        self.pattern.categories.push(category.to_string());
        self
    }

    pub fn categories(mut self, categories: Vec<&str>) -> Self {
        self.pattern.categories = categories.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn keywords(mut self, keywords: Vec<&str>) -> Self {
        self.pattern.keywords = keywords.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn content(mut self, content: &str) -> Self {
        self.pattern.content = content.to_string();
        self
    }

    pub fn viewport_width(mut self, width: u32) -> Self {
        self.pattern.viewport_width = Some(width);
        self
    }

    pub fn block_types(mut self, types: Vec<&str>) -> Self {
        self.pattern.block_types = types.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn template_types(mut self, types: Vec<&str>) -> Self {
        self.pattern.template_types = types.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn hidden(mut self) -> Self {
        self.pattern.inserter = false;
        self
    }

    pub fn source(mut self, source: PatternSource) -> Self {
        self.pattern.source = source;
        self
    }

    pub fn build(self) -> BlockPattern {
        self.pattern
    }
}

/// Create built-in patterns
pub fn register_builtin_patterns(registry: &PatternRegistry) {
    // Hero pattern
    registry.register(
        PatternBuilder::new("core/hero-centered", "Centered Hero")
            .description("A centered hero section with heading and buttons")
            .categories(vec!["hero", "featured"])
            .keywords(vec!["hero", "header", "intro", "landing"])
            .viewport_width(1200)
            .content(r#"
<!-- wp:group {"align":"full","style":{"spacing":{"padding":{"top":"var:preset|spacing|80","bottom":"var:preset|spacing|80"}}}} -->
<div class="wp-block-group alignfull" style="padding-top:var(--wp--preset--spacing--80);padding-bottom:var(--wp--preset--spacing--80)">
    <!-- wp:heading {"textAlign":"center","level":1} -->
    <h1 class="has-text-align-center">Welcome to Our Site</h1>
    <!-- /wp:heading -->

    <!-- wp:paragraph {"align":"center"} -->
    <p class="has-text-align-center">Discover amazing content and services.</p>
    <!-- /wp:paragraph -->

    <!-- wp:buttons {"layout":{"type":"flex","justifyContent":"center"}} -->
    <div class="wp-block-buttons">
        <!-- wp:button -->
        <div class="wp-block-button"><a class="wp-block-button__link">Get Started</a></div>
        <!-- /wp:button -->
        <!-- wp:button {"className":"is-style-outline"} -->
        <div class="wp-block-button is-style-outline"><a class="wp-block-button__link">Learn More</a></div>
        <!-- /wp:button -->
    </div>
    <!-- /wp:buttons -->
</div>
<!-- /wp:group -->
"#)
            .build(),
    );

    // Two column text pattern
    registry.register(
        PatternBuilder::new("core/two-column-text", "Two Column Text")
            .description("Two columns of text")
            .categories(vec!["text", "columns"])
            .keywords(vec!["columns", "text", "two"])
            .content(r#"
<!-- wp:columns -->
<div class="wp-block-columns">
    <!-- wp:column -->
    <div class="wp-block-column">
        <!-- wp:heading {"level":3} -->
        <h3>Column One</h3>
        <!-- /wp:heading -->
        <!-- wp:paragraph -->
        <p>Add your content here. This is a two-column layout that works great for comparing items or organizing content.</p>
        <!-- /wp:paragraph -->
    </div>
    <!-- /wp:column -->
    <!-- wp:column -->
    <div class="wp-block-column">
        <!-- wp:heading {"level":3} -->
        <h3>Column Two</h3>
        <!-- /wp:heading -->
        <!-- wp:paragraph -->
        <p>Add your content here. This is a two-column layout that works great for comparing items or organizing content.</p>
        <!-- /wp:paragraph -->
    </div>
    <!-- /wp:column -->
</div>
<!-- /wp:columns -->
"#)
            .build(),
    );

    // Call to action pattern
    registry.register(
        PatternBuilder::new("core/cta-simple", "Simple Call to Action")
            .description("A simple call to action section")
            .categories(vec!["call-to-action"])
            .keywords(vec!["cta", "action", "button"])
            .content(r#"
<!-- wp:group {"align":"wide","style":{"spacing":{"padding":{"top":"var:preset|spacing|50","bottom":"var:preset|spacing|50"}}},"backgroundColor":"primary","textColor":"white"} -->
<div class="wp-block-group alignwide has-white-color has-primary-background-color has-text-color has-background" style="padding-top:var(--wp--preset--spacing--50);padding-bottom:var(--wp--preset--spacing--50)">
    <!-- wp:heading {"textAlign":"center"} -->
    <h2 class="has-text-align-center">Ready to Get Started?</h2>
    <!-- /wp:heading -->
    <!-- wp:paragraph {"align":"center"} -->
    <p class="has-text-align-center">Join thousands of satisfied customers today.</p>
    <!-- /wp:paragraph -->
    <!-- wp:buttons {"layout":{"type":"flex","justifyContent":"center"}} -->
    <div class="wp-block-buttons">
        <!-- wp:button {"backgroundColor":"white","textColor":"primary"} -->
        <div class="wp-block-button"><a class="wp-block-button__link has-primary-color has-white-background-color has-text-color has-background">Start Now</a></div>
        <!-- /wp:button -->
    </div>
    <!-- /wp:buttons -->
</div>
<!-- /wp:group -->
"#)
            .build(),
    );

    // Testimonial pattern
    registry.register(
        PatternBuilder::new("core/testimonial-simple", "Simple Testimonial")
            .description("A simple testimonial with quote and author")
            .categories(vec!["testimonials"])
            .keywords(vec!["testimonial", "quote", "review"])
            .content(r#"
<!-- wp:group {"style":{"spacing":{"padding":{"top":"var:preset|spacing|40","bottom":"var:preset|spacing|40"}}}} -->
<div class="wp-block-group" style="padding-top:var(--wp--preset--spacing--40);padding-bottom:var(--wp--preset--spacing--40)">
    <!-- wp:quote -->
    <blockquote class="wp-block-quote">
        <p>This product has completely transformed how we work. Highly recommended!</p>
        <cite>— Jane Doe, CEO at Company</cite>
    </blockquote>
    <!-- /wp:quote -->
</div>
<!-- /wp:group -->
"#)
            .build(),
    );

    // Pricing table pattern
    registry.register(
        PatternBuilder::new("core/pricing-table", "Pricing Table")
            .description("A three-column pricing table")
            .categories(vec!["pricing"])
            .keywords(vec!["pricing", "plans", "table"])
            .viewport_width(1200)
            .content(r#"
<!-- wp:columns {"align":"wide"} -->
<div class="wp-block-columns alignwide">
    <!-- wp:column -->
    <div class="wp-block-column">
        <!-- wp:group {"style":{"border":{"width":"1px"},"spacing":{"padding":"30px"}}} -->
        <div class="wp-block-group" style="border-width:1px;padding:30px">
            <!-- wp:heading {"textAlign":"center","level":3} -->
            <h3 class="has-text-align-center">Basic</h3>
            <!-- /wp:heading -->
            <!-- wp:paragraph {"align":"center","style":{"typography":{"fontSize":"2.5rem"}}} -->
            <p class="has-text-align-center" style="font-size:2.5rem">$9/mo</p>
            <!-- /wp:paragraph -->
            <!-- wp:list -->
            <ul><li>Feature One</li><li>Feature Two</li><li>Feature Three</li></ul>
            <!-- /wp:list -->
            <!-- wp:buttons {"layout":{"type":"flex","justifyContent":"center"}} -->
            <div class="wp-block-buttons">
                <!-- wp:button {"width":100} -->
                <div class="wp-block-button has-custom-width wp-block-button__width-100"><a class="wp-block-button__link">Choose Plan</a></div>
                <!-- /wp:button -->
            </div>
            <!-- /wp:buttons -->
        </div>
        <!-- /wp:group -->
    </div>
    <!-- /wp:column -->
    <!-- wp:column -->
    <div class="wp-block-column">
        <!-- wp:group {"style":{"border":{"width":"2px"},"spacing":{"padding":"30px"}},"borderColor":"primary"} -->
        <div class="wp-block-group has-border-color has-primary-border-color" style="border-width:2px;padding:30px">
            <!-- wp:heading {"textAlign":"center","level":3} -->
            <h3 class="has-text-align-center">Pro</h3>
            <!-- /wp:heading -->
            <!-- wp:paragraph {"align":"center","style":{"typography":{"fontSize":"2.5rem"}}} -->
            <p class="has-text-align-center" style="font-size:2.5rem">$19/mo</p>
            <!-- /wp:paragraph -->
            <!-- wp:list -->
            <ul><li>Everything in Basic</li><li>Feature Four</li><li>Feature Five</li><li>Priority Support</li></ul>
            <!-- /wp:list -->
            <!-- wp:buttons {"layout":{"type":"flex","justifyContent":"center"}} -->
            <div class="wp-block-buttons">
                <!-- wp:button {"width":100} -->
                <div class="wp-block-button has-custom-width wp-block-button__width-100"><a class="wp-block-button__link">Choose Plan</a></div>
                <!-- /wp:button -->
            </div>
            <!-- /wp:buttons -->
        </div>
        <!-- /wp:group -->
    </div>
    <!-- /wp:column -->
    <!-- wp:column -->
    <div class="wp-block-column">
        <!-- wp:group {"style":{"border":{"width":"1px"},"spacing":{"padding":"30px"}}} -->
        <div class="wp-block-group" style="border-width:1px;padding:30px">
            <!-- wp:heading {"textAlign":"center","level":3} -->
            <h3 class="has-text-align-center">Enterprise</h3>
            <!-- /wp:heading -->
            <!-- wp:paragraph {"align":"center","style":{"typography":{"fontSize":"2.5rem"}}} -->
            <p class="has-text-align-center" style="font-size:2.5rem">Custom</p>
            <!-- /wp:paragraph -->
            <!-- wp:list -->
            <ul><li>Everything in Pro</li><li>Unlimited Users</li><li>Custom Features</li><li>Dedicated Support</li></ul>
            <!-- /wp:list -->
            <!-- wp:buttons {"layout":{"type":"flex","justifyContent":"center"}} -->
            <div class="wp-block-buttons">
                <!-- wp:button {"width":100} -->
                <div class="wp-block-button has-custom-width wp-block-button__width-100"><a class="wp-block-button__link">Contact Us</a></div>
                <!-- /wp:button -->
            </div>
            <!-- /wp:buttons -->
        </div>
        <!-- /wp:group -->
    </div>
    <!-- /wp:column -->
</div>
<!-- /wp:columns -->
"#)
            .build(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_registry() {
        let registry = PatternRegistry::new();

        let pattern = PatternBuilder::new("test/pattern", "Test Pattern")
            .description("A test pattern")
            .categories(vec!["text"])
            .content("<p>Test</p>")
            .build();

        registry.register(pattern);

        let retrieved = registry.get("test/pattern");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Test Pattern");
    }

    #[test]
    fn test_pattern_search() {
        let registry = PatternRegistry::new();
        register_builtin_patterns(&registry);

        let results = registry.search("hero");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_pattern_by_category() {
        let registry = PatternRegistry::new();
        register_builtin_patterns(&registry);

        let results = registry.get_by_category("pricing");
        assert!(!results.is_empty());
    }
}
