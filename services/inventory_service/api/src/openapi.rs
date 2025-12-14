use utoipa::OpenApi;

// Import DTOs for components
use crate::handlers::lot_serial::{
    CreateLotSerialRequest, ListLotSerialsQuery, QuarantineResponse,
};
use inventory_service_core::domains::inventory::dto::picking_method_dto::{
    CreatePickingMethodRequest, PickingMethodResponse, UpdatePickingMethodRequest,
};
use inventory_service_core::dto::common::PaginationInfo;
use inventory_service_core::models::{LotSerial, LotSerialLifecycle};

/// OpenAPI documentation for Inventory Service
#[derive(OpenApi)]
#[openapi(
    components(
        schemas(
            // Common
            PaginationInfo,
            // Lot Serial
            CreateLotSerialRequest,
            LotSerial,
            ListLotSerialsQuery,
            LotSerialLifecycle,
            QuarantineResponse,
            // Picking Method
            CreatePickingMethodRequest,
            PickingMethodResponse,
            UpdatePickingMethodRequest,
        )
    ),
    info(
        title = "Inventory Service API",
        version = "0.1.0",
        description = "Multi-tenant inventory management and warehouse operations service",
        contact(
            name = "Anthill Team",
            email = "team@example.com"
        ),
    ),
    servers(
        (url = "http://localhost:8001", description = "Local development server"),
    ),
)]
pub struct ApiDoc;

/// Export OpenAPI spec to YAML file (only with --features export-spec)
#[cfg(feature = "export-spec")]
#[allow(dead_code)]
pub fn export_spec() -> Result<(), Box<dyn std::error::Error>> {
    use std::path::Path;

    let openapi = ApiDoc::openapi();
    let yaml = serde_yaml::to_string(&openapi).map_err(std::io::Error::other)?;

    let path =
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../shared/openapi/inventory.yaml"));

    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, yaml)?;

    eprintln!("📄 OpenAPI spec exported to {:?}", path);
    Ok(())
}
