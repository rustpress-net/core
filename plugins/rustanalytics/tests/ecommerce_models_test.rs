//! Tests for e-commerce analytics models
//!
//! This module contains comprehensive tests for all e-commerce model types
//! including serialization, deserialization, and edge cases.

use chrono::NaiveDate;
use rustanalytics::models::ecommerce::*;
use rustanalytics::models::DateRange;

// ============================================================================
// Helper Functions
// ============================================================================

fn sample_date_range() -> DateRange {
    DateRange {
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
    }
}

fn sample_revenue_trend_data() -> RevenueTrendData {
    RevenueTrendData {
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        revenue: 15000.0,
        transactions: 150,
        average_order_value: 100.0,
    }
}

fn sample_ecommerce_comparison() -> EcommerceComparison {
    EcommerceComparison {
        revenue_change: 25.5,
        transactions_change: 18.0,
        average_order_value_change: 6.5,
        conversion_rate_change: 0.8,
        quantity_change: 22.0,
    }
}

fn sample_ecommerce_overview() -> EcommerceOverview {
    EcommerceOverview {
        date_range: sample_date_range(),
        revenue: 150000.0,
        transactions: 1500,
        average_order_value: 100.0,
        ecommerce_conversion_rate: 3.5,
        quantity: 4500,
        unique_purchases: 3800,
        avg_qty_per_transaction: 3.0,
        per_session_value: 5.0,
        revenue_trend: vec![sample_revenue_trend_data()],
        comparison: Some(sample_ecommerce_comparison()),
    }
}

fn sample_product_data() -> ProductData {
    ProductData {
        product_id: "PROD-001".to_string(),
        product_name: "Rust Programming Book".to_string(),
        product_sku: Some("SKU-001".to_string()),
        product_category: Some("Books/Programming".to_string()),
        product_brand: Some("TechPress".to_string()),
        product_variant: Some("Hardcover".to_string()),
        quantity: 500,
        unique_purchases: 450,
        product_revenue: 24950.0,
        avg_price: 49.90,
        avg_qty_per_transaction: 1.1,
        product_refund_amount: 499.0,
        cart_to_detail_rate: 25.0,
        buy_to_detail_rate: 15.0,
        percentage_of_revenue: 16.6,
    }
}

fn sample_product_category_data() -> ProductCategoryData {
    ProductCategoryData {
        category: "Books/Programming".to_string(),
        quantity: 2000,
        unique_purchases: 1800,
        product_revenue: 89900.0,
        avg_price: 44.95,
        percentage_of_revenue: 60.0,
    }
}

fn sample_product_sku_data() -> ProductSkuData {
    ProductSkuData {
        product_sku: "SKU-001".to_string(),
        product_name: "Rust Programming Book".to_string(),
        quantity: 500,
        unique_purchases: 450,
        product_revenue: 24950.0,
        avg_price: 49.90,
    }
}

fn sample_daily_sales_data() -> DailySalesData {
    DailySalesData {
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        revenue: 5000.0,
        transactions: 50,
        average_order_value: 100.0,
        shipping: 250.0,
        tax: 400.0,
        quantity: 150,
    }
}

fn sample_transaction_summary() -> TransactionSummary {
    TransactionSummary {
        transaction_id: "TXN-12345".to_string(),
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        revenue: 250.0,
        shipping: 10.0,
        tax: 20.0,
        quantity: 3,
        source: "google".to_string(),
        medium: "organic".to_string(),
    }
}

fn sample_sales_by_source() -> SalesBySource {
    SalesBySource {
        source: "google".to_string(),
        revenue: 50000.0,
        transactions: 500,
        average_order_value: 100.0,
        ecommerce_conversion_rate: 4.0,
        per_session_value: 6.0,
    }
}

fn sample_sales_by_keyword() -> SalesByKeyword {
    SalesByKeyword {
        keyword: "rust programming book".to_string(),
        revenue: 15000.0,
        transactions: 150,
        average_order_value: 100.0,
        ecommerce_conversion_rate: 5.0,
    }
}

fn sample_sales_by_campaign() -> SalesByCampaign {
    SalesByCampaign {
        campaign: "summer_sale_2024".to_string(),
        revenue: 25000.0,
        transactions: 250,
        average_order_value: 100.0,
        ecommerce_conversion_rate: 4.5,
        cost: Some(5000.0),
        roas: Some(5.0),
    }
}

fn sample_transaction_item() -> TransactionItem {
    TransactionItem {
        product_id: "PROD-001".to_string(),
        product_name: "Rust Programming Book".to_string(),
        product_sku: Some("SKU-001".to_string()),
        product_category: Some("Books".to_string()),
        product_variant: Some("Hardcover".to_string()),
        price: 49.90,
        quantity: 2,
        item_revenue: 99.80,
    }
}

fn sample_transaction_detail() -> TransactionDetail {
    TransactionDetail {
        transaction_id: "TXN-12345".to_string(),
        transaction_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        transaction_time: Some("14:30:00".to_string()),
        revenue: 250.0,
        tax: 20.0,
        shipping: 10.0,
        quantity: 3,
        affiliation: Some("Main Store".to_string()),
        coupon: Some("SAVE10".to_string()),
        items: vec![sample_transaction_item()],
        source: "google".to_string(),
        medium: "organic".to_string(),
        campaign: Some("brand_campaign".to_string()),
        user_type: "Returning Visitor".to_string(),
    }
}

fn sample_days_to_transaction_bucket() -> DaysToTransactionBucket {
    DaysToTransactionBucket {
        days: "0".to_string(),
        transactions: 500,
        revenue: 50000.0,
        percentage: 33.3,
    }
}

fn sample_sessions_to_transaction_bucket() -> SessionsToTransactionBucket {
    SessionsToTransactionBucket {
        sessions: "1".to_string(),
        transactions: 400,
        revenue: 40000.0,
        percentage: 26.7,
    }
}

fn sample_shopping_funnel_step() -> ShoppingFunnelStep {
    ShoppingFunnelStep {
        step_name: "Product View".to_string(),
        sessions: 10000,
        continuation_rate: 30.0,
        abandonment_rate: 70.0,
    }
}

fn sample_shopping_behavior_data() -> ShoppingBehaviorData {
    ShoppingBehaviorData {
        all_sessions: 50000,
        sessions_with_product_views: 15000,
        sessions_with_add_to_cart: 5000,
        sessions_with_checkout: 2000,
        sessions_with_transactions: 1500,
        product_view_rate: 30.0,
        add_to_cart_rate: 33.3,
        cart_to_checkout_rate: 40.0,
        checkout_to_purchase_rate: 75.0,
        overall_conversion_rate: 3.0,
        funnel_data: vec![sample_shopping_funnel_step()],
    }
}

