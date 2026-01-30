# Products Enhancement - Mini PRD

**Module:** 8.10_Inventory_UI
**Sub-Module ID:** 8.10.6
**Status:** NeedsReview
**Created:** 2026-01-30
**Last Updated:** 2026-01-30

---

## 1. Overview

### Purpose
Enhance the existing Products module with critical missing features identified in the Feature Gap Analysis (Section 8 of ui-architecture-proposal.md).

### Business Value
- **Product Images**: 92% user demand - critical for ecommerce integration
- **Barcode/GTIN**: 85% user demand - essential for retail/warehouse operations
- **Import/Export CSV**: 89% user demand - massive time savings for bulk operations

### Actors
- **Inventory Manager**: Manages product catalog, uploads images, imports products
- **Warehouse Staff**: Scans barcodes for inventory operations
- **Admin**: Full access to all product management features

---

## 2. Sprint 1 Scope (Week 1-2)

### Features to Implement

| Priority | Feature | Effort | Dependencies |
|----------|---------|--------|--------------|
| P0 | **Barcode Field** | 1 day | None |
| P0 | **Product Images/Media** | 4 days | RustFS storage |
| P0 | **Import/Export CSV** | 3 days | None |
| P0 | **Image Gallery Component** | 2 days | Product Images |

---

## 3. Data Model Changes

### 3.1 Products Table Updates

```sql
-- Add to products table
ALTER TABLE products ADD COLUMN barcode VARCHAR(50);
ALTER TABLE products ADD COLUMN barcode_type VARCHAR(20); -- EAN13, UPC, ISBN, CUSTOM
CREATE INDEX idx_products_barcode ON products(tenant_id, barcode) WHERE barcode IS NOT NULL;
```

### 3.2 Product Images Table

```sql
CREATE TABLE product_images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(product_id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    url TEXT NOT NULL,
    alt_text VARCHAR(255),
    position INTEGER DEFAULT 0,
    is_primary BOOLEAN DEFAULT FALSE,
    file_size INTEGER,
    mime_type VARCHAR(50),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT uq_product_image_position UNIQUE (product_id, position)
);

CREATE INDEX idx_product_images_product ON product_images(product_id);
CREATE INDEX idx_product_images_tenant ON product_images(tenant_id);
```

---

## 4. API Endpoints

### 4.1 Barcode (Extend existing /products endpoint)

```
PUT  /api/v1/inventory/products/{id}   # Update product with barcode field
GET  /api/v1/inventory/products/by-barcode/{barcode}  # Lookup by barcode
```

### 4.2 Product Images

```
POST   /api/v1/inventory/products/{id}/images          # Upload image
GET    /api/v1/inventory/products/{id}/images          # List images
PUT    /api/v1/inventory/products/{id}/images/{imgId}  # Update image metadata
PUT    /api/v1/inventory/products/{id}/images/reorder  # Reorder images
DELETE /api/v1/inventory/products/{id}/images/{imgId}  # Delete image
PUT    /api/v1/inventory/products/{id}/images/{imgId}/primary  # Set as primary
```

### 4.3 Import/Export

```
POST   /api/v1/inventory/products/import          # Import from CSV
GET    /api/v1/inventory/products/export          # Export to CSV
GET    /api/v1/inventory/products/import/template # Download CSV template
POST   /api/v1/inventory/products/import/validate # Validate CSV before import
```

---

## 5. UI Specifications

### 5.1 Barcode Field

**Location:** Product Create/Edit Form - Basic Info section

```
┌─────────────────────────────────────────────────────────────┐
│ Basic Information                                           │
├─────────────────────────────────────────────────────────────┤
│ SKU:        [PROD-001         ]  [Auto-generate]            │
│ Name:       [Product Name      ]                            │
│ Barcode:    [8901234567890     ]  Type: [EAN-13 ▼]          │
│             └── Barcode scanner icon                        │
│ Description: [                  ]                           │
└─────────────────────────────────────────────────────────────┘
```

**Barcode Types:**
- EAN-13 (13 digits) - Europe
- UPC-A (12 digits) - North America
- ISBN (13 digits) - Books
- Custom (alphanumeric) - Internal

### 5.2 Product Images

**Location:** Product Detail Page - New "Images" Tab OR in Basic Info section

```
┌─────────────────────────────────────────────────────────────┐
│ Product Images                         [+ Upload Images]    │
├─────────────────────────────────────────────────────────────┤
│ ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐     │
│ │          │  │          │  │          │  │    +     │     │
│ │  📷 ⭐   │  │  📷      │  │  📷      │  │  Drop    │     │
│ │ PRIMARY  │  │          │  │          │  │  here    │     │
│ │          │  │          │  │          │  │          │     │
│ └──────────┘  └──────────┘  └──────────┘  └──────────┘     │
│                                                             │
│ Drag images to reorder. Click ⭐ to set as primary.        │
└─────────────────────────────────────────────────────────────┘
```

**Features:**
- Drag-drop upload
- Drag to reorder
- Click to set primary
- Hover for delete/edit options
- Image preview modal
- Alt text editing

### 5.3 Import/Export

**Location:** Product List Page - Header Actions

