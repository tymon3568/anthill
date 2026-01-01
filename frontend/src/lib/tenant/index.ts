/**
 * Tenant Context Utilities
 *
 * This module provides utilities for detecting and managing tenant context
 * in a multi-tenant SaaS application.
 *
 * Tenant detection strategies (in priority order):
 * 1. Subdomain-based: acme.localhost:5173 -> tenant "acme"
 * 2. X-Tenant-ID header (for API calls)
 * 3. Fallback to manual tenant selection
 */

import { browser } from '$app/environment';

// Storage key for persisted tenant slug
const TENANT_STORAGE_KEY = 'anthill_tenant_slug';

/**
 * Parse tenant slug from a hostname/subdomain
 *
 * Examples:
 * - acme.localhost:5173 -> "acme"
 * - acme.anthill.example.com -> "acme"
 * - localhost:5173 -> null
 * - anthill.example.com -> null (no subdomain)
 *
 * @param hostname - The hostname to parse (e.g., from window.location.hostname)
 * @returns The tenant slug or null if not found
 */
export function parseTenantFromHostname(hostname: string): string | null {
	// Remove port if present
	const host = hostname.split(':')[0];

	// Handle localhost specifically
	if (host === 'localhost' || host === '127.0.0.1') {
		return null;
	}

	// Check for subdomain pattern: tenant.domain.tld or tenant.localhost
	// For *.localhost pattern (e.g., acme.localhost)
	if (host.endsWith('.localhost')) {
		const subdomain = host.replace('.localhost', '');
		if (subdomain && subdomain !== 'www') {
			return subdomain;
		}
		return null;
	}

	// For production domains with tenant subdomain (e.g., tenant.anthill.example.com)
	// Require at least 4 parts to avoid false positives with ccTLDs like .co.uk
	// Examples:
	// - tenant.anthill.example.com (4 parts) -> "tenant"
	// - anthill.example.com (3 parts) -> null (app domain, no tenant)
	// - example.co.uk (3 parts) -> null (ccTLD, no tenant)
	const parts = host.split('.');
	if (parts.length >= 4) {
		const subdomain = parts[0];
		// Ignore www subdomain
		if (subdomain && subdomain !== 'www') {
			return subdomain;
		}
	}

	return null;
}

/**
 * Get the current tenant slug from various sources
 *
 * Priority:
 * 1. URL subdomain (browser only)
 * 2. Persisted storage (browser only)
 * 3. null
 *
 * @returns The tenant slug or null
 */
export function getCurrentTenantSlug(): string | null {
	if (!browser) {
		return null;
	}

	// Try subdomain first
	const subdomainTenant = parseTenantFromHostname(window.location.hostname);
	if (subdomainTenant) {
		return subdomainTenant;
	}

	// Fall back to persisted storage
	try {
		const stored = localStorage.getItem(TENANT_STORAGE_KEY);
		if (stored) {
			return stored;
		}
	} catch {
		// localStorage might not be available
	}

	return null;
}

/**
 * Persist tenant slug to local storage
 * This is used when user manually selects a tenant
 *
 * @param slug - The tenant slug to persist
 */
export function setPersistedTenantSlug(slug: string): void {
	if (!browser) return;

	try {
		localStorage.setItem(TENANT_STORAGE_KEY, slug);
	} catch {
		// localStorage might not be available
	}
}

/**
 * Clear persisted tenant slug
 */
export function clearPersistedTenantSlug(): void {
	if (!browser) return;

	try {
		localStorage.removeItem(TENANT_STORAGE_KEY);
	} catch {
		// localStorage might not be available
	}
}

/**
 * Check if tenant context is available
 * @returns true if tenant slug is available from any source
 */
export function hasTenantContext(): boolean {
	return getCurrentTenantSlug() !== null;
}

/**
 * Tenant context object for use in API calls and stores
 */
export interface TenantContext {
	/** Tenant slug (subdomain or stored value) */
	slug: string | null;
	/** Source of the tenant context */
	source: 'subdomain' | 'storage' | 'manual' | null;
	/** Whether tenant context is available */
	hasContext: boolean;
}

/**
 * Get full tenant context information
 */
export function getTenantContext(): TenantContext {
	if (!browser) {
		return { slug: null, source: null, hasContext: false };
	}

	// Check subdomain first
	const subdomainTenant = parseTenantFromHostname(window.location.hostname);
	if (subdomainTenant) {
		return { slug: subdomainTenant, source: 'subdomain', hasContext: true };
	}

	// Check storage
	try {
		const stored = localStorage.getItem(TENANT_STORAGE_KEY);
		if (stored) {
			return { slug: stored, source: 'storage', hasContext: true };
		}
	} catch {
		// localStorage might not be available
	}

	return { slug: null, source: null, hasContext: false };
}