fn sample_checkout_option() -> CheckoutOption {
    CheckoutOption {
        option: "Credit Card".to_string(),
        sessions: 800,
        percentage: 53.3,
    }
}

fn sample_checkout_step() -> CheckoutStep {
    CheckoutStep {
        step_number: 1,
        step_name: "Shipping Info".to_string(),
        sessions: 2000,
        abandonment_rate: 20.0,
        continuation_rate: 80.0,
        step_option: Some("Standard Shipping".to_string()),
    }
}

fn sample_exit_page_from_checkout() -> ExitPageFromCheckout {
    ExitPageFromCheckout {
        page: "/cart".to_string(),
        exits: 150,
        percentage: 30.0,
    }
}

fn sample_checkout_drop_off() -> CheckoutDropOff {
    CheckoutDropOff {
        from_step: 2,
        drop_offs: 500,
        top_exit_pages: vec![sample_exit_page_from_checkout()],
    }
}

fn sample_checkout_funnel_vis_step() -> CheckoutFunnelVisStep {
    CheckoutFunnelVisStep {
        step_number: 1,
        step_name: "Shipping Info".to_string(),
        sessions: 2000,
        percentage: 100.0,
        options: vec![sample_checkout_option()],
    }
}

fn sample_checkout_funnel_visualization() -> CheckoutFunnelVisualization {
    CheckoutFunnelVisualization {
        steps: vec![sample_checkout_funnel_vis_step()],
        drop_off_analysis: vec![sample_checkout_drop_off()],
    }
}

fn sample_checkout_behavior_data() -> CheckoutBehaviorData {
    CheckoutBehaviorData {
        checkout_steps: vec![sample_checkout_step()],
        overall_abandonment_rate: 25.0,
        funnel_visualization: sample_checkout_funnel_visualization(),
    }
}

fn sample_product_list_data() -> ProductListData {
    ProductListData {
        product_list_name: "Featured Products".to_string(),
        product_list_views: 50000,
        product_list_clicks: 5000,
        product_list_ctr: 10.0,
        product_adds_to_cart: 1500,
        product_checkouts: 500,
        unique_purchases: 400,
        product_revenue: 20000.0,
    }
}

fn sample_internal_promotion_data() -> InternalPromotionData {
    InternalPromotionData {
        promotion_id: "PROMO-001".to_string(),
        promotion_name: "Summer Sale Banner".to_string(),
        promotion_creative: Some("banner_large.jpg".to_string()),
        promotion_position: Some("homepage_hero".to_string()),
        internal_promotion_views: 100000,
        internal_promotion_clicks: 5000,
        internal_promotion_ctr: 5.0,
    }
}

fn sample_order_coupon_data() -> OrderCouponData {
    OrderCouponData {
        order_coupon_code: "SAVE20".to_string(),
        transactions: 200,
        revenue: 16000.0,
        avg_order_value: 80.0,
        avg_discount: 20.0,
    }
}

fn sample_product_coupon_data() -> ProductCouponData {
    ProductCouponData {
        product_coupon_code: "BOOK10".to_string(),
        product_name: "Rust Programming Book".to_string(),
        quantity: 100,
        product_revenue: 4490.0,
        avg_discount: 10.0,
    }
}

fn sample_affiliate_code_data() -> AffiliateCodeData {
    AffiliateCodeData {
        affiliate_code: "AFF-12345".to_string(),
        transactions: 50,
        revenue: 5000.0,
        avg_order_value: 100.0,
        commission: Some(500.0),
    }
}

fn sample_product_detail_view() -> ProductDetailView {
    ProductDetailView {
        product_id: "PROD-001".to_string(),
        product_name: "Rust Programming Book".to_string(),
        product_detail_views: 5000,
        product_adds_to_cart: 1250,
        product_removes_from_cart: 200,
        product_checkouts: 400,
        unique_purchases: 350,
        cart_to_detail_rate: 25.0,
        buy_to_detail_rate: 7.0,
    }
}

fn sample_product_detail_view_trend() -> ProductDetailViewTrend {
    ProductDetailViewTrend {
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        product_detail_views: 500,
        cart_to_detail_rate: 26.0,
    }
}

fn sample_cart_product_data() -> CartProductData {
    CartProductData {
        product_id: "PROD-001".to_string(),
        product_name: "Rust Programming Book".to_string(),
        count: 500,
        value: 24950.0,
    }
}

fn sample_cart_trend_data() -> CartTrendData {
    CartTrendData {
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        adds_to_cart: 200,
        removes_from_cart: 30,
        cart_abandonment_rate: 65.0,
    }
}

fn sample_product_refund_data() -> ProductRefundData {
    ProductRefundData {
        product_id: "PROD-002".to_string(),
        product_name: "Defective Widget".to_string(),
        refund_quantity: 50,
        refund_amount: 2500.0,
        refund_rate: 5.0,
    }
}

fn sample_refund_reason_data() -> RefundReasonData {
    RefundReasonData {
        reason: "Product Defect".to_string(),
        count: 30,
        amount: 1500.0,
        percentage: 40.0,
    }
}

fn sample_refund_trend_data() -> RefundTrendData {
    RefundTrendData {
        date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        refunds: 10,
        refund_amount: 500.0,
    }
}

fn sample_customer_segment() -> CustomerSegment {
    CustomerSegment {
        segment_name: "High Value".to_string(),
        customers: 500,
        revenue: 75000.0,
        avg_order_value: 150.0,
        orders_per_customer: 3.0,
        percentage_of_revenue: 50.0,
    }
}

fn sample_value_distribution_bucket() -> ValueDistributionBucket {
    ValueDistributionBucket {
        value_range: "$100-$500".to_string(),
        customers: 1000,
        revenue: 50000.0,
        percentage: 33.3,
    }
}

fn sample_top_customer_data() -> TopCustomerData {
    TopCustomerData {
        customer_id: "CUST-001".to_string(),
        total_revenue: 5000.0,
        transactions: 10,
        avg_order_value: 500.0,
        first_purchase: NaiveDate::from_ymd_opt(2023, 1, 15).unwrap(),
        last_purchase: NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
    }
}

// ============================================================================
// EcommerceOverview Tests
// ============================================================================

