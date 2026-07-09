---
name: Typstify
description: High-performance static site generator with Typst and Markdown support
colors:
  primary: "#d4760a"
  primary-deep: "#a85c08"
  accent: "#1e5a8a"
  bg: "#ffffff"
  surface: "#f8f7f5"
  ink: "#1a1a1e"
  muted: "#6b6b73"
  border: "#e2e1dd"
  success: "#2d8a4e"
  error: "#c4382a"
typography:
  display:
    fontFamily: "Inter, system-ui, sans-serif"
    fontSize: "clamp(2rem, 5vw, 3.5rem)"
    fontWeight: 700
    lineHeight: 1.1
    letterSpacing: "-0.02em"
  headline:
    fontFamily: "Inter, system-ui, sans-serif"
    fontSize: "clamp(1.5rem, 3vw, 2rem)"
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: "-0.01em"
  title:
    fontFamily: "Inter, system-ui, sans-serif"
    fontSize: "1.25rem"
    fontWeight: 600
    lineHeight: 1.3
  body:
    fontFamily: "Inter, system-ui, sans-serif"
    fontSize: "1rem"
    fontWeight: 400
    lineHeight: 1.6
  label:
    fontFamily: "Inter, system-ui, sans-serif"
    fontSize: "0.875rem"
    fontWeight: 500
    lineHeight: 1.4
    letterSpacing: "0.01em"
  code:
    fontFamily: "JetBrains Mono, ui-monospace, monospace"
    fontSize: "0.875rem"
    fontWeight: 400
    lineHeight: 1.5
rounded:
  sm: "4px"
  md: "6px"
  lg: "8px"
  full: "9999px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "16px"
  lg: "24px"
  xl: "32px"
  2xl: "48px"
  3xl: "64px"
components:
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "#ffffff"
    rounded: "{rounded.md}"
    padding: "10px 24px"
    typography: "{typography.label}"
  button-primary-hover:
    backgroundColor: "{colors.primary-deep}"
    textColor: "#ffffff"
    rounded: "{rounded.md}"
    padding: "10px 24px"
    typography: "{typography.label}"
  button-secondary:
    backgroundColor: "transparent"
    textColor: "{colors.ink}"
    rounded: "{rounded.md}"
    padding: "10px 24px"
    typography: "{typography.label}"
  input:
    backgroundColor: "{colors.bg}"
    textColor: "{colors.ink}"
    rounded: "{rounded.md}"
    padding: "10px 14px"
    typography: "{typography.body}"
  chip:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.muted}"
    rounded: "{rounded.full}"
    padding: "4px 12px"
    typography: "{typography.label}"
---

## Design System: Typstify

### 1. Overview

## Creative North Star: "The Clean Workshop"

A quiet workshop — precise tools on a clean bench, nothing wasted. Every surface earns its place. The palette is restrained: warm honey accent against near-white surfaces, deep ink for text, structural blue for secondary emphasis. This system explicitly rejects the generic AI aesthetic — no cream backgrounds, no gradient text, no identical card grids, no glassmorphism.

**Key Characteristics:**

- Restrained palette: one warm accent (honey/amber) used sparingly, structural blue for secondary emphasis
- Pure white surfaces with minimal tint — the accent carries the warmth, not the background
- Typography-first hierarchy: Inter in multiple weights, no decorative font pairings
- Flat elevation by default — shadows appear only as state response (hover, focus)
- Developer-grade precision: every token has a specific role, nothing decorative

### 2. Colors

The palette is restrained — tinted neutrals plus one warm accent at ≤10% of surface area. The accent does the emotional work; the surface stays clean.

#### Primary

