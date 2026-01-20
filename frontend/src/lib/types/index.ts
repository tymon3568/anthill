// Core types for inventory management SaaS

export interface User {
	id: string;
	email?: string;
	name?: string;
	role: 'owner' | 'admin' | 'manager' | 'user';
	tenantId: string;
	createdAt: string;
	updatedAt: string;
	// Optional fields
	preferred_username?: string;
}

export interface Tenant {
	id: string;
	name: string;
	domain?: string;
	slug?: string;
	createdAt: string;
	updatedAt: string;
}

export interface Product {
	id: string;
	sku: string;
	name: string;
	description?: string;
	category: string;
	price: number; // cents
	cost: number; // cents
	stock: number;
	minStock: number;
	maxStock?: number;
	tenantId: string;
	createdAt: string;
	updatedAt: string;
	deletedAt?: string;
}

export interface Category {
	id: string;
	name: string;
	description?: string;
	tenantId: string;
	createdAt: string;
	updatedAt: string;
}

export interface InventoryTransaction {
	id: string;
	productId: string;
	type: 'in' | 'out' | 'adjustment';
	quantity: number;
	reason?: string;
	reference?: string; // order id, etc.
	tenantId: string;
	createdAt: string;
	createdBy: string;
}

export interface Order {
	id: string;
	orderNumber: string;
	customerName: string;
	customerEmail?: string;
	status: 'pending' | 'confirmed' | 'shipped' | 'delivered' | 'cancelled';
	items: OrderItem[];
	total: number; // cents
	tenantId: string;
	createdAt: string;
	updatedAt: string;
}

export interface OrderItem {
	id: string;
	productId: string;
	productName: string;
	quantity: number;
	unitPrice: number; // cents
	total: number; // cents
}

// API Response types
export interface ApiResponse<T> {
	success: boolean;
	data?: T;
	error?: string;
	message?: string;
}

export interface PaginatedResponse<T> {
	data: T[];
	total: number;
	page: number;
	limit: number;
	totalPages: number;
}

// Form types
export interface LoginForm {
	email: string;
	password: string;
}

export interface RegisterForm {
	email: string;
	password: string;
	confirmPassword: string;
	fullName: string;
	tenantName: string;
}

export interface ProductForm {
	sku: string;
	name: string;
	description?: string;
	category: string;
	price: number;
	cost: number;
	stock: number;
	minStock: number;
	maxStock?: number;
}

// Store types for Svelte 5 runes
export interface AuthStore {
	user: User | null;
	tenant: Tenant | null;
	isAuthenticated: boolean;
	isLoading: boolean;
}

export interface InventoryStore {
	products: Product[];
	categories: Category[];
	isLoading: boolean;
	error: string | null;
}