#[test]
fn test_ecommerce_overview_creation() {
    let overview = sample_ecommerce_overview();
    assert_eq!(overview.revenue, 150000.0);
    assert_eq!(overview.transactions, 1500);
    assert!(overview.comparison.is_some());
}

#[test]
fn test_ecommerce_overview_without_comparison() {
    let overview = EcommerceOverview {
        date_range: sample_date_range(),
        revenue: 100000.0,
        transactions: 1000,
        average_order_value: 100.0,
        ecommerce_conversion_rate: 3.0,
        quantity: 3000,
        unique_purchases: 2500,
        avg_qty_per_transaction: 3.0,
        per_session_value: 4.5,
        revenue_trend: vec![],
        comparison: None,
    };
    assert!(overview.comparison.is_none());
}

#[test]
fn test_ecommerce_overview_serialization() {
    let overview = sample_ecommerce_overview();
    let json = serde_json::to_string(&overview).unwrap();
    assert!(json.contains("\"revenue\":150000"));
    assert!(json.contains("\"transactions\":1500"));
}

#[test]
fn test_ecommerce_overview_roundtrip() {
    let overview = sample_ecommerce_overview();
    let json = serde_json::to_string(&overview).unwrap();
    let deserialized: EcommerceOverview = serde_json::from_str(&json).unwrap();
    assert_eq!(overview.revenue, deserialized.revenue);
}

// ============================================================================
// RevenueTrendData Tests
// ============================================================================

#[test]
fn test_revenue_trend_data_creation() {
    let trend = sample_revenue_trend_data();
    assert_eq!(trend.revenue, 15000.0);
    assert_eq!(trend.transactions, 150);
}

#[test]
fn test_revenue_trend_data_serialization() {
    let trend = sample_revenue_trend_data();
    let json = serde_json::to_string(&trend).unwrap();
    assert!(json.contains("\"date\":\"2024-01-15\""));
    assert!(json.contains("\"revenue\":15000"));
}

// ============================================================================
// EcommerceComparison Tests
// ============================================================================

#[test]
fn test_ecommerce_comparison_creation() {
    let comparison = sample_ecommerce_comparison();
    assert!((comparison.revenue_change - 25.5).abs() < f64::EPSILON);
}

#[test]
fn test_ecommerce_comparison_negative_values() {
    let comparison = EcommerceComparison {
        revenue_change: -15.0,
        transactions_change: -10.0,
        average_order_value_change: -5.0,
        conversion_rate_change: -0.5,
        quantity_change: -12.0,
    };
    assert!(comparison.revenue_change < 0.0);
}

#[test]
fn test_ecommerce_comparison_serialization() {
    let comparison = sample_ecommerce_comparison();
    let json = serde_json::to_string(&comparison).unwrap();
    assert!(json.contains("revenue_change"));
}

// ============================================================================
// ProductData Tests
// ============================================================================

#[test]
fn test_product_data_creation() {
    let product = sample_product_data();
    assert_eq!(product.product_id, "PROD-001");
    assert_eq!(product.product_name, "Rust Programming Book");
    assert!(product.product_sku.is_some());
}

#[test]
fn test_product_data_without_optional_fields() {
    let product = ProductData {
        product_id: "PROD-002".to_string(),
        product_name: "Simple Product".to_string(),
        product_sku: None,
        product_category: None,
        product_brand: None,
        product_variant: None,
        quantity: 100,
        unique_purchases: 90,
        product_revenue: 5000.0,
        avg_price: 50.0,
        avg_qty_per_transaction: 1.1,
        product_refund_amount: 0.0,
        cart_to_detail_rate: 20.0,
        buy_to_detail_rate: 10.0,
        percentage_of_revenue: 5.0,
    };
    assert!(product.product_sku.is_none());
    assert!(product.product_category.is_none());
}

#[test]
fn test_product_data_serialization() {
    let product = sample_product_data();
    let json = serde_json::to_string(&product).unwrap();
    assert!(json.contains("\"product_id\":\"PROD-001\""));
    assert!(json.contains("\"product_name\":\"Rust Programming Book\""));
}

// ============================================================================
// ProductPerformanceData Tests
// ============================================================================

#[test]
fn test_product_performance_data_creation() {
    let data = ProductPerformanceData {
        date_range: sample_date_range(),
        products: vec![sample_product_data()],
        product_categories: vec![sample_product_category_data()],
        product_skus: vec![sample_product_sku_data()],
        top_revenue_products: vec![sample_product_data()],
        top_quantity_products: vec![sample_product_data()],
    };
    assert!(!data.products.is_empty());
}

#[test]
fn test_product_performance_data_serialization() {
    let data = ProductPerformanceData {
        date_range: sample_date_range(),
        products: vec![],
        product_categories: vec![],
        product_skus: vec![],
        top_revenue_products: vec![],
        top_quantity_products: vec![],
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"products\":[]"));
}

// ============================================================================
// ProductCategoryData Tests
// ============================================================================

#[test]
fn test_product_category_data_creation() {
    let category = sample_product_category_data();
    assert_eq!(category.category, "Books/Programming");
}

#[test]
fn test_product_category_data_serialization() {
    let category = sample_product_category_data();
    let json = serde_json::to_string(&category).unwrap();
    assert!(json.contains("\"category\":\"Books/Programming\""));
}

// ============================================================================
// ProductSkuData Tests
// ============================================================================

#[test]
fn test_product_sku_data_creation() {
    let sku = sample_product_sku_data();
    assert_eq!(sku.product_sku, "SKU-001");
}

#[test]
fn test_product_sku_data_serialization() {
    let sku = sample_product_sku_data();
    let json = serde_json::to_string(&sku).unwrap();
    assert!(json.contains("\"product_sku\":\"SKU-001\""));
}

// ============================================================================
// SalesPerformanceData Tests
// ============================================================================

#[test]
fn test_sales_performance_data_creation() {
    let data = SalesPerformanceData {
        date_range: sample_date_range(),
        by_date: vec![sample_daily_sales_data()],
        by_transaction_id: vec![sample_transaction_summary()],
        by_source: vec![sample_sales_by_source()],
        by_keyword: vec![sample_sales_by_keyword()],
        by_campaign: vec![sample_sales_by_campaign()],
    };
    assert!(!data.by_date.is_empty());
}

