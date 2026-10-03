---
description: Feature 004 - GUI chrome: toolbars, menus, sidebars, keyboard shortcuts, window management. Headless core + webview UI integration.
---

# Feature Specification: GUI Chrome & Interactions

**Feature Branch**: `004-gui-chrome-interactions`
**Status**: Draft
**Created**: 2026-09-30
**Constitution**: `.claude/constitution.md` v1.1.0

## Purpose

Feature 001 (headless), 002 (graph), 003 (state) entregan core + graph + state. Feature 004 aade **chrome de aplicacin**: ventana, toolbars, mens, sidebars, atajos, gestin de ventanas. Convierte la app headless en aplicacin de escritorio usable.

## Scope

### In Scope

- Ventana principal Tauri v2 (titlebar, min/max/close, resize)
- Toolbar principal: acciones Git comunes (commit, push, pull, branch, merge, rebase, stash)
- Men aplicacin: File, Edit, View, Repository, Branch, Remote, Window, Help
- Sidebar izquierdo: repos abiertos, ramas, tags, remotes, stashes
- Sidebar derecho: diff viewer, commit details, file tree
- Keyboard shortcuts: estndar (Ctrl+S commit, Ctrl+P push, Ctrl+B branch, etc.)
- Window management: multiple repos tabs, split view, fullscreen
- Theme: light/dark/system, persistido
- Settings dialog: Git config, UI preferences, shortcuts customization

### Out of Scope

- Conflict resolution inline editor (Feature 005)
- Search/filter UI (Feature 005)
- Blame/history view per file (Feature 005)
- Multi-repo dashboard (Feature 005)
- Plugin system (future)

## User Stories

### Story 1: Ventana + Toolbar (P0)

**As a** usuario abriendo la app
**I want** ventana nativa con toolbar funcional
**so that** pueda acceder a acciones Git rpidas

**Independent test**: Launch app, verify window opens, toolbar shows: Commit, Push, Pull, Branch, Merge, Rebase, Stash. Each click invokes `GitCommand` via Tauri.

**Acceptance scenarios**:
1. Given app launch, when window opens, then titlebar + toolbar visible, actions enabled
2. Given repo open, when click Commit, then abre commit dialog (Feature 005) o ejecuta `git commit`
3. Given repo open, when click Push, then ejecuta `git push` (refused si network, Feature 001)
4. Given repo open, when click Branch, then abre branch picker (Feature 005)

### Story 2: Men Aplicacin Completo (P0)

**As a** usuario
**I want** men estndar macOS/Windows/Linux
**so that** navegue por atajos conocidos

**Independent test**: Verificar men: File (New Repo, Open, Close, Quit), Edit (Undo, Redo, Copy, Paste), View (Toggle Sidebar, Zoom, Theme), Repository (Open, Clone, Settings), Branch (New, Delete, Rename, Checkout), Remote (Add, Remove, Fetch, Push, Pull), Window (Minimize, Zoom, Next Repo), Help (About, Shortcuts, Docs).

**Acceptance scenarios**:
1. Given app running, when click File > Open, then abre file picker para repo
2. Given repo open, when click Repository > Settings, then abre settings dialog
3. Given multiple repos, when Window > Next Repo, then switch tab

### Story 3: Sidebars (P0)

**As a** usuario trabajando en repo
**I want** sidebar izquierdo con navegacin, derecho con detalles
**so that** navegue sin cambiar contexto

**Independent test**: Left sidebar tabs: Repos, Branches, Tags, Remotes, Stashes. Right sidebar tabs: Diff, Commit Details, File Tree. Cada tab navegable, selecciones sincronizadas.

**Acceptance scenarios**:
1. Given repo open, when click Branches tab, then lista ramas, checkout con click
2. Given commit selected, when click Diff tab, then muestra diff completo (Feature 005)
3. Given file selected, when click File Tree, then muestra rbol navegable

### Story 4: Keyboard Shortcuts (P0)

**As a** power user
**I want** atajos estndar + personalizables
**so that** trabaje sin mouse

**Independent test**: Verificar atajos: `Ctrl+S` (commit), `Ctrl+P` (push), `Ctrl+Shift+P` (pull), `Ctrl+B` (branch), `Ctrl+Shift+B` (new branch), `Ctrl+M` (merge), `Ctrl+R` (rebase), `Ctrl+T` (stash), `Ctrl+Shift+F` (fetch), `Ctrl+Shift+G` (graph focus), `Ctrl+1/2/3` (sidebar tabs), `F11` (fullscreen), `Ctrl+,` (settings).