- **Workshop Honey** (oklch(0.691 0.146 74.6) / #d4760a): The brand anchor. Used on primary buttons, active states, key CTAs, and the logo mark. Its warmth carries the brand personality against the otherwise neutral surface.

- **Honey Deep** (oklch(0.55 0.146 74.6) / #a85c08): Hover/active state for primary elements. Darker for contrast when the element needs to feel pressed or engaged.

#### Secondary

- **Structural Blue** (oklch(0.45 0.10 250) / #1e5a8a): Links, secondary actions, informational badges, and code syntax. Provides cool contrast against the warm primary without competing for attention.

#### Neutral

- **Clean White** (oklch(1.000 0.000 0) / #ffffff): Body background. Pure, no tint. The accent carries the warmth; the surface stays neutral.

- **Workshop Surface** (oklch(0.975 0.008 75) / #f8f7f5): Cards, panels, elevated surfaces. Slight warm tint toward the primary hue — barely perceptible, just enough to separate from pure white.

- **Deep Ink** (oklch(0.145 0.005 250) / #1a1a1e): Body text. Near-black with a cool undertone for maximum contrast against white. Never pure black (#000000).

- **Tool Gray** (oklch(0.50 0.008 250) / #6b6b73): Secondary text, captions, timestamps, muted labels. Ink pulled toward the surface.

- **Edge** (oklch(0.91 0.006 75) / #e2e1dd): Borders, dividers, subtle separations. Warm-tinted to harmonize with the surface.

#### Named Rules

**The Ten Percent Rule.** Workshop Honey appears on ≤10% of any given screen. Its rarity is the point — it marks what matters. If every element is honey, nothing is.

**The Clean Surface Rule.** Backgrounds are pure white or near-pure. Warmth lives in the accent and typography, never in the surface tint. The moment you add warmth to both accent AND background, you're in AI-cream territory.

### 3. Typography

**Display Font:** Inter (with system-ui, sans-serif fallback)
**Body Font:** Inter (with system-ui, sans-serif fallback)
**Code Font:** JetBrains Mono (with ui-monospace, monospace fallback)

**Character:** A single sans-serif family in multiple weights — precise, technical, no decoration. Inter's geometry reads as clean and modern without being cold. The pairing with JetBrains Mono for code blocks reinforces the developer-tool identity.

#### Hierarchy

- **Display** (700, clamp(2rem, 5vw, 3.5rem), 1.1): Hero headlines on landing pages and documentation headers. Maximum presence, minimum decoration.
- **Headline** (600, clamp(1.5rem, 3vw, 2rem), 1.2): Section headers, page titles. Clear hierarchy break from display.
- **Title** (600, 1.25rem, 1.3): Card titles, subsection headers, navigation items.
- **Body** (400, 1rem, 1.6): Primary reading text. Line length capped at 65–75ch for comfortable reading.
- **Label** (500, 0.875rem, 0.01em): Buttons, badges, navigation, form labels. Slightly tracked for clarity at small sizes.

#### Named Rules

**The Single Family Rule.** One font family (Inter) in multiple weights. No serif display, no decorative pairings. The type system is a toolbox, not a showcase.

**The Weight Hierarchy Rule.** Weight carries meaning: 700 = hero, 600 = section, 500 = label, 400 = body. Never use weight decoratively.

### 4. Elevation

Flat by default. Surfaces are flat at rest — no shadows, no lift. Depth is conveyed through tonal layering (the surface color) and borders, not drop shadows. Shadows appear only as a response to state: hover elevation, focus rings, dropdown menus, and modal overlays.

#### Shadow Vocabulary

- **Hover Lift** (`0 2px 8px rgba(0,0,0,0.08)`): Subtle lift on interactive elements (cards, buttons) when hovered. ambient, not structural.
- **Focus Ring** (`0 0 0 2px rgba(212,118,10,0.3)`): Honey-tinted focus indicator. Functional, not decorative.
- **Dropdown** (`0 4px 16px rgba(0,0,0,0.12)`): Menus, tooltips, popovers. Structural shadow for separation.

#### Named Rules

**The Flat-By-Default Rule.** Surfaces ship flat. Shadows are earned through interaction, not applied as decoration. If a shadow exists at rest, it's wrong.

### 5. Components

#### Buttons

- **Shape:** Gently curved edges (6px radius). Full-pill for tags and badges only.
- **Primary:** Workshop Honey fill, white text, 10px 24px padding. Label typography (500, 0.875rem).
- **Hover / Focus:** Background shifts to Honey Deep. Focus ring: 2px honey-tinted glow.
- **Secondary:** Transparent background, Deep Ink text, same padding. Border: 1px Edge color.

#### Chips / Tags

- **Style:** Workshop Surface background, Tool Gray text, full-pill radius (9999px). Compact: 4px 12px padding.
- **State:** Selected state swaps to Primary fill with white text.

#### Cards / Containers

- **Corner Style:** 6px radius — barely curved, structural not decorative.
- **Background:** Clean White (default) or Workshop Surface (elevated).
- **Shadow Strategy:** Flat at rest. Hover Lift shadow on interactive cards.
- **Border:** 1px Edge color. Never decorative side-stripe borders.

#### Inputs / Fields

- **Style:** Clean White background, 1px Edge border, 6px radius. Body typography.
- **Focus:** Border shifts to Primary, 2px honey-tinted focus ring replaces default border.
- **Error:** Border shifts to Error (#c4382a), no other decoration.

#### Navigation

- **Style:** Horizontal top bar or sidebar. Clean White background, Deep Ink text.
- **Default:** Label typography, Tool Gray text.
- **Hover:** Deep Ink text, subtle Workshop Surface background.
- **Active:** Primary text color, 2px bottom border (horizontal) or left border (sidebar) in Workshop Honey.

#### Search

- **Style:** Inline dropdown in nav bar. Clean White surface, 1px Edge border, 6px radius.
- **Input:** 10px 14px padding, Body typography, placeholder in Tool Gray. Focus: border shifts to Primary, 2px honey-tinted ring.
- **Results:** Absolute positioned dropdown, max-height 320px, scrollable. Each item: 12px 14px padding, hover shifts to Workshop Surface background.
- **Tags:** Workshop Surface chips with Tool Gray text, full-pill radius.
- **States:** Default (collapsed), typing (results appear), loading, no results ("No results for..."), error, keyboard navigation highlight.
- **Keyboard:** Arrow keys navigate, Enter selects, Escape closes.

#### Code Blocks

- **Style:** Deep Ink background (#1a1a1e), near-white text. JetBrains Mono at 0.875rem.
- **Border:** None — the dark background provides sufficient separation.
- **Language Badge:** Workshop Surface chip in top-right corner.

### 6. Do's and Don'ts

#### Do

- **Do** use Workshop Honey sparingly — it marks what matters, not everything.
- **Do** keep backgrounds pure white. The accent carries warmth; the surface stays neutral.
- **Do** use weight hierarchy (700/600/500/400) to convey importance, not size alone.
- **Do** keep shadows flat at rest. Hover and focus earn their shadows.
- **Do** use JetBrains Mono for all code. It's the developer identity.
- **Do** maintain ≥4.5:1 contrast for body text, ≥3:1 for large text.
- **Do** respect `prefers-reduced-motion: reduce` with instant transitions.

#### Don't

- **Don't** use cream/sand/beige backgrounds — the saturated AI default. Pure white.
- **Don't** apply gradient text (`background-clip: text`). Decorative, never meaningful.
- **Don't** use glassmorphism (blurs, glass cards) as decoration.
- **Don't** put border-left > 1px as a colored accent stripe on cards or alerts.
- **Don't** use `border-radius: 16px+` on cards. Top out at 8px; full-pill for tags only.
- **Don't** create identical card grids with icon + heading + text repeated endlessly.
- **Don't** add tiny uppercase tracked eyebrows above every section.
- **Don't** pair fonts that are similar but not identical. One family, multiple weights.
- **Don't** animate CSS layout properties. Use transform and opacity only.
- **Don't** gate content visibility on class-triggered transitions — content must be visible by default.