#[test]
fn test_sales_performance_data_serialization() {
    let data = SalesPerformanceData {
        date_range: sample_date_range(),
        by_date: vec![],
        by_transaction_id: vec![],
        by_source: vec![],
        by_keyword: vec![],
        by_campaign: vec![],
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"by_date\":[]"));
}

// ============================================================================
// DailySalesData Tests
// ============================================================================

#[test]
fn test_daily_sales_data_creation() {
    let sales = sample_daily_sales_data();
    assert_eq!(sales.revenue, 5000.0);
    assert_eq!(sales.transactions, 50);
}

#[test]
fn test_daily_sales_data_serialization() {
    let sales = sample_daily_sales_data();
    let json = serde_json::to_string(&sales).unwrap();
    assert!(json.contains("\"date\":\"2024-01-15\""));
}

// ============================================================================
// TransactionSummary Tests
// ============================================================================

#[test]
fn test_transaction_summary_creation() {
    let summary = sample_transaction_summary();
    assert_eq!(summary.transaction_id, "TXN-12345");
}

#[test]
fn test_transaction_summary_serialization() {
    let summary = sample_transaction_summary();
    let json = serde_json::to_string(&summary).unwrap();
    assert!(json.contains("\"transaction_id\":\"TXN-12345\""));
}

// ============================================================================
// SalesBySource Tests
// ============================================================================

#[test]
fn test_sales_by_source_creation() {
    let sales = sample_sales_by_source();
    assert_eq!(sales.source, "google");
}

#[test]
fn test_sales_by_source_serialization() {
    let sales = sample_sales_by_source();
    let json = serde_json::to_string(&sales).unwrap();
    assert!(json.contains("\"source\":\"google\""));
}

// ============================================================================
// SalesByKeyword Tests
// ============================================================================

#[test]
fn test_sales_by_keyword_creation() {
    let sales = sample_sales_by_keyword();
    assert_eq!(sales.keyword, "rust programming book");
}

#[test]
fn test_sales_by_keyword_serialization() {
    let sales = sample_sales_by_keyword();
    let json = serde_json::to_string(&sales).unwrap();
    assert!(json.contains("\"keyword\":\"rust programming book\""));
}

// ============================================================================
// SalesByCampaign Tests
// ============================================================================

#[test]
fn test_sales_by_campaign_creation() {
    let sales = sample_sales_by_campaign();
    assert_eq!(sales.campaign, "summer_sale_2024");
    assert!(sales.cost.is_some());
    assert!(sales.roas.is_some());
}

#[test]
fn test_sales_by_campaign_without_cost() {
    let sales = SalesByCampaign {
        campaign: "organic_campaign".to_string(),
        revenue: 10000.0,
        transactions: 100,
        average_order_value: 100.0,
        ecommerce_conversion_rate: 3.0,
        cost: None,
        roas: None,
    };
    assert!(sales.cost.is_none());
}

#[test]
fn test_sales_by_campaign_serialization() {
    let sales = sample_sales_by_campaign();
    let json = serde_json::to_string(&sales).unwrap();
    assert!(json.contains("\"campaign\":\"summer_sale_2024\""));
}

// ============================================================================
// TransactionDetails Tests
// ============================================================================

#[test]
fn test_transaction_details_creation() {
    let details = TransactionDetails {
        date_range: sample_date_range(),
        transactions: vec![sample_transaction_detail()],
        total_revenue: 150000.0,
        total_transactions: 1500,
        avg_transaction_value: 100.0,
    };
    assert!(!details.transactions.is_empty());
}

#[test]
fn test_transaction_details_serialization() {
    let details = TransactionDetails {
        date_range: sample_date_range(),
        transactions: vec![],
        total_revenue: 0.0,
        total_transactions: 0,
        avg_transaction_value: 0.0,
    };
    let json = serde_json::to_string(&details).unwrap();
    assert!(json.contains("\"transactions\":[]"));
}

// ============================================================================
// TransactionDetail Tests
// ============================================================================

#[test]
fn test_transaction_detail_creation() {
    let detail = sample_transaction_detail();
    assert_eq!(detail.transaction_id, "TXN-12345");
    assert!(detail.affiliation.is_some());
    assert!(detail.coupon.is_some());
}

#[test]
fn test_transaction_detail_without_optional_fields() {
    let detail = TransactionDetail {
        transaction_id: "TXN-99999".to_string(),
        transaction_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        transaction_time: None,
        revenue: 100.0,
        tax: 8.0,
        shipping: 5.0,
        quantity: 1,
        affiliation: None,
        coupon: None,
        items: vec![],
        source: "direct".to_string(),
        medium: "(none)".to_string(),
        campaign: None,
        user_type: "New Visitor".to_string(),
    };
    assert!(detail.transaction_time.is_none());
    assert!(detail.affiliation.is_none());
}

#[test]
fn test_transaction_detail_serialization() {
    let detail = sample_transaction_detail();
    let json = serde_json::to_string(&detail).unwrap();
    assert!(json.contains("\"transaction_id\":\"TXN-12345\""));
}

// ============================================================================
// TransactionItem Tests
// ============================================================================

#[test]
fn test_transaction_item_creation() {
    let item = sample_transaction_item();
    assert_eq!(item.product_id, "PROD-001");
    assert_eq!(item.quantity, 2);
}

#[test]
fn test_transaction_item_serialization() {
    let item = sample_transaction_item();
    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("\"product_id\":\"PROD-001\""));
}

// ============================================================================
// TimeToPurchaseData Tests
// ============================================================================

#[test]
fn test_time_to_purchase_data_creation() {
    let data = TimeToPurchaseData {
        date_range: sample_date_range(),
        days_to_transaction: vec![sample_days_to_transaction_bucket()],
        sessions_to_transaction: vec![sample_sessions_to_transaction_bucket()],
        avg_days_to_purchase: 2.5,
        avg_sessions_to_purchase: 3.2,
    };
    assert!(!data.days_to_transaction.is_empty());
}

#[test]
fn test_time_to_purchase_data_serialization() {
    let data = TimeToPurchaseData {
        date_range: sample_date_range(),
        days_to_transaction: vec![],
        sessions_to_transaction: vec![],
        avg_days_to_purchase: 0.0,
        avg_sessions_to_purchase: 0.0,
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"avg_days_to_purchase\":0"));
}

// ============================================================================
// DaysToTransactionBucket Tests
// ============================================================================

#[test]
fn test_days_to_transaction_bucket_creation() {
    let bucket = sample_days_to_transaction_bucket();
    assert_eq!(bucket.days, "0");
    assert_eq!(bucket.transactions, 500);
}

