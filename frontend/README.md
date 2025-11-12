## ⚠️ **CRITICAL: Svelte 5 Runes & MCP Usage**

**🚨 ALL Svelte/SvelteKit development MUST use Svelte 5 runes and consult MCP documentation first!**

### 📚 **Before Any Code Changes:**

1. **ALWAYS** call `mcp_svelte_list-sections` to explore available documentation
2. **ALWAYS** call `mcp_svelte_get-documentation` with relevant sections
3. **NEVER** implement without checking current Svelte 5 documentation

### 🏗️ **Svelte 5 Runes (MANDATORY):**

- **`$state`** for reactive state (not `writable()` stores)
- **`$derived`** for computed values (not `$:` statements)
- **`$effect`** for side effects (not `$:` statements)
- **`$props`** for component props (not `export let`)
- **Direct state access** (not `$store` syntax)

### 📖 **Full Guidelines:**

See [`.svelte-instructions.md`](.svelte-instructions.md) for complete development guidelines.

---

## Features

- **Svelte 5 Runes**: Modern reactivity system with `$state`, `$derived`, `$effect`, and `$props` runes
- **TypeScript**: Full type safety with strict mode configuration
- **shadcn-svelte**: Beautiful, accessible UI components
- **Tailwind CSS**: Utility-first CSS framework
- **CapRover Deployment**: Ready for containerized deployment
- **Multi-tenant Architecture**: Built for SaaS with tenant isolation
- **Kanidm Authentication**: OAuth2/OIDC integration with Kanidm

## Tech Stack

- **Framework**: SvelteKit 5 with adapter-node
- **Language**: TypeScript 5.9+
- **Styling**: Tailwind CSS 4.x + shadcn-svelte
- **State Management**: Svelte 5 runes (no external stores needed)
- **Forms**: Valibot for validation
- **Testing**: Vitest (unit) + Playwright (E2E)
- **Package Manager**: npm (with native tooling)
- **Deployment**: CapRover with Docker

## Svelte 5 Runes Usage

This project uses Svelte 5's modern reactivity system:

### State Management

```typescript
// Reactive state
let count = $state(0);
let user = $state<User | null>(null);

// Derived values
let doubled = $derived(count * 2);
let isLoggedIn = $derived(!!user);

// Side effects
$effect(() => {
	console.log('Count changed:', count);
});
```

### Component Props

```svelte
<script lang="ts">
	let { title, items = $bindable() } = $props<{
		title: string;
		items?: string[];
	}>();
</script>
```

## Development

### Prerequisites

- **Node.js 20+** (native tools: vite, svelte-kit, vitest, playwright)
- Docker (for local development)

### Setup

1. Install dependencies:

```bash
npm install
```

2. Start development server:

```bash
npm run dev
```

3. Open [http://localhost:5173](http://localhost:5173) in your browser

### Available Scripts

- `npm run dev` - Start development server
- `npm run build` - Build for production
- `npm run preview` - Preview production build
- `npm run check` - Type checking and linting
- `npm run test:unit` - Run unit tests
- `npm run test:e2e` - Run E2E tests
- `npm run test` - Run all tests
- `npm run format` - Format code with Prettier

## Project Structure

```
src/
├── lib/
│   ├── api/           # API client functions
│   ├── components/    # Reusable UI components
│   │   └── ui/        # shadcn-svelte components
│   ├── hooks/         # Custom Svelte hooks
│   ├── stores/        # State management (Svelte 5 runes)
│   └── types/         # TypeScript type definitions
├── routes/            # SvelteKit routes
│   └── (dashboard)/   # Protected dashboard routes
└── app.html           # Main HTML template
```

## State Management

The app uses Svelte 5 runes for state management instead of traditional stores:

- **Auth State**: `src/lib/stores/auth.ts` - User authentication and tenant data
- **Inventory State**: `src/lib/stores/inventory.ts` - Products, categories, and inventory data

All state is reactive and automatically updates the UI when changed.

## API Integration

API calls are handled through dedicated modules:

- `src/lib/api/client.ts` - Base HTTP client with auth
- `src/lib/api/auth.ts` - Authentication endpoints
- `src/lib/api/inventory.ts` - Inventory management endpoints

## Deployment

### CapRover

The app is configured for CapRover deployment:

1. Build the Docker image:

```bash
docker build -t anthill-frontend .
```

2. Deploy to CapRover using the `caprover.json` configuration

### Environment Variables

Create a `.env` file with:

```env
VITE_API_BASE_URL=http://srv-anthill-user-service
VITE_KANIDM_BASE_URL=https://idm.example.com
```

## Migration from Svelte 4

This project has been migrated to Svelte 5 runes:

- `$state` replaces `let` declarations for reactive state
- `$derived` replaces reactive statements (`$:`) for computed values
- `$effect` replaces reactive statements for side effects
- `$props` replaces `export let` for component props
- `$bindable` enables two-way data binding for props

## Contributing

1. Follow the established code style (ESLint + Prettier)
2. Use Svelte 5 runes for all new reactive code
3. Write tests for new features
4. Update TypeScript types as needed
5. Follow the GitHub Flow workflow

## License

Private - Anthill Inventory SaaS Platform
