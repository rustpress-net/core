//! E-commerce analytics models

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::DateRange;

/// E-commerce overview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcommerceOverview {
    pub date_range: DateRange,
    pub revenue: f64,
    pub transactions: u64,
    pub average_order_value: f64,
    pub ecommerce_conversion_rate: f64,
    pub quantity: u64,
    pub unique_purchases: u64,
    pub avg_qty_per_transaction: f64,
    pub per_session_value: f64,
    pub revenue_trend: Vec<RevenueTrendData>,
    pub comparison: Option<EcommerceComparison>,
}

/// Revenue trend data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueTrendData {
    pub date: NaiveDate,
    pub revenue: f64,
    pub transactions: u64,
    pub average_order_value: f64,
}

/// E-commerce comparison data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcommerceComparison {
    pub revenue_change: f64,
    pub transactions_change: f64,
    pub average_order_value_change: f64,
    pub conversion_rate_change: f64,
    pub quantity_change: f64,
}

/// Product performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductPerformanceData {
    pub date_range: DateRange,
    pub products: Vec<ProductData>,
    pub product_categories: Vec<ProductCategoryData>,
    pub product_skus: Vec<ProductSkuData>,
    pub top_revenue_products: Vec<ProductData>,
    pub top_quantity_products: Vec<ProductData>,
}

/// Individual product data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductData {
    pub product_id: String,
    pub product_name: String,
    pub product_sku: Option<String>,
    pub product_category: Option<String>,
    pub product_brand: Option<String>,
    pub product_variant: Option<String>,
    pub quantity: u64,
    pub unique_purchases: u64,
    pub product_revenue: f64,
    pub avg_price: f64,
    pub avg_qty_per_transaction: f64,
    pub product_refund_amount: f64,
    pub cart_to_detail_rate: f64,
    pub buy_to_detail_rate: f64,
    pub percentage_of_revenue: f64,
}

/// Product category data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductCategoryData {
    pub category: String,
    pub quantity: u64,
    pub unique_purchases: u64,
    pub product_revenue: f64,
    pub avg_price: f64,
    pub percentage_of_revenue: f64,
}

/// Product SKU data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductSkuData {
    pub product_sku: String,
    pub product_name: String,
    pub quantity: u64,
    pub unique_purchases: u64,
    pub product_revenue: f64,
    pub avg_price: f64,
}

/// Sales performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesPerformanceData {
    pub date_range: DateRange,
    pub by_date: Vec<DailySalesData>,
    pub by_transaction_id: Vec<TransactionSummary>,
    pub by_source: Vec<SalesBySource>,
    pub by_keyword: Vec<SalesByKeyword>,
    pub by_campaign: Vec<SalesByCampaign>,
}

/// Daily sales data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySalesData {
    pub date: NaiveDate,
    pub revenue: f64,
    pub transactions: u64,
    pub average_order_value: f64,
    pub shipping: f64,
    pub tax: f64,
    pub quantity: u64,
}

/// Transaction summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub transaction_id: String,
    pub date: NaiveDate,
    pub revenue: f64,
    pub shipping: f64,
    pub tax: f64,
    pub quantity: u64,
    pub source: String,
    pub medium: String,
}

/// Sales by source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesBySource {
    pub source: String,
    pub revenue: f64,
    pub transactions: u64,
    pub average_order_value: f64,
    pub ecommerce_conversion_rate: f64,
    pub per_session_value: f64,
}

/// Sales by keyword
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesByKeyword {
    pub keyword: String,
    pub revenue: f64,
    pub transactions: u64,
    pub average_order_value: f64,
    pub ecommerce_conversion_rate: f64,
}

/// Sales by campaign
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesByCampaign {
    pub campaign: String,
    pub revenue: f64,
    pub transactions: u64,
    pub average_order_value: f64,
    pub ecommerce_conversion_rate: f64,
    pub cost: Option<f64>,
    pub roas: Option<f64>,
}

/// Transaction details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDetails {
    pub date_range: DateRange,
    pub transactions: Vec<TransactionDetail>,
    pub total_revenue: f64,
    pub total_transactions: u64,
    pub avg_transaction_value: f64,
}

