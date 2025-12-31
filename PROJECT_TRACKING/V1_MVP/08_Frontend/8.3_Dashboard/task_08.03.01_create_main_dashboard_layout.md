# Task: Create Main Dashboard Layout and Navigation

**Task ID:** V1_MVP/08_Frontend/8.3_Dashboard/task_08.03.01_create_main_dashboard_layout.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.3_Dashboard
**Priority:** High
**Status:** NeedsReview
**Assignee:** Claude
**Created Date:** 2025-01-21
**Last Updated:** 2025-12-31 21:30

## Detailed Description:
Create the main dashboard layout with responsive sidebar navigation, header, and content area using shadcn-svelte components. The layout follows modern SaaS dashboard patterns with collapsible sidebar, command palette, and full accessibility support.

## Technical Approach:
- **UI Framework:** shadcn-svelte components (Sidebar, Sheet, Command, Breadcrumb)
- **Design System:** Frappe-inspired minimal design (gray-based, minimal shadows)
- **State Management:** Svelte 5 runes ($state, $derived)
- **Routing:** SvelteKit file-based routing with (protected) group

## Specific Sub-tasks:

### Phase 1: Install Required shadcn-svelte Components
- [x] 1.1. Install sidebar component (`bunx shadcn-svelte@latest add sidebar`)
- [x] 1.2. Install sheet component for mobile drawer
- [x] 1.3. Install collapsible component for menu groups
- [x] 1.4. Install breadcrumb component
- [x] 1.5. Install avatar component for user profile
- [x] 1.6. Install scroll-area component
- [x] 1.7. Install sonner/toast for notifications
- [x] 1.8. Install command component for ⌘K palette

### Phase 2: Create App Sidebar Component
- [x] 2.1. Create `src/lib/components/app-sidebar.svelte` with navigation structure
- [x] 2.2. Define navigation items for inventory app (Dashboard, Inventory, Orders, Integrations, Settings)
- [x] 2.3. Implement collapsible menu groups (Inventory > Products, Categories, Stock)
- [x] 2.4. Add icons from lucide-svelte for menu items
- [x] 2.5. Implement active state highlighting based on current route
- [x] 2.6. Add user profile section at bottom of sidebar

### Phase 3: Create Header Component
- [x] 3.1. Create `src/lib/components/app-header.svelte`
- [x] 3.2. Add sidebar trigger button for mobile
- [x] 3.3. Implement breadcrumb navigation
- [x] 3.4. Add search trigger (opens command palette)
- [x] 3.5. Add notifications dropdown
- [x] 3.6. Add theme toggle (light/dark mode)
- [ ] 3.7. Add user profile dropdown with logout (moved to sidebar footer)

### Phase 4: Create Dashboard Layout
- [x] 4.1. Update `src/routes/(protected)/+layout.svelte` with SidebarProvider
- [x] 4.2. Integrate AppSidebar and AppHeader components
- [x] 4.3. Configure sidebar collapsible modes (offcanvas for mobile, icon for desktop)
- [x] 4.4. Implement proper content area with padding and scroll

### Phase 5: Implement Theme System
- [x] 5.1. Create theme store using Svelte 5 runes
- [x] 5.2. Implement system preference detection
- [x] 5.3. Add theme persistence to localStorage
- [x] 5.4. Create ThemeToggle component

### Phase 6: Mobile Responsiveness
- [x] 6.1. Configure sidebar as sheet/drawer on mobile (<768px)
- [x] 6.2. Add hamburger menu trigger in header
- [x] 6.3. Implement touch-friendly navigation
- [ ] 6.4. Test on various mobile viewports

### Phase 7: Accessibility & Keyboard Navigation
- [x] 7.1. Implement keyboard shortcuts for navigation
- [x] 7.2. Add proper ARIA labels to all navigation elements
- [x] 7.3. Ensure focus management for sidebar toggle
- [ ] 7.4. Test with screen reader

### Phase 8: Loading States & Skeleton
- [x] 8.1. Create skeleton components for sidebar
- [x] 8.2. Add loading states for async navigation data
- [x] 8.3. Implement Suspense boundaries where needed

### Phase 9: Testing
- [x] 9.1. Write Vitest unit tests for navigation logic
- [x] 9.2. Write Vitest tests for theme store
- [ ] 9.3. Write Playwright E2E tests for navigation flow
- [ ] 9.4. Test mobile navigation with Playwright