#[test]
fn test_days_to_transaction_bucket_various_ranges() {
    let ranges = vec!["0", "1", "2", "3-7", "8-14", "15-30", "31+"];
    for range in ranges {
        let bucket = DaysToTransactionBucket {
            days: range.to_string(),
            transactions: 100,
            revenue: 10000.0,
            percentage: 10.0,
        };
        assert_eq!(bucket.days, range);
    }
}

#[test]
fn test_days_to_transaction_bucket_serialization() {
    let bucket = sample_days_to_transaction_bucket();
    let json = serde_json::to_string(&bucket).unwrap();
    assert!(json.contains("\"days\":\"0\""));
}

// ============================================================================
// SessionsToTransactionBucket Tests
// ============================================================================

#[test]
fn test_sessions_to_transaction_bucket_creation() {
    let bucket = sample_sessions_to_transaction_bucket();
    assert_eq!(bucket.sessions, "1");
}

#[test]
fn test_sessions_to_transaction_bucket_serialization() {
    let bucket = sample_sessions_to_transaction_bucket();
    let json = serde_json::to_string(&bucket).unwrap();
    assert!(json.contains("\"sessions\":\"1\""));
}

// ============================================================================
// EnhancedEcommerceData Tests
// ============================================================================

#[test]
fn test_enhanced_ecommerce_data_creation() {
    let data = EnhancedEcommerceData {
        date_range: sample_date_range(),
        shopping_behavior: sample_shopping_behavior_data(),
        checkout_behavior: sample_checkout_behavior_data(),
        product_list_performance: vec![sample_product_list_data()],
        internal_promotion: vec![sample_internal_promotion_data()],
        order_coupon: vec![sample_order_coupon_data()],
        product_coupon: vec![sample_product_coupon_data()],
        affiliate_code: vec![sample_affiliate_code_data()],
    };
    assert!(!data.product_list_performance.is_empty());
}

#[test]
fn test_enhanced_ecommerce_data_serialization() {
    let data = EnhancedEcommerceData {
        date_range: sample_date_range(),
        shopping_behavior: sample_shopping_behavior_data(),
        checkout_behavior: sample_checkout_behavior_data(),
        product_list_performance: vec![],
        internal_promotion: vec![],
        order_coupon: vec![],
        product_coupon: vec![],
        affiliate_code: vec![],
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"product_list_performance\":[]"));
}

// ============================================================================
// ShoppingBehaviorData Tests
// ============================================================================

#[test]
fn test_shopping_behavior_data_creation() {
    let data = sample_shopping_behavior_data();
    assert_eq!(data.all_sessions, 50000);
    assert_eq!(data.overall_conversion_rate, 3.0);
}

#[test]
fn test_shopping_behavior_data_serialization() {
    let data = sample_shopping_behavior_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"all_sessions\":50000"));
}

// ============================================================================
// ShoppingFunnelStep Tests
// ============================================================================

#[test]
fn test_shopping_funnel_step_creation() {
    let step = sample_shopping_funnel_step();
    assert_eq!(step.step_name, "Product View");
}

#[test]
fn test_shopping_funnel_step_serialization() {
    let step = sample_shopping_funnel_step();
    let json = serde_json::to_string(&step).unwrap();
    assert!(json.contains("\"step_name\":\"Product View\""));
}

// ============================================================================
// CheckoutBehaviorData Tests
// ============================================================================

#[test]
fn test_checkout_behavior_data_creation() {
    let data = sample_checkout_behavior_data();
    assert!(!data.checkout_steps.is_empty());
}

#[test]
fn test_checkout_behavior_data_serialization() {
    let data = sample_checkout_behavior_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"overall_abandonment_rate\":25"));
}

// ============================================================================
// CheckoutStep Tests
// ============================================================================

#[test]
fn test_checkout_step_creation() {
    let step = sample_checkout_step();
    assert_eq!(step.step_number, 1);
    assert_eq!(step.step_name, "Shipping Info");
}

#[test]
fn test_checkout_step_without_option() {
    let step = CheckoutStep {
        step_number: 2,
        step_name: "Review Order".to_string(),
        sessions: 1500,
        abandonment_rate: 10.0,
        continuation_rate: 90.0,
        step_option: None,
    };
    assert!(step.step_option.is_none());
}

#[test]
fn test_checkout_step_serialization() {
    let step = sample_checkout_step();
    let json = serde_json::to_string(&step).unwrap();
    assert!(json.contains("\"step_number\":1"));
}

// ============================================================================
// CheckoutFunnelVisualization Tests
// ============================================================================

#[test]
fn test_checkout_funnel_visualization_creation() {
    let viz = sample_checkout_funnel_visualization();
    assert!(!viz.steps.is_empty());
    assert!(!viz.drop_off_analysis.is_empty());
}

#[test]
fn test_checkout_funnel_visualization_serialization() {
    let viz = sample_checkout_funnel_visualization();
    let json = serde_json::to_string(&viz).unwrap();
    assert!(json.contains("\"steps\""));
}

// ============================================================================
// CheckoutFunnelVisStep Tests
// ============================================================================

#[test]
fn test_checkout_funnel_vis_step_creation() {
    let step = sample_checkout_funnel_vis_step();
    assert_eq!(step.step_number, 1);
    assert!(!step.options.is_empty());
}

#[test]
fn test_checkout_funnel_vis_step_serialization() {
    let step = sample_checkout_funnel_vis_step();
    let json = serde_json::to_string(&step).unwrap();
    assert!(json.contains("\"step_name\":\"Shipping Info\""));
}

// ============================================================================
// CheckoutOption Tests
// ============================================================================

#[test]
fn test_checkout_option_creation() {
    let option = sample_checkout_option();
    assert_eq!(option.option, "Credit Card");
}

#[test]
fn test_checkout_option_serialization() {
    let option = sample_checkout_option();
    let json = serde_json::to_string(&option).unwrap();
    assert!(json.contains("\"option\":\"Credit Card\""));
}

// ============================================================================
// CheckoutDropOff Tests
// ============================================================================

#[test]
fn test_checkout_drop_off_creation() {
    let drop_off = sample_checkout_drop_off();
    assert_eq!(drop_off.from_step, 2);
    assert!(!drop_off.top_exit_pages.is_empty());
}

#[test]
fn test_checkout_drop_off_serialization() {
    let drop_off = sample_checkout_drop_off();
    let json = serde_json::to_string(&drop_off).unwrap();
    assert!(json.contains("\"from_step\":2"));
}