/// Individual transaction detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDetail {
    pub transaction_id: String,
    pub transaction_date: NaiveDate,
    pub transaction_time: Option<String>,
    pub revenue: f64,
    pub tax: f64,
    pub shipping: f64,
    pub quantity: u64,
    pub affiliation: Option<String>,
    pub coupon: Option<String>,
    pub items: Vec<TransactionItem>,
    pub source: String,
    pub medium: String,
    pub campaign: Option<String>,
    pub user_type: String,
}

/// Transaction item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionItem {
    pub product_id: String,
    pub product_name: String,
    pub product_sku: Option<String>,
    pub product_category: Option<String>,
    pub product_variant: Option<String>,
    pub price: f64,
    pub quantity: u64,
    pub item_revenue: f64,
}

/// Time to purchase data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeToPurchaseData {
    pub date_range: DateRange,
    pub days_to_transaction: Vec<DaysToTransactionBucket>,
    pub sessions_to_transaction: Vec<SessionsToTransactionBucket>,
    pub avg_days_to_purchase: f64,
    pub avg_sessions_to_purchase: f64,
}

/// Days to transaction bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaysToTransactionBucket {
    pub days: String,
    pub transactions: u64,
    pub revenue: f64,
    pub percentage: f64,
}

/// Sessions to transaction bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionsToTransactionBucket {
    pub sessions: String,
    pub transactions: u64,
    pub revenue: f64,
    pub percentage: f64,
}

/// Enhanced e-commerce data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedEcommerceData {
    pub date_range: DateRange,
    pub shopping_behavior: ShoppingBehaviorData,
    pub checkout_behavior: CheckoutBehaviorData,
    pub product_list_performance: Vec<ProductListData>,
    pub internal_promotion: Vec<InternalPromotionData>,
    pub order_coupon: Vec<OrderCouponData>,
    pub product_coupon: Vec<ProductCouponData>,
    pub affiliate_code: Vec<AffiliateCodeData>,
}

/// Shopping behavior analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingBehaviorData {
    pub all_sessions: u64,
    pub sessions_with_product_views: u64,
    pub sessions_with_add_to_cart: u64,
    pub sessions_with_checkout: u64,
    pub sessions_with_transactions: u64,
    pub product_view_rate: f64,
    pub add_to_cart_rate: f64,
    pub cart_to_checkout_rate: f64,
    pub checkout_to_purchase_rate: f64,
    pub overall_conversion_rate: f64,
    pub funnel_data: Vec<ShoppingFunnelStep>,
}

/// Shopping funnel step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingFunnelStep {
    pub step_name: String,
    pub sessions: u64,
    pub continuation_rate: f64,
    pub abandonment_rate: f64,
}

/// Checkout behavior analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutBehaviorData {
    pub checkout_steps: Vec<CheckoutStep>,
    pub overall_abandonment_rate: f64,
    pub funnel_visualization: CheckoutFunnelVisualization,
}

/// Checkout step data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutStep {
    pub step_number: u32,
    pub step_name: String,
    pub sessions: u64,
    pub abandonment_rate: f64,
    pub continuation_rate: f64,
    pub step_option: Option<String>,
}

/// Checkout funnel visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutFunnelVisualization {
    pub steps: Vec<CheckoutFunnelVisStep>,
    pub drop_off_analysis: Vec<CheckoutDropOff>,
}

/// Checkout funnel visualization step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutFunnelVisStep {
    pub step_number: u32,
    pub step_name: String,
    pub sessions: u64,
    pub percentage: f64,
    pub options: Vec<CheckoutOption>,
}

/// Checkout option within a step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutOption {
    pub option: String,
    pub sessions: u64,
    pub percentage: f64,
}

/// Checkout drop-off analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutDropOff {
    pub from_step: u32,
    pub drop_offs: u64,
    pub top_exit_pages: Vec<ExitPageFromCheckout>,
}

/// Exit page from checkout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitPageFromCheckout {
    pub page: String,
    pub exits: u64,
    pub percentage: f64,
}

/// Product list performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductListData {
    pub product_list_name: String,
    pub product_list_views: u64,
    pub product_list_clicks: u64,
    pub product_list_ctr: f64,
    pub product_adds_to_cart: u64,
    pub product_checkouts: u64,
    pub unique_purchases: u64,
    pub product_revenue: f64,
}

