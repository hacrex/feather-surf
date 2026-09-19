# FeatherSurf animations

These animations are intentionally light and optional. They use CSS transforms and opacity only, which keeps them inexpensive for a browser UI and allows them to be disabled through `prefers-reduced-motion`.

The reusable classes are defined in [`theme/feathersurf.css`](../../theme/feathersurf.css):

```html
<link rel="stylesheet" href="../../theme/feathersurf.css" />

<img
  class="fs-feather-mark fs-feather-mark--enter"
  src="../logos/feathersurf-mark.svg"
  alt="FeatherSurf"
/>
```

Use `fs-feather-mark--enter` for one-time arrival or startup motion. Use
`fs-feather-mark--idle` only for a persistent brand mark. The reduced-motion
media query disables both animations automatically.