## Acceptance Criteria:
- [x] Main dashboard layout component functional with shadcn-svelte Sidebar
- [x] Responsive sidebar navigation working (collapsible on desktop, drawer on mobile)
- [x] Header with user profile, notifications, and search implemented
- [x] Breadcrumb navigation system operational
- [x] Mobile-responsive design with touch-friendly interactions
- [x] Theme support with light/dark mode toggle and persistence
- [x] Navigation guards protecting routes properly (via hooks.server.ts)
- [x] Loading states and skeleton screens displayed during data fetch
- [x] Keyboard navigation and accessibility features working (Tab, Escape, ⌘K)
- [ ] Cross-browser compatibility ensured (Chrome, Firefox, Safari, Edge) - needs manual verification
- [x] Unit tests written with Vitest for layout components
- [ ] E2E tests written with Playwright for navigation flows - deferred to separate task
- [x] Code passes `bun run check` and `bun run lint`

## Dependencies:
- V1_MVP/08_Frontend/8.1_Project_Setup/task_08.01.01_setup_sveltekit_project.md ✅ Done

## Related Documents:
- `frontend/src/routes/(protected)/+layout.svelte` - Main protected layout
- `frontend/src/lib/components/app-sidebar.svelte` - Sidebar component (to be created)
- `frontend/src/lib/components/app-header.svelte` - Header component (to be created)
- `frontend/src/lib/stores/theme.svelte.ts` - Theme store (to be created)

## Files to Create/Modify:
```
frontend/
├── src/
│   ├── lib/
│   │   ├── components/
│   │   │   ├── app-sidebar.svelte          # Main sidebar component
│   │   │   ├── app-header.svelte           # Header with breadcrumb, search, profile
│   │   │   ├── nav-main.svelte             # Main navigation items
│   │   │   ├── nav-user.svelte             # User profile in sidebar
│   │   │   ├── theme-toggle.svelte         # Light/dark mode toggle
│   │   │   └── ui/
│   │   │       ├── sidebar/                # shadcn sidebar (to install)
│   │   │       ├── sheet/                  # shadcn sheet (to install)
│   │   │       ├── collapsible/            # shadcn collapsible (to install)
│   │   │       ├── breadcrumb/             # shadcn breadcrumb (to install)
│   │   │       ├── avatar/                 # shadcn avatar (to install)
│   │   │       ├── command/                # shadcn command (to install)
│   │   │       └── sonner/                 # shadcn sonner (to install)
│   │   └── stores/
│   │       └── theme.svelte.ts             # Theme state with runes
│   └── routes/
│       └── (protected)/
│           ├── +layout.svelte              # Update with SidebarProvider
│           └── dashboard/
│               └── +page.svelte            # Update with new layout
```

## PR Review Issues (PR #128):
---
### Fixed Issues (Batch 1 - 2025-12-31 13:25):
- [x] Fix `getInitials()` edge case - names with consecutive spaces producing "JUNDEFINED" (nav-user.svelte) - **Severity: Warning**
- [x] Fix `openItems` effect accumulating state instead of resetting on navigation (nav-main.svelte) - **Severity: Warning**
- [x] Fix `metricCardCount` RangeError with non-integer/negative values (dashboard-skeleton.svelte) - **Severity: Warning**
- [x] Fix `Math.random()` SSR hydration mismatch - use deterministic heights (dashboard-skeleton.svelte) - **Severity: Warning**
- [x] Fix `children` prop type missing `Snippet` in type definition (breadcrumb-link.svelte) - **Severity: Warning**
- [x] Fix breadcrumb ellipsis accessibility - sr-only text hidden by parent aria-hidden (breadcrumb-ellipsis.svelte) - **Severity: Warning**
- [x] Fix `data-[disabled=true]` vs `data-[disabled]` selector mismatch (command-link-item.svelte) - **Severity: Warning**
- [x] Fix keyboard shortcut case sensitivity for CapsLock users (context.svelte.ts) - **Severity: Style**
- [x] Fix Dialog.Header placement - move inside Dialog.Content for accessibility (command-dialog.svelte) - **Severity: Warning**
- [x] Remove unused `allItems` variable and import (command-palette.svelte) - **Severity: Style**
- [x] Fix task file date inconsistency (Last Updated before Created Date) - **Severity: Style**

### Fixed Issues (Batch 2 - 2025-12-31 21:30):
- [x] Fix `safeMetricCardCount` to handle NaN/Infinity using `Number.isFinite()` (dashboard-skeleton.svelte) - **Severity: Warning**
- [x] Add null check to `useSidebar()` to throw clear error outside SidebarProvider (context.svelte.ts) - **Severity: Critical**
- [x] Remove redundant Escape handler - Bits UI Dialog handles this (command-palette.svelte) - **Severity: Nitpick**
- [x] Remove unused `getItemIcon` function and `FileTextIcon` import (command-palette.svelte) - **Severity: Nitpick**
- [x] Implement ⌘D, ⌘P, ⌘, keyboard shortcuts for Quick Actions (command-palette.svelte) - **Severity: Warning**
- [x] Remove redundant setTimeout delay in handleLinkClick (nav-main.svelte) - **Severity: Nitpick**
- [x] Fix ARIA role semantics - remove role="menu"/role="menuitem" from navigation (nav-main.svelte) - **Severity: Warning**

