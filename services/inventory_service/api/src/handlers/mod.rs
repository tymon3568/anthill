pub mod category;
pub mod cycle_count;
pub mod delivery;
pub mod health;
pub mod lot_serial;
pub mod picking;
pub mod products;
pub mod putaway;
pub mod quality;
pub mod receipt;
pub mod reconciliation;
pub mod replenishment;
pub mod reports;
pub mod rma;
pub mod scrap;
pub mod search;
pub mod stock_take;
pub mod transfer;
pub mod valuation;
pub mod warehouses;

// Re-export handlers for OpenAPI
pub use cycle_count::{
    cancel_session, close_session, create_cycle_count, create_cycle_count_routes, generate_lines,
    get_cycle_count, list_cycle_counts, reconcile, skip_lines, submit_counts,
};
pub use health::health_check;
pub use lot_serial::{
    create_lot_serial, delete_lot_serial, get_lot_serial, get_lot_serial_lifecycle,
    list_lot_serials_by_product, quarantine_expired_lots, update_lot_serial,
};
pub use picking::{
    confirm_picking_plan, create_picking_method, delete_picking_method, get_picking_method,
    list_picking_methods, optimize_picking, set_default_method, update_picking_method,
};
pub use products::{create_product, delete_product, get_product, list_products, update_product};
pub use scrap::{
    add_scrap_lines, cancel_scrap, create_scrap, create_scrap_routes, get_scrap, list_scraps,
    post_scrap,
};