// ============================================================================
// ExitPageFromCheckout Tests
// ============================================================================

#[test]
fn test_exit_page_from_checkout_creation() {
    let exit = sample_exit_page_from_checkout();
    assert_eq!(exit.page, "/cart");
}

#[test]
fn test_exit_page_from_checkout_serialization() {
    let exit = sample_exit_page_from_checkout();
    let json = serde_json::to_string(&exit).unwrap();
    assert!(json.contains("\"page\":\"/cart\""));
}

// ============================================================================
// ProductListData Tests
// ============================================================================

#[test]
fn test_product_list_data_creation() {
    let data = sample_product_list_data();
    assert_eq!(data.product_list_name, "Featured Products");
}

#[test]
fn test_product_list_data_serialization() {
    let data = sample_product_list_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"product_list_name\":\"Featured Products\""));
}

// ============================================================================
// InternalPromotionData Tests
// ============================================================================

#[test]
fn test_internal_promotion_data_creation() {
    let promo = sample_internal_promotion_data();
    assert_eq!(promo.promotion_id, "PROMO-001");
    assert!(promo.promotion_creative.is_some());
}

#[test]
fn test_internal_promotion_data_without_optional() {
    let promo = InternalPromotionData {
        promotion_id: "PROMO-002".to_string(),
        promotion_name: "Simple Promo".to_string(),
        promotion_creative: None,
        promotion_position: None,
        internal_promotion_views: 50000,
        internal_promotion_clicks: 2500,
        internal_promotion_ctr: 5.0,
    };
    assert!(promo.promotion_creative.is_none());
}

#[test]
fn test_internal_promotion_data_serialization() {
    let promo = sample_internal_promotion_data();
    let json = serde_json::to_string(&promo).unwrap();
    assert!(json.contains("\"promotion_id\":\"PROMO-001\""));
}

// ============================================================================
// OrderCouponData Tests
// ============================================================================

#[test]
fn test_order_coupon_data_creation() {
    let coupon = sample_order_coupon_data();
    assert_eq!(coupon.order_coupon_code, "SAVE20");
}

#[test]
fn test_order_coupon_data_serialization() {
    let coupon = sample_order_coupon_data();
    let json = serde_json::to_string(&coupon).unwrap();
    assert!(json.contains("\"order_coupon_code\":\"SAVE20\""));
}

// ============================================================================
// ProductCouponData Tests
// ============================================================================

#[test]
fn test_product_coupon_data_creation() {
    let coupon = sample_product_coupon_data();
    assert_eq!(coupon.product_coupon_code, "BOOK10");
}

#[test]
fn test_product_coupon_data_serialization() {
    let coupon = sample_product_coupon_data();
    let json = serde_json::to_string(&coupon).unwrap();
    assert!(json.contains("\"product_coupon_code\":\"BOOK10\""));
}

// ============================================================================
// AffiliateCodeData Tests
// ============================================================================

#[test]
fn test_affiliate_code_data_creation() {
    let affiliate = sample_affiliate_code_data();
    assert_eq!(affiliate.affiliate_code, "AFF-12345");
    assert!(affiliate.commission.is_some());
}

#[test]
fn test_affiliate_code_data_without_commission() {
    let affiliate = AffiliateCodeData {
        affiliate_code: "AFF-99999".to_string(),
        transactions: 10,
        revenue: 1000.0,
        avg_order_value: 100.0,
        commission: None,
    };
    assert!(affiliate.commission.is_none());
}

#[test]
fn test_affiliate_code_data_serialization() {
    let affiliate = sample_affiliate_code_data();
    let json = serde_json::to_string(&affiliate).unwrap();
    assert!(json.contains("\"affiliate_code\":\"AFF-12345\""));
}

// ============================================================================
// ProductDetailViewsData Tests
// ============================================================================

#[test]
fn test_product_detail_views_data_creation() {
    let data = ProductDetailViewsData {
        date_range: sample_date_range(),
        products: vec![sample_product_detail_view()],
        trend: vec![sample_product_detail_view_trend()],
    };
    assert!(!data.products.is_empty());
}

#[test]
fn test_product_detail_views_data_serialization() {
    let data = ProductDetailViewsData {
        date_range: sample_date_range(),
        products: vec![],
        trend: vec![],
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"products\":[]"));
}

// ============================================================================
// ProductDetailView Tests
// ============================================================================

#[test]
fn test_product_detail_view_creation() {
    let view = sample_product_detail_view();
    assert_eq!(view.product_id, "PROD-001");
    assert_eq!(view.product_detail_views, 5000);
}

#[test]
fn test_product_detail_view_serialization() {
    let view = sample_product_detail_view();
    let json = serde_json::to_string(&view).unwrap();
    assert!(json.contains("\"product_detail_views\":5000"));
}

// ============================================================================
// ProductDetailViewTrend Tests
// ============================================================================

#[test]
fn test_product_detail_view_trend_creation() {
    let trend = sample_product_detail_view_trend();
    assert_eq!(trend.product_detail_views, 500);
}

#[test]
fn test_product_detail_view_trend_serialization() {
    let trend = sample_product_detail_view_trend();
    let json = serde_json::to_string(&trend).unwrap();
    assert!(json.contains("\"date\":\"2024-01-15\""));
}

// ============================================================================
// CartAnalysisData Tests
// ============================================================================

#[test]
fn test_cart_analysis_data_creation() {
    let data = CartAnalysisData {
        date_range: sample_date_range(),
        add_to_cart_count: 5000,
        remove_from_cart_count: 500,
        cart_abandonment_rate: 70.0,
        avg_cart_value: 150.0,
        top_added_products: vec![sample_cart_product_data()],
        top_removed_products: vec![sample_cart_product_data()],
        cart_trend: vec![sample_cart_trend_data()],
    };
    assert_eq!(data.add_to_cart_count, 5000);
}

#[test]
fn test_cart_analysis_data_serialization() {
    let data = CartAnalysisData {
        date_range: sample_date_range(),
        add_to_cart_count: 0,
        remove_from_cart_count: 0,
        cart_abandonment_rate: 0.0,
        avg_cart_value: 0.0,
        top_added_products: vec![],
        top_removed_products: vec![],
        cart_trend: vec![],
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"add_to_cart_count\":0"));
}

// ============================================================================
// CartProductData Tests
// ============================================================================

#[test]
fn test_cart_product_data_creation() {
    let data = sample_cart_product_data();
    assert_eq!(data.product_id, "PROD-001");
}