### Deferred/Not Applicable:
- [ ] Security: Upgrade @sveltejs/kit to 2.47.2+ for CVE fixes - **Deferred to separate security task**
- [N/A] `onclick` vs `on:click` - INVALID for Svelte 5 (onclick is correct syntax for bits-ui components)
- [N/A] Theme store import path - VALID ($lib/stores/theme.svelte works for .svelte.ts files)
- [N/A] Task file markdown formatting (hard tabs) - DEFERRED (cosmetic only, not blocking)

## Notes / Discussion:
---
* Using shadcn-svelte Sidebar component for consistent, accessible navigation
* Sidebar supports 3 collapsible modes: offcanvas, icon, none
* Command palette (⌘K) provides power-user quick navigation
* Theme follows Frappe UI principles: minimal shadows, gray-based colors
* Navigation structure designed for inventory management workflow
* Consider role-based menu visibility for future multi-role support

## AI Agent Log:
---
* 2025-12-31 17:43: Task claimed by Claude
  - Verified dependencies: task_08.01.01 is Done ✅
  - Created feature branch: feature/08.03-dashboard-layout-shadcn
  - Updated task with detailed sub-tasks for shadcn-svelte implementation
  - Design decision: Option C (shadcn-svelte Dashboard Pattern) selected by user
  - Starting implementation with Phase 1: Install shadcn components
* 2025-12-31 18:00: Phase 1 completed by Claude
  - Installed 8 shadcn-svelte components: sidebar, sheet, collapsible, breadcrumb, avatar, scroll-area, sonner, command
  - Also added is-mobile hook from shadcn-svelte
* 2025-12-31 18:15: Phases 2-5 completed by Claude
  - Created navigation config: `src/lib/config/navigation.ts`
  - Created theme store: `src/lib/stores/theme.svelte.ts`
  - Created nav-main.svelte with collapsible menu groups
  - Created nav-user.svelte with user profile dropdown
  - Created theme-toggle.svelte with light/dark/system options
  - Created app-sidebar.svelte integrating all navigation
  - Created app-header.svelte with breadcrumbs, search, notifications
  - Updated protected layout with SidebarProvider and Toaster
  - Simplified dashboard layout (parent handles sidebar/header)
  - Code passes `bun run check` with 0 errors
* 2025-01-01 10:00: Starting Phase 6 (Mobile Responsiveness) by Claude
  - Verified shadcn-svelte sidebar has built-in mobile support via Sheet component
  - sidebar.svelte already uses Sheet.Root on mobile (isMobile detection via IsMobile hook)
  - Sidebar.Trigger already in app-header.svelte (toggles sidebar.openMobile on mobile)
  - Need to: enhance header for mobile, verify touch interactions, test viewports
* 2025-01-01 10:30: Phase 6 implementation completed by Claude
  - 6.1 ✅ Sidebar already configured as Sheet on mobile via shadcn-svelte (IsMobile hook at 768px)
  - 6.2 ✅ Added hamburger menu (MenuIcon) trigger in app-header for mobile
  - 6.3 ✅ Touch-friendly: min 44px tap targets on all nav items, icons sized 20px mobile / 16px desktop
  - Added auto-close sidebar on mobile navigation (afterNavigate hook)
  - Made user dropdown open upward on mobile for better UX
  - All checks pass: `bun run check` 0 errors
  - Remaining: 6.4 viewport testing (manual/Playwright)
* 2025-01-01 11:00: Starting Phase 7 (Accessibility & Keyboard Navigation) by Claude
  - Plan: implement ⌘/Ctrl+K command palette, ⌘/Ctrl+B sidebar toggle
  - Add ARIA labels to all interactive elements
  - Ensure proper focus management for modals/sheets
  - Test with keyboard-only navigation
* 2025-01-01 11:30: Phase 7 implementation completed by Claude
  - 7.1 ✅ Created command-palette.svelte with Ctrl+K shortcut for quick navigation
  - 7.1 ✅ Sidebar already has Ctrl+B toggle (built into shadcn-svelte sidebar)
  - 7.2 ✅ Added ARIA labels to sidebar, nav-main, app-header (aria-label, aria-labelledby, aria-expanded, aria-controls, aria-current)
  - 7.3 ✅ Focus management handled by shadcn-svelte Dialog/Sheet components
  - Created command palette with: Quick Actions, Navigation, Settings groups
  - Wired search button in header to open command palette
  - All checks pass: `bun run check` 0 errors
  - Remaining: 7.4 screen reader testing (manual)
