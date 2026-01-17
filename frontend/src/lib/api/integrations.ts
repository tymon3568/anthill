import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';

// Integration types
export interface Integration {
	id: string;
	name: string;
	type: 'marketplace' | 'shipping' | 'payment' | 'erp' | 'custom';
	provider: string;
	status: 'active' | 'inactive' | 'error' | 'pending';
	lastSyncAt?: string;
	config: Record<string, unknown>;
	createdAt: string;
	updatedAt: string;
}

export interface SyncLog {
	id: string;
	integrationId: string;
	type: 'import' | 'export' | 'sync';
	status: 'success' | 'partial' | 'failed';
	recordsProcessed: number;
	recordsFailed: number;
	errorMessage?: string;
	startedAt: string;
	completedAt: string;
}

export interface Webhook {
	id: string;
	name: string;
	url: string;
	events: string[];
	status: 'active' | 'inactive';
	secret?: string;
	lastTriggeredAt?: string;
	createdAt: string;
}

// Integrations API client
export const integrationsApi = {
	async list(): Promise<ApiResponse<Integration[]>> {
		return apiClient.get<Integration[]>('/integrations');
	},

	async get(id: string): Promise<ApiResponse<Integration>> {
		return apiClient.get<Integration>(`/integrations/${id}`);
	},

	async sync(id: string): Promise<ApiResponse<SyncLog>> {
		return apiClient.post<SyncLog>(`/integrations/${id}/sync`, {});
	},

	async getSyncLogs(id: string): Promise<ApiResponse<SyncLog[]>> {
		return apiClient.get<SyncLog[]>(`/integrations/${id}/logs`);
	},

	async listWebhooks(): Promise<ApiResponse<Webhook[]>> {
		return apiClient.get<Webhook[]>('/webhooks');
	},

	async createWebhook(data: Partial<Webhook>): Promise<ApiResponse<Webhook>> {
		return apiClient.post<Webhook>('/webhooks', data as Record<string, unknown>);
	}
};

// Mock data
export const mockIntegrations: Integration[] = [
	{
		id: '1',
		name: 'Shopify Store',
		type: 'marketplace',
		provider: 'shopify',
		status: 'active',
		lastSyncAt: '2026-01-17T09:00:00Z',
		config: { storeUrl: 'mystore.myshopify.com' },
		createdAt: '2025-12-01T10:00:00Z',
		updatedAt: '2026-01-17T09:00:00Z'
	},
	{
		id: '2',
		name: 'Amazon Marketplace',
		type: 'marketplace',
		provider: 'amazon',
		status: 'active',
		lastSyncAt: '2026-01-17T08:30:00Z',
		config: { region: 'us-east-1' },
		createdAt: '2025-12-15T14:00:00Z',
		updatedAt: '2026-01-17T08:30:00Z'
	},
	{
		id: '3',
		name: 'Stripe Payments',
		type: 'payment',
		provider: 'stripe',
		status: 'active',
		config: {},
		createdAt: '2025-11-20T09:00:00Z',
		updatedAt: '2026-01-10T12:00:00Z'
	}
];