#[test]
fn test_cart_product_data_serialization() {
    let data = sample_cart_product_data();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"product_id\":\"PROD-001\""));
}

// ============================================================================
// CartTrendData Tests
// ============================================================================

#[test]
fn test_cart_trend_data_creation() {
    let trend = sample_cart_trend_data();
    assert_eq!(trend.adds_to_cart, 200);
}

#[test]
fn test_cart_trend_data_serialization() {
    let trend = sample_cart_trend_data();
    let json = serde_json::to_string(&trend).unwrap();
    assert!(json.contains("\"adds_to_cart\":200"));
}

// ============================================================================
// RefundData Tests
// ============================================================================

#[test]
fn test_refund_data_creation() {
    let data = RefundData {
        date_range: sample_date_range(),
        total_refunds: 100,
        total_refund_amount: 5000.0,
        refund_rate: 2.0,
        refunds_by_product: vec![sample_product_refund_data()],
        refunds_by_reason: vec![sample_refund_reason_data()],
        refund_trend: vec![sample_refund_trend_data()],
    };
    assert_eq!(data.total_refunds, 100);
}

#[test]
fn test_refund_data_serialization() {
    let data = RefundData {
        date_range: sample_date_range(),
        total_refunds: 0,
        total_refund_amount: 0.0,
        refund_rate: 0.0,
        refunds_by_product: vec![],
        refunds_by_reason: vec![],
        refund_trend: vec![],
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"total_refunds\":0"));
}

// ============================================================================
// ProductRefundData Tests
// ============================================================================

#[test]
fn test_product_refund_data_creation() {
    let refund = sample_product_refund_data();
    assert_eq!(refund.product_id, "PROD-002");
}

#[test]
fn test_product_refund_data_serialization() {
    let refund = sample_product_refund_data();
    let json = serde_json::to_string(&refund).unwrap();
    assert!(json.contains("\"product_id\":\"PROD-002\""));
}

// ============================================================================
// RefundReasonData Tests
// ============================================================================

#[test]
fn test_refund_reason_data_creation() {
    let reason = sample_refund_reason_data();
    assert_eq!(reason.reason, "Product Defect");
}

#[test]
fn test_refund_reason_data_serialization() {
    let reason = sample_refund_reason_data();
    let json = serde_json::to_string(&reason).unwrap();
    assert!(json.contains("\"reason\":\"Product Defect\""));
}

// ============================================================================
// RefundTrendData Tests
// ============================================================================

#[test]
fn test_refund_trend_data_creation() {
    let trend = sample_refund_trend_data();
    assert_eq!(trend.refunds, 10);
}

#[test]
fn test_refund_trend_data_serialization() {
    let trend = sample_refund_trend_data();
    let json = serde_json::to_string(&trend).unwrap();
    assert!(json.contains("\"refunds\":10"));
}

// ============================================================================
// CustomerValueData Tests
// ============================================================================

#[test]
fn test_customer_value_data_creation() {
    let data = CustomerValueData {
        date_range: sample_date_range(),
        avg_customer_value: 250.0,
        customer_segments: vec![sample_customer_segment()],
        value_distribution: vec![sample_value_distribution_bucket()],
        top_customers: vec![sample_top_customer_data()],
    };
    assert!(!data.customer_segments.is_empty());
}

#[test]
fn test_customer_value_data_serialization() {
    let data = CustomerValueData {
        date_range: sample_date_range(),
        avg_customer_value: 0.0,
        customer_segments: vec![],
        value_distribution: vec![],
        top_customers: vec![],
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"avg_customer_value\":0"));
}

// ============================================================================
// CustomerSegment Tests
// ============================================================================

#[test]
fn test_customer_segment_creation() {
    let segment = sample_customer_segment();
    assert_eq!(segment.segment_name, "High Value");
}

#[test]
fn test_customer_segment_serialization() {
    let segment = sample_customer_segment();
    let json = serde_json::to_string(&segment).unwrap();
    assert!(json.contains("\"segment_name\":\"High Value\""));
}

// ============================================================================
// ValueDistributionBucket Tests
// ============================================================================

#[test]
fn test_value_distribution_bucket_creation() {
    let bucket = sample_value_distribution_bucket();
    assert_eq!(bucket.value_range, "$100-$500");
}

#[test]
fn test_value_distribution_bucket_serialization() {
    let bucket = sample_value_distribution_bucket();
    let json = serde_json::to_string(&bucket).unwrap();
    assert!(json.contains("\"value_range\":\"$100-$500\""));
}

// ============================================================================
// TopCustomerData Tests
// ============================================================================

#[test]
fn test_top_customer_data_creation() {
    let customer = sample_top_customer_data();
    assert_eq!(customer.customer_id, "CUST-001");
    assert_eq!(customer.total_revenue, 5000.0);
}

#[test]
fn test_top_customer_data_serialization() {
    let customer = sample_top_customer_data();
    let json = serde_json::to_string(&customer).unwrap();
    assert!(json.contains("\"customer_id\":\"CUST-001\""));
}

// ============================================================================
// Edge Cases and Complex Scenarios
// ============================================================================

#[test]
fn test_unicode_in_product_names() {
    let product = ProductData {
        product_id: "PROD-JP-001".to_string(),
        product_name: "プログラミング入門書".to_string(),
        product_sku: None,
        product_category: Some("書籍/技術".to_string()),
        product_brand: Some("技術出版".to_string()),
        product_variant: None,
        quantity: 100,
        unique_purchases: 90,
        product_revenue: 5000.0,
        avg_price: 50.0,
        avg_qty_per_transaction: 1.1,
        product_refund_amount: 0.0,
        cart_to_detail_rate: 20.0,
        buy_to_detail_rate: 10.0,
        percentage_of_revenue: 3.0,
    };
    let json = serde_json::to_string(&product).unwrap();
    let deserialized: ProductData = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.product_name, "プログラミング入門書");
}

#[test]
fn test_empty_string_fields() {
    let product = ProductData {
        product_id: "".to_string(),
        product_name: "".to_string(),
        product_sku: Some("".to_string()),
        product_category: None,
        product_brand: None,
        product_variant: None,
        quantity: 0,
        unique_purchases: 0,
        product_revenue: 0.0,
        avg_price: 0.0,
        avg_qty_per_transaction: 0.0,
        product_refund_amount: 0.0,
        cart_to_detail_rate: 0.0,
        buy_to_detail_rate: 0.0,
        percentage_of_revenue: 0.0,
    };
    let json = serde_json::to_string(&product).unwrap();
    assert!(json.contains("\"product_id\":\"\""));
}