* 2025-01-01 12:00: Starting Phase 8 (Loading States & Skeleton) by Claude
  - Plan: create skeleton components for sidebar, dashboard cards
  - Add loading states during auth initialization
  - Implement Suspense-like patterns for async content
* 2025-01-01 12:30: Phase 8 implementation completed by Claude
  - 8.1 ✅ Created sidebar-skeleton.svelte with header, nav, settings, and user sections
  - 8.2 ✅ Created dashboard-skeleton.svelte with metric cards, charts, lists, tables
  - 8.3 ✅ Updated protected layout with conditional rendering (isLoading ? skeleton : content)
  - Skeletons show during authState.isLoading (auth initialization)
  - Added aria-busy attributes for accessibility
  - All checks pass: `bun run check` 0 errors
* 2025-01-01 13:00: Phase 9 (Unit Tests) implementation completed by Claude
  - 9.1 ✅ Created navigation.test.ts with 35 tests for navigation helpers
  - 9.2 ✅ Created theme.test.ts with 28 tests for theme store logic
  - Fixed isPathActive() bug: now properly handles path boundaries (/inventory vs /inventory-new)
  - All 63 unit tests pass
  - All checks pass: `bun run check` 0 errors
  - Remaining: 9.3, 9.4 Playwright E2E tests (recommend deferring to separate task)
* 2025-01-01 13:30: Task set to NeedsReview by Claude
  - All core phases (1-8) completed
  - Unit tests (9.1, 9.2) completed with 63 passing tests
  - E2E tests (9.3, 9.4) deferred - recommend separate task for comprehensive E2E coverage
  - Manual testing items remaining: viewport testing (6.4), screen reader (7.4), cross-browser (acceptance criteria)
  - Commits pushed to feature/08.03-dashboard-layout-shadcn
  - Ready for code review and PR creation
* 2025-12-31 13:25: PR #128 Review Auto-Fix Batch 1 by Claude
  - Fetched PR review comments from CodeRabbit, cubic-dev-ai, codeant-ai, gemini-code-assist
  - Categorized 12 unresolved issues (1 Critical, 9 Warning, 3 Style)
  - Fixed 11 issues:
    1. nav-user.svelte: getInitials() now filters empty parts from name.split()
    2. nav-main.svelte: openItems effect now resets state instead of accumulating
    3. dashboard-skeleton.svelte: metricCardCount normalized to prevent RangeError
    4. dashboard-skeleton.svelte: replaced Math.random() with deterministic heights for SSR
    5. breadcrumb-link.svelte: added children?: Snippet to props type
    6. breadcrumb-ellipsis.svelte: moved aria-hidden to icon only for screen reader access
    7. command-link-item.svelte: changed data-[disabled=true] to data-[disabled]
    8. context.svelte.ts: added toLowerCase() for case-insensitive keyboard shortcut
    9. command-dialog.svelte: moved Dialog.Header inside Dialog.Content for a11y
    10. command-palette.svelte: removed unused allItems variable and import
    11. task file: fixed date inconsistency (Last Updated was before Created Date)
  - Deferred @sveltejs/kit upgrade to separate security task
  - Validated: `bun run check` passes (0 errors), 63 dashboard tests pass
  - Invalidated 2 false positives (onclick syntax, theme import path are correct for Svelte 5)
* 2025-12-31 21:30: PR #128 Review Auto-Fix Batch 2 by Claude
  - Re-fetched PR #128 to find remaining unresolved issues after first batch
  - Identified 7 additional issues from CodeRabbit and cubic-dev-ai reviews
  - Fixed all 7 issues:
    1. dashboard-skeleton.svelte: safeMetricCardCount now uses Number.isFinite() for NaN/Infinity
    2. context.svelte.ts: useSidebar() now throws clear error when called outside SidebarProvider
    3. command-palette.svelte: removed redundant Escape handler (Bits UI Dialog handles it)
    4. command-palette.svelte: removed unused getItemIcon function and FileTextIcon import
    5. command-palette.svelte: implemented ⌘D, ⌘P, ⌘, keyboard shortcuts for Quick Actions
    6. nav-main.svelte: removed setTimeout delay in handleLinkClick (immediate close is fine)
    7. nav-main.svelte: removed role="menu"/role="menuitem" to fix ARIA navigation semantics
  - Validated: `bun run check` passes (0 errors), 63 dashboard tests pass
  - Pre-existing auth test failures (URL encoding) unrelated to dashboard changes
