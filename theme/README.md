# FeatherSurf theme

FeatherSurf uses a **lightweight feather** visual language: open space, soft sky tones, a small mint accent, quiet borders, and motion that feels like a feather settling rather than a heavy interface animation.

## Design principles

The interface should feel calm, quick, and legible. Use blue for navigation and trusted interaction, mint for healthy or efficient states, amber for attention, and muted red only for destructive or blocked states. Keep surfaces bright and low-contrast in light mode. Dark mode uses deep navy rather than pure black so the feather mark remains airy.

Use `theme/feathersurf.css` as the shared source of truth for colors, typography, radii, shadows, focus rings, and motion timing.

## Theme modes

The default theme is light. Add `data-theme="dark"` to the root element for dark mode:

```html
<html data-theme="dark">
  <body class="fs-theme">...</body>
</html>
```

The design tokens are CSS custom properties, so a native browser shell or a web-based prototype can consume the same values. The UI should respect the user's system preference when no explicit theme is selected.

## Logo assets

- [`FeatherSurf SVG mark`](../assets/logos/feathersurf-mark.svg) — source-controlled, crisp at any size.
- [`Light-interface PNG mark`](../assets/logos/feathersurf-mark-light.png) — transparent generated artwork for light surfaces.
- [`Dark-interface PNG mark`](../assets/logos/feathersurf-mark-dark.png) — transparent generated artwork for dark surfaces.

Use the SVG for browser chrome and icons whenever possible. Use the PNG variants for product surfaces that need the softer generated feather treatment.

## Motion

The `fs-feather-mark--enter` animation is for one-time entry or startup. The
`fs-feather-mark--idle` animation is for an always-visible brand mark. Both are
transform/opacity-only and automatically disable under
`prefers-reduced-motion: reduce`.