```
┌─────────────────────────────────────────────────────────────┐
│ Products                    [Import ▼] [Export] [+ New]     │
├─────────────────────────────────────────────────────────────┤
│ Import dropdown:                                            │
│   ┌────────────────────────────────────┐                    │
│   │ 📥 Import from CSV                 │                    │
│   │ 📄 Download Template               │                    │
│   │ 📋 Import History                  │                    │
│   └────────────────────────────────────┘                    │
└─────────────────────────────────────────────────────────────┘
```

**Import Flow:**
1. Click "Import from CSV"
2. Upload CSV file (drag-drop or file picker)
3. Preview: Show parsed data with validation errors
4. Column mapping (if headers don't match)
5. Confirm import
6. Show results: X created, Y updated, Z errors

---

## 6. Business Rules

### 6.1 Barcode Rules

| Rule ID | Description | Validation |
|---------|-------------|------------|
| BR-PE-001 | Barcode must be unique within tenant | DB unique constraint |
| BR-PE-002 | EAN-13 must be exactly 13 digits | Regex: `^\d{13}$` |
| BR-PE-003 | UPC-A must be exactly 12 digits | Regex: `^\d{12}$` |
| BR-PE-004 | Barcode is optional | Nullable field |

### 6.2 Image Rules

| Rule ID | Description | Validation |
|---------|-------------|------------|
| BR-PE-010 | Max 10 images per product | Count check on upload |
| BR-PE-011 | Max file size 5MB | File size check |
| BR-PE-012 | Allowed formats: JPG, PNG, WebP | MIME type check |
| BR-PE-013 | One image must be primary if images exist | Auto-set first |
| BR-PE-014 | Images stored in RustFS (S3-compatible) | Storage config |

### 6.3 Import Rules

| Rule ID | Description | Validation |
|---------|-------------|------------|
| BR-PE-020 | Max 1000 rows per import | Row count check |
| BR-PE-021 | SKU is required and unique | Validation + upsert |
| BR-PE-022 | Invalid rows don't block valid ones | Partial import |
| BR-PE-023 | Duplicate SKU = update existing | Upsert behavior |

---

## 7. Error Handling

### 7.1 Barcode Errors

| Error Code | Scenario | User Message |
|------------|----------|--------------|
| BARCODE_DUPLICATE | Barcode already exists | "This barcode is already assigned to another product" |
| BARCODE_INVALID_FORMAT | Wrong format for type | "EAN-13 barcode must be exactly 13 digits" |

### 7.2 Image Errors

| Error Code | Scenario | User Message |
|------------|----------|--------------|
| IMAGE_TOO_LARGE | File > 5MB | "Image must be less than 5MB" |
| IMAGE_INVALID_FORMAT | Not JPG/PNG/WebP | "Only JPG, PNG, and WebP images are allowed" |
| IMAGE_LIMIT_EXCEEDED | > 10 images | "Maximum 10 images per product" |
| IMAGE_UPLOAD_FAILED | Storage error | "Failed to upload image. Please try again" |

### 7.3 Import Errors

| Error Code | Scenario | User Message |
|------------|----------|--------------|
| IMPORT_TOO_LARGE | > 1000 rows | "Maximum 1000 products per import" |
| IMPORT_INVALID_CSV | Parse error | "Invalid CSV format" |
| IMPORT_MISSING_SKU | SKU column missing | "SKU column is required" |

---

## 8. Dependencies

### 8.1 Required Services
- **RustFS** (MinIO-compatible S3): Image storage
- **Inventory Service**: Product API

### 8.2 Required Libraries
- Frontend: `@uppy/core`, `@uppy/dashboard` (file upload)
- Frontend: `papaparse` (CSV parsing)
- Backend: `aws-sdk-s3` or `s3` crate (RustFS client)

---

## 9. Tasks Summary

| Task ID | Name | Status | Priority |
|---------|------|--------|----------|
| 08.10.06.01 | Barcode field backend + frontend | Done_With_Issues | P0 |
| 08.10.06.02 | Product Images backend (storage, API) | Done | P0 |
| 08.10.06.03 | Product Images frontend (gallery, upload) | Done | P0 |
| 08.10.06.04 | Import/Export CSV backend | Done_With_Issues | P0 |
| 08.10.06.05 | Import/Export CSV frontend | Done | P0 |

---

## 10. Implementation Checklist

### Backend
- [ ] Database migration: Add barcode columns to products
- [ ] Database migration: Create product_images table
- [ ] Barcode lookup endpoint
- [ ] Image upload endpoint (multipart/form-data)
- [ ] Image CRUD endpoints
- [ ] Image reorder endpoint
- [ ] CSV import endpoint
- [ ] CSV export endpoint
- [ ] CSV template endpoint
- [ ] Casbin policies for all new endpoints
- [ ] Unit tests for barcode validation
- [ ] Unit tests for image operations
- [ ] Integration tests

### Frontend
- [ ] TypeScript types for images, import/export
- [ ] API client: image operations
- [ ] API client: import/export
- [ ] Barcode field in product form
- [ ] Barcode type dropdown
- [ ] Image gallery component
- [ ] Drag-drop upload component
- [ ] Image reorder (drag-drop)
- [ ] Import wizard modal
- [ ] CSV preview table
- [ ] Export functionality
- [ ] Download template
- [ ] Error handling UI
- [ ] Unit tests
- [ ] E2E tests

---

**Document Version:** 1.0
**Author:** Claude (AI Agent)