#[test]
fn test_large_values() {
    let overview = EcommerceOverview {
        date_range: sample_date_range(),
        revenue: 1_000_000_000.0,
        transactions: u64::MAX / 2,
        average_order_value: 1000000.0,
        ecommerce_conversion_rate: 99.99,
        quantity: u64::MAX / 4,
        unique_purchases: u64::MAX / 8,
        avg_qty_per_transaction: 1000.0,
        per_session_value: 100000.0,
        revenue_trend: vec![],
        comparison: None,
    };
    let json = serde_json::to_string(&overview).unwrap();
    let deserialized: EcommerceOverview = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.transactions, u64::MAX / 2);
}

#[test]
fn test_floating_point_precision() {
    let comparison = EcommerceComparison {
        revenue_change: 33.333333333333336,
        transactions_change: 16.666666666666668,
        average_order_value_change: -0.00000001,
        conversion_rate_change: 0.00000001,
        quantity_change: 99.999999999999,
    };
    let json = serde_json::to_string(&comparison).unwrap();
    let deserialized: EcommerceComparison = serde_json::from_str(&json).unwrap();
    assert!((deserialized.revenue_change - 33.333333333333336).abs() < 1e-10);
}

#[test]
fn test_complex_checkout_funnel() {
    let steps: Vec<CheckoutStep> = (1..=5).map(|i| {
        CheckoutStep {
            step_number: i,
            step_name: format!("Step {}", i),
            sessions: 2000 - (i as u64 * 300),
            abandonment_rate: i as f64 * 5.0,
            continuation_rate: 100.0 - (i as f64 * 5.0),
            step_option: Some(format!("Option {}", i)),
        }
    }).collect();

    let checkout = CheckoutBehaviorData {
        checkout_steps: steps,
        overall_abandonment_rate: 25.0,
        funnel_visualization: sample_checkout_funnel_visualization(),
    };

    assert_eq!(checkout.checkout_steps.len(), 5);
}

#[test]
fn test_transaction_with_many_items() {
    let items: Vec<TransactionItem> = (1..=20).map(|i| {
        TransactionItem {
            product_id: format!("PROD-{:03}", i),
            product_name: format!("Product {}", i),
            product_sku: Some(format!("SKU-{:03}", i)),
            product_category: Some("Category".to_string()),
            product_variant: None,
            price: i as f64 * 10.0,
            quantity: i as u64,
            item_revenue: i as f64 * 10.0 * i as f64,
        }
    }).collect();

    let transaction = TransactionDetail {
        transaction_id: "TXN-LARGE".to_string(),
        transaction_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        transaction_time: Some("12:00:00".to_string()),
        revenue: 2100.0,
        tax: 168.0,
        shipping: 50.0,
        quantity: 210,
        affiliation: Some("Main Store".to_string()),
        coupon: None,
        items,
        source: "google".to_string(),
        medium: "cpc".to_string(),
        campaign: Some("big_sale".to_string()),
        user_type: "New Visitor".to_string(),
    };

    assert_eq!(transaction.items.len(), 20);
}

#[test]
fn test_special_characters_in_coupon_codes() {
    let coupon = OrderCouponData {
        order_coupon_code: "SAVE-20%OFF!".to_string(),
        transactions: 100,
        revenue: 8000.0,
        avg_order_value: 80.0,
        avg_discount: 20.0,
    };
    let json = serde_json::to_string(&coupon).unwrap();
    let deserialized: OrderCouponData = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.order_coupon_code, "SAVE-20%OFF!");
}

#[test]
fn test_clone_all_major_types() {
    let overview = sample_ecommerce_overview();
    let cloned = overview.clone();
    assert_eq!(overview.revenue, cloned.revenue);

    let product = sample_product_data();
    let cloned_product = product.clone();
    assert_eq!(product.product_id, cloned_product.product_id);

    let transaction = sample_transaction_detail();
    let cloned_transaction = transaction.clone();
    assert_eq!(transaction.transaction_id, cloned_transaction.transaction_id);
}

#[test]
fn test_debug_trait_implementation() {
    let overview = sample_ecommerce_overview();
    let debug_str = format!("{:?}", overview);
    assert!(debug_str.contains("EcommerceOverview"));

    let product = sample_product_data();
    let debug_str = format!("{:?}", product);
    assert!(debug_str.contains("ProductData"));
}

#[test]
fn test_many_products() {
    let products: Vec<ProductData> = (1..=100).map(|i| {
        ProductData {
            product_id: format!("PROD-{:04}", i),
            product_name: format!("Product {}", i),
            product_sku: Some(format!("SKU-{:04}", i)),
            product_category: Some("Category".to_string()),
            product_brand: Some("Brand".to_string()),
            product_variant: None,
            quantity: (100 - i + 1) as u64 * 10,
            unique_purchases: (100 - i + 1) as u64 * 9,
            product_revenue: (100 - i + 1) as f64 * 500.0,
            avg_price: 50.0,
            avg_qty_per_transaction: 1.1,
            product_refund_amount: 0.0,
            cart_to_detail_rate: 25.0,
            buy_to_detail_rate: 10.0,
            percentage_of_revenue: 1.0,
        }
    }).collect();

    let performance = ProductPerformanceData {
        date_range: sample_date_range(),
        products,
        product_categories: vec![],
        product_skus: vec![],
        top_revenue_products: vec![],
        top_quantity_products: vec![],
    };

    assert_eq!(performance.products.len(), 100);
}

#[test]
fn test_zero_values() {
    let comparison = EcommerceComparison {
        revenue_change: 0.0,
        transactions_change: 0.0,
        average_order_value_change: 0.0,
        conversion_rate_change: 0.0,
        quantity_change: 0.0,
    };

    let json = serde_json::to_string(&comparison).unwrap();
    assert!(json.contains(":0"));
}

#[test]
fn test_customer_lifecycle() {
    let customer = TopCustomerData {
        customer_id: "CUST-LOYAL".to_string(),
        total_revenue: 50000.0,
        transactions: 100,
        avg_order_value: 500.0,
        first_purchase: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        last_purchase: NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
    };

    // Customer has been active for about 4 years
    let days_between = customer.last_purchase.signed_duration_since(customer.first_purchase).num_days();
    assert!(days_between > 1400); // More than ~4 years
}
