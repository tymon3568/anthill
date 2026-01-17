import { browser } from '$app/environment';
import { PUBLIC_API_BASE_URL } from '$env/static/public';

type EventHandler<T> = (data: T) => void;
type ErrorHandler = (error: Event) => void;

interface RealtimeConfig {
	url: string;
	reconnectInterval?: number;
	maxReconnectAttempts?: number;
}

/**
 * Real-time updates manager using Server-Sent Events (SSE)
 * SSE is preferred over WebSocket for one-way server-to-client updates
 */
export class RealtimeManager<T = unknown> {
	private eventSource: EventSource | null = null;
	private handlers: Map<string, Set<EventHandler<T>>> = new Map();
	private errorHandlers: Set<ErrorHandler> = new Set();
	private reconnectAttempts = 0;
	private reconnectTimeout: ReturnType<typeof setTimeout> | null = null;

	constructor(private config: RealtimeConfig) {
		this.config.reconnectInterval = config.reconnectInterval ?? 5000;
		this.config.maxReconnectAttempts = config.maxReconnectAttempts ?? 10;
	}

	/**
	 * Attach event listeners for all registered event types
	 */
	private attachEventListeners(): void {
		if (!this.eventSource) return;

		// Attach listeners for all registered event types
		for (const eventType of this.handlers.keys()) {
			this.eventSource.addEventListener(eventType, (event: MessageEvent) => {
				try {
					const data = JSON.parse(event.data) as T;
					this.notifyHandlers(eventType, data);
				} catch (e) {
					console.error(`Failed to parse ${eventType} event:`, e);
				}
			});
		}
	}

	/**
	 * Connect to SSE endpoint
	 */
	connect(): void {
		if (!browser) return;
		if (this.eventSource?.readyState === EventSource.OPEN) return;

		try {
			this.eventSource = new EventSource(this.config.url, { withCredentials: true });

			this.eventSource.onopen = () => {
				console.log('SSE connection established');
				this.reconnectAttempts = 0;
			};

			this.eventSource.onerror = (error) => {
				console.error('SSE connection error:', error);
				this.errorHandlers.forEach((handler) => handler(error));
				this.handleReconnect();
			};

			this.eventSource.onmessage = (event) => {
				try {
					const data = JSON.parse(event.data) as T;
					this.notifyHandlers('message', data);
				} catch (e) {
					console.error('Failed to parse SSE message:', e);
				}
			};

			// Attach listeners for all pre-registered event types
			this.attachEventListeners();
		} catch (error) {
			console.error('Failed to create EventSource:', error);
		}
	}

	/**
	 * Subscribe to specific event type
	 */
	on(eventType: string, handler: EventHandler<T>): () => void {
		if (!this.handlers.has(eventType)) {
			this.handlers.set(eventType, new Set());

			// Add event listener for this type if already connected
			if (this.eventSource) {
				this.eventSource.addEventListener(eventType, (event: MessageEvent) => {
					try {
						const data = JSON.parse(event.data) as T;
						this.notifyHandlers(eventType, data);
					} catch (e) {
						console.error(`Failed to parse ${eventType} event:`, e);
					}
				});
			}
		}

		this.handlers.get(eventType)!.add(handler);

		// Return unsubscribe function
		return () => {
			this.handlers.get(eventType)?.delete(handler);
		};
	}

	/**
	 * Subscribe to error events
	 */
	onError(handler: ErrorHandler): () => void {
		this.errorHandlers.add(handler);
		return () => {
			this.errorHandlers.delete(handler);
		};
	}

	/**
	 * Disconnect from SSE
	 */
	disconnect(): void {
		if (this.reconnectTimeout) {
			clearTimeout(this.reconnectTimeout);
			this.reconnectTimeout = null;
		}

		if (this.eventSource) {
			this.eventSource.close();
			this.eventSource = null;
		}
		// Note: Don't reset reconnectAttempts here to allow proper max attempt tracking
	}

	/**
	 * Check if connected
	 */
	isConnected(): boolean {
		return this.eventSource?.readyState === EventSource.OPEN;
	}

	private notifyHandlers(eventType: string, data: T): void {
		this.handlers.get(eventType)?.forEach((handler) => {
			try {
				handler(data);
			} catch (e) {
				console.error(`Error in ${eventType} handler:`, e);
			}
		});
	}

	private handleReconnect(): void {
		// Guard against scheduling duplicate timers
		if (this.reconnectTimeout != null) return;

		if (this.reconnectAttempts >= (this.config.maxReconnectAttempts ?? 10)) {
			console.error('Max reconnection attempts reached');
			return;
		}

		this.reconnectAttempts++;
		const delay = this.config.reconnectInterval ?? 5000;

		console.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);

		this.reconnectTimeout = setTimeout(() => {
			this.reconnectTimeout = null;
			if (this.eventSource) {
				this.eventSource.close();
				this.eventSource = null;
			}
			this.connect();
		}, delay);
	}
}

// Dashboard-specific real-time types
export interface DashboardUpdate {
	type: 'metrics' | 'order' | 'inventory' | 'alert';
	data: unknown;
	timestamp: string;
}

// Create dashboard realtime instance
export function createDashboardRealtime(tenantId: string): RealtimeManager<DashboardUpdate> {
	// Use PUBLIC_API_BASE_URL for SSE endpoint, with proper tenantId encoding
	const baseUrl = PUBLIC_API_BASE_URL || '/api/v1';
	const encodedTenantId = encodeURIComponent(tenantId);
	return new RealtimeManager<DashboardUpdate>({
		url: `${baseUrl}/dashboard/events?tenant=${encodedTenantId}`,
		reconnectInterval: 5000,
		maxReconnectAttempts: 10
	});
}