/// Internal promotion data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalPromotionData {
    pub promotion_id: String,
    pub promotion_name: String,
    pub promotion_creative: Option<String>,
    pub promotion_position: Option<String>,
    pub internal_promotion_views: u64,
    pub internal_promotion_clicks: u64,
    pub internal_promotion_ctr: f64,
}

/// Order coupon data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCouponData {
    pub order_coupon_code: String,
    pub transactions: u64,
    pub revenue: f64,
    pub avg_order_value: f64,
    pub avg_discount: f64,
}

/// Product coupon data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductCouponData {
    pub product_coupon_code: String,
    pub product_name: String,
    pub quantity: u64,
    pub product_revenue: f64,
    pub avg_discount: f64,
}

/// Affiliate code data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffiliateCodeData {
    pub affiliate_code: String,
    pub transactions: u64,
    pub revenue: f64,
    pub avg_order_value: f64,
    pub commission: Option<f64>,
}

/// Product detail views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductDetailViewsData {
    pub date_range: DateRange,
    pub products: Vec<ProductDetailView>,
    pub trend: Vec<ProductDetailViewTrend>,
}

/// Product detail view data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductDetailView {
    pub product_id: String,
    pub product_name: String,
    pub product_detail_views: u64,
    pub product_adds_to_cart: u64,
    pub product_removes_from_cart: u64,
    pub product_checkouts: u64,
    pub unique_purchases: u64,
    pub cart_to_detail_rate: f64,
    pub buy_to_detail_rate: f64,
}

/// Product detail view trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductDetailViewTrend {
    pub date: NaiveDate,
    pub product_detail_views: u64,
    pub cart_to_detail_rate: f64,
}

/// Cart analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartAnalysisData {
    pub date_range: DateRange,
    pub add_to_cart_count: u64,
    pub remove_from_cart_count: u64,
    pub cart_abandonment_rate: f64,
    pub avg_cart_value: f64,
    pub top_added_products: Vec<CartProductData>,
    pub top_removed_products: Vec<CartProductData>,
    pub cart_trend: Vec<CartTrendData>,
}

/// Cart product data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartProductData {
    pub product_id: String,
    pub product_name: String,
    pub count: u64,
    pub value: f64,
}

/// Cart trend data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartTrendData {
    pub date: NaiveDate,
    pub adds_to_cart: u64,
    pub removes_from_cart: u64,
    pub cart_abandonment_rate: f64,
}

/// Refund data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundData {
    pub date_range: DateRange,
    pub total_refunds: u64,
    pub total_refund_amount: f64,
    pub refund_rate: f64,
    pub refunds_by_product: Vec<ProductRefundData>,
    pub refunds_by_reason: Vec<RefundReasonData>,
    pub refund_trend: Vec<RefundTrendData>,
}

/// Product refund data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductRefundData {
    pub product_id: String,
    pub product_name: String,
    pub refund_quantity: u64,
    pub refund_amount: f64,
    pub refund_rate: f64,
}

/// Refund reason data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundReasonData {
    pub reason: String,
    pub count: u64,
    pub amount: f64,
    pub percentage: f64,
}

/// Refund trend data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundTrendData {
    pub date: NaiveDate,
    pub refunds: u64,
    pub refund_amount: f64,
}

/// Customer value analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerValueData {
    pub date_range: DateRange,
    pub avg_customer_value: f64,
    pub customer_segments: Vec<CustomerSegment>,
    pub value_distribution: Vec<ValueDistributionBucket>,
    pub top_customers: Vec<TopCustomerData>,
}

/// Customer segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerSegment {
    pub segment_name: String,
    pub customers: u64,
    pub revenue: f64,
    pub avg_order_value: f64,
    pub orders_per_customer: f64,
    pub percentage_of_revenue: f64,
}

/// Value distribution bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueDistributionBucket {
    pub value_range: String,
    pub customers: u64,
    pub revenue: f64,
    pub percentage: f64,
}

/// Top customer data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopCustomerData {
    pub customer_id: String,
    pub total_revenue: f64,
    pub transactions: u64,
    pub avg_order_value: f64,
    pub first_purchase: NaiveDate,
    pub last_purchase: NaiveDate,
}