**Acceptance scenarios**:
1. Given repo open, when press `Ctrl+S`, then abre commit dialog
2. Given settings open, when change shortcut, then persiste y funciona
3. Given no repo, when press `Ctrl+S`, then no-op o disabled

### Story 5: Window Management (P1)

**As a** usuario con mltiples repos
**I want** tabs por repo, split view, fullscreen
**so that** cambie contexto rpido

**Independent test**: Abrir 3 repos, verificar tabs, drag to reorder, split horizontal/vertical, fullscreen toggle.

**Acceptance scenarios**:
1. Given 3 repos open, when click tab, then switch repo instant
2. Given graph open, when drag split handle, then split view horizontal/vertical
3. When press `F11`, then fullscreen toggle

### Story 6: Settings Persistence (P1)

**As a** usuario
**I want** preferencias persistidas
**so that** no reconfigure cada sesin

**Independent test**: Cambiar theme, shortcuts, window layout, cerrar app, reabrir, assert todo restaurado.

**Acceptance scenarios**:
1. Given theme dark, when restart, then theme dark
2. Given shortcut `Ctrl+S` cambiado a `Ctrl+Shift+C`, when restart, then nuevo shortcut funciona
3. Given split view horizontal, when restart, then layout restaurado

## Non-Functional Requirements

### NFR-001: App Launch < 2s

Cold start (incl. Tauri + webview + renderer) < 2s en mquina estndar.

### NFR-002: Memory < 200MB

Total RSS (Tauri + webview + renderer + core) < 200MB en repo 600k.

### NFR-003: 60fps UI

Todas las animaciones, pan/zoom, sidebar transitions a 60fps.

### NFR-004: Accessibility (WCAG 2.1 AA)

Semantic HTML, ARIA labels, keyboard nav, focus visible, color contrast, screen reader support.

### NFR-005: Cross-Platform Native Feel

Windows: Win32 titlebar + Mica/Acrylic. macOS: native titlebar + traffic lights. Linux: GTK/Qt integration.

### NFR-006: Settings Persistence

Settings en `~/.config/gitflowfy/settings.json` (o `%APPDATA%` Windows). JSON, versionado, migracin automtica.

## Constraints & Decisions

### Q1: UI Framework en crates/ui -> **Resuelto: SolidJS (B)**

**Decisin**: SolidJS por fine-grained reactivity, tamao pequeo (~7KB), sin VDOM, TypeScript nativo, performance cercano a vanilla.

**Justificacin**: Renderer Feature 002 ya usa Canvas directo (no React/Vue). SolidJS signals encajan con Canvas rendering. Tamao ~7KB gzipped. TypeScript first-class.

### Q2: State Management en UI -> **Resuelto: SolidJS Signals + Stores (A)**

**Decisin**: Signals para estado local (sidebar open, active tab), Stores para estado global (repos abiertos, theme, shortcuts). Tauri events para IPC.

**Justificacin**: Signals fine-grained = 60fps UI sin re-renders innecesarios. Stores globales simples. Tauri events para IPC nativo. Sin Redux/Zustand overhead.

### Q3: Theme System -> **Resuelto: CSS Custom Properties + `prefers-color-scheme` (A)**

**Decisin**: CSS custom properties (variables) + `prefers-color-scheme` media query + manual toggle. Persistencia en settings JSON.

**Justificacin**: Zero runtime overhead, nativo CSS, funciona con SolidJS sin extra deps. Tailwind (C) aade build step. CSS-in-JS (B) runtime overhead.

### Q4: Sidebar Layout System -> **Resuelto: CSS Grid + Custom Resize Handles (A)**

**Decisin**: CSS Grid para layout base + custom resize handles (drag) para split panes. Sin librera externa.

**Justificacin**: CSS Grid nativo, 0 deps. Split.js (B) aade ~10KB. Custom handles = control total, accessibility nativa.

### Q5: Settings Schema & Migration -> **Resuelto: JSON + semver + auto-migration functions (B)**

**Decisin**: JSON + semver en schema + funciones de migracin automticas por versin. Schema versionado en `settings.json`.

**Justificacin**: JSON nativo, semver estndar, migracin automtica evita breaking changes. TOML (C) requiere parser extra. Manual (A) frgil.

---

## Clarification Check

- [x] Q1 resuelto
- [x] Q2 resuelto
- [x] Q3 resuelto
- [x] Q4 resuelto
- [x] Q5 resuelto

---

*Spec version 1.0.0 - all clarifications resolved, ready for review*