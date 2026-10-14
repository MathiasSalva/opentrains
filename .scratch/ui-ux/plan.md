# UI/UX enhancement plan, informed by taste-library

Status: needs-triage

Scope: research and planning only. No app code was changed. Domain terms follow `CONTEXT.md`.

## 1. The reference repo and what applies

**What it is.** The shared link (`https://t.co/LlbAsLTOXF`) redirects to **https://github.com/MartinPuli/taste-library** (inspected at commit `1b67f96`, 2026-10-14). The repo collects design references for coding agents. It holds 40 website captures of companies from Cosign's "High Taste" ranking (Linear, Apple, Notion, Vercel, Perplexity, Nothing, and others), extracted with TasteLabs. Each reference has a profile (`profiles/<id>.md`) and per-section JSON (`data/sections/<id>/{typography,colors,surfaces,elevation,interactions,actions,layout,data_display,...}.json`) with source pointers and a review record. It also includes a selection workflow (`AGENTS.md`, `guides/selection.md`, `guides/decision-template.md`), an agent skill (`skills/taste-library/SKILL.md`), and Python query scripts. It is not a component library and contains no reusable code. Every record is a marketing web page marked `extracted_unverified`. The reviews state that hover, focus, animation timing, and responsive layouts were **not checked** (`data/reviews/linear.json` → `not_checked`).

**Which references fit a 400×640 tray widget.** The library has no transit or widget category, so I followed its own method (`guides/selection.md`): start from the primary task, pick one primary reference, and borrow at most one specific pattern from a second.

- **Primary: Linear** (`profiles/linear.md`, productivity/developer-tools). It is the only reference whose captured content is a dense, status-driven tool UI rather than a marketing hero. The patterns that transfer:
  - **Compact status rows that scan quickly.** "Issue List View: a compact row-based list … with priority icons, status indicators, IDs …" (`data/sections/linear/data_display.json` `/data/tables/0`). Our `FilaArribo` plays the same role.
  - **Four text levels instead of two.** Linear separates primary, muted ("Muted Text / Metadata" `#8A8F98`), and faint ("Faint Text / Icons" `#62666D`, used for timestamps and section headers) text (`colors.json` `/data/secondary`).
  - **Depth from surface luminance steps and translucent borders, not shadows.** "translucent white borders (rgba 8-14%) … avoiding drop shadows" (`interactions.json` `/data/insights/0`). Surfaces step up for hover and active states (`surfaces.json` `/data/solid_colors`, "Elevated Surface", "Secondary Surface"). When Linear does use a shadow on popovers, it is black-alpha (`elevation.json` `/data/shadows/levels/2`, `0px 7px 32px #00000059`).
  - **A small label/body type scale with intermediate variable weights.** Body Small 13/19.5, Badge 12/14 at weight 510, Tiny Badge 10/14 (`typography.json` `/data/labels`, `/data/paragraphs`). The insight notes "non-standard intermediate variable weights like 510 … and 590".
  - **Two motion tokens.** UI transitions use `.16s cubic-bezier(.25,.46,.45,.94)` and drawers use `.5s cubic-bezier(.32,.72,0,1)` with a slide-from-bottom drawer (`interactions.json` `/data/animations`, `button-transition`, `drawer-transition`, `slide-from-bottom`). Popovers scale in from 0.9–0.98 over ~0.12–0.18 s (`tooltip-open`, `scale-in`).
- **Secondary, for structure only: Perplexity** (`profiles/perplexity.md`). Its one useful contribution is the principle "Center the page on search and keep secondary functions subordinate". In our widget the primary task is the next train at the current Estación, so the catalogue selector is the secondary function.
- I passed on Nothing and teenage engineering (monochrome/industrial, small labels). Their identity comes from proprietary fonts (Ndot, NType82) and product photography, which we don't have and shouldn't copy. Vercel adds nothing beyond what Linear covers.

All values above are evidence from a dark marketing page. They are inputs to our own tokens, not values to paste in.

## 2. Audit of the current UI against those principles

Line numbers refer to `master` at `adf6aa6`.

**Primary task vs. secondary functions (Perplexity)**
- A1. When an Estación is open, the full Línea/Ramal/Estación selector panel (`app/src/routes/+page.svelte:325-372`) still sits above it, together with the app title (`:281-283`), the Favoritos row (`:285-301`), and the "Cerca tuyo" row (`:303-323`). In a 640px flyout the featured `ProximoArribo` starts roughly halfway down. The secondary function pushes the primary one below the fold.
- A2. With location off, a permanent hint row appears on every open: "Activá la ubicación de Windows…" (`+page.svelte:303-307`). Nothing dismisses it.
- A3. Both chip rows hide their scrollbar (`scrollbar-width: none`, `+page.svelte:570`, `:613`) and don't map the vertical wheel to horizontal scroll. On desktop, Favoritos past ~400px are effectively unreachable and nothing shows that they exist.
- A4. The two chip rows are labelled inconsistently. "Cerca tuyo" has an uppercase label (`:310`, `:618-625`) and Favoritos has none (`:286`). They are told apart only by the icon colour (amber star vs. accent pin, `:590-592`, `:615-617`).

**Semantic colour discipline**
- A5. The comment at `+page.svelte:495` says the Situación colours are "semánticos, no decorativos", but they are reused decoratively in several places:
  - Favorito stars use `--demorado` amber (`:590-592`, `:730-732`).
  - Available Instalaciones are painted `--en_anden` green (`:831-832`).
  - The "Desactualizado" staleness notice uses the `--demorado` palette (`.aviso` default, `:744-754`).

  As a result, amber and green no longer mean only "delayed" and "on platform".
- A6. Light-mode contrast of Situación pills fails WCAG AA for their size. Pill text is 0.75rem (≈11px) semibold (`SituacionPill.svelte:39-40`) on a 16% tint of itself (`:43`). Computed ratios on `--superficie`: en_anden **3.97:1**, demorado **3.97:1**, en_camino 4.22:1, cancelado 4.46:1. All are below 4.5:1. Inside `ProximoArribo` the pill sits on an already tinted card (`ProximoArribo.svelte:55`), so the contrast is lower still. Dark mode passes (≥4.9:1).

**Text hierarchy and type scale (Linear)**
- A7. There are only two text levels, `--texto` and `--texto-suave` (`+page.svelte:491-492`). Tertiary metadata uses the same muted grey as secondary labels: "Actualizado hace…" (`:385`), "N trenes" (`GrupoRamal.svelte:20`), distances (`:296`, `:319`), and `dt` labels (`ProximoArribo.svelte:123-126`).
- A8. There are about 14 ad-hoc font sizes and no scale: 0.7, 0.72, 0.75, 0.78, 0.8, 0.82, 0.83, 0.85, 0.88, 0.9, 1.05, 1.15, 1.25, and 3rem (across `+page.svelte`, `FilaArribo.svelte`, `ProximoArribo.svelte`, `GrupoRamal.svelte`, `BotonAvisar.svelte`). Weights are only 500/600/700.
- A9. The root font is `"Segoe UI Variable Text"` (`+page.svelte:487`), so the 3rem countdown (`ProximoArribo.svelte:90-95`) renders with the *Text* optical size. Segoe UI Variable has a Display optical size meant for exactly that use, and a Small one for the ≤12px labels.

**Surfaces, borders, elevation (Linear)**
- A10. The popover shadow in the Avisar menu uses the text colour: `box-shadow: 0 10px 28px color-mix(in srgb, var(--texto) 18%, transparent)` (`BotonAvisar.svelte:122`). In dark mode `--texto` is `#eceef2`, so the menu gets a **light glow** instead of a shadow.
- A11. Hover states reuse the border token as a fill (`button.icono:hover { background: var(--borde) }`, `+page.svelte:720-723`; `BotonAvisar.svelte:97-99`). Also, `--borde` is an opaque hex (`:493`, `:509`), so it reads wrong on the Situación-tinted `ProximoArribo` card (which compensates with its own `color-mix` border, `ProximoArribo.svelte:56`, `:121`).
- A12. The radius values are 14, 13, 12, 10, 8, 3 and 999px with no tokens (`+page.svelte:562,664,710,750,782`; `GrupoRamal.svelte:45,80`; `BotonAvisar.svelte:87,119`).

**Row density and redundancy (Linear's status list)**
- A13. A delayed row says the delay twice: `+N min` in the meta line (`FilaArribo.svelte:19`) and "Demorado +N min" in the pill (`arribos.ts:48`, rendered at `FilaArribo.svelte:23`). The row never shows the *estimated* time, only the scheduled one, so the user has to add the delay mentally. `ProximoArribo` has the same issue (`ProximoArribo.svelte:36-37`).
- A14. Row columns are `4.2rem 1fr auto auto` (`FilaArribo.svelte:30`). A long pill such as "Demorado +12 min" plus the bell squeezes the Destino (the item that matters) into an ellipsis at 400px.
- A15. When only one Ramal is shown (Favorito acotado or "Solo el ramal elegido"), the `GrupoRamal` header (`GrupoRamal.svelte:18-21`) repeats the Ramal name already in the station subtitle (`+page.svelte:380`).

**Keyboard, accessibility, motion**
- A16. Escape always hides the window (`+page.svelte:269`), even when the Avisar menu is open. Pressing Escape to close the menu closes the whole panel. The menu has `role="menu"` (`BotonAvisar.svelte:63`) but no arrow-key handling and no focus move, which that role implies.
- A17. Icon buttons are named only by `title`. Refresh (`+page.svelte:399-407`) has no `aria-label`, and the star (`:390-398`) and bell (`BotonAvisar.svelte:45-55`) rely on `title` too. On touch, `title` never shows.
- A18. The motion-sensitive CSS transitions are not reduced-motion-guarded: chip and icon `:active` scale (`+page.svelte:596-598`, `:724-726`), caret rotation (`:808-812`), and 0.4–0.5s colour fades (`ProximoArribo.svelte:57-59`, `SituacionPill.svelte:45-48`). Durations are spread across 0.15, 0.2, 0.4 and 0.5s with plain `ease`, while JS transitions use `EASE` (`movimiento.ts:10`). The `latido` pulse runs indefinitely (`SituacionPill.svelte:61-64`) on every en_anden pill, including in the Ramal lists, so several things can pulse at once.

## 3. Prioritized enhancements

Priority: **P1** = bug or accessibility fix, **P2** = foundation that later items build on, **P3** = layout change or room for planned features.

### P1. Fixes

**E1. Escape closes the innermost layer first, and the Avisar menu becomes keyboard-complete** (A16)
- *Change:* In `alTeclear`, skip `ventana.hide()` when the event was already handled (`e.defaultPrevented`). In `BotonAvisar`, handle Escape on the menu (close it, return focus to the bell, `preventDefault`), move focus to the first `menuitem` on open, and support ArrowUp/ArrowDown. An alternative is to drop `role="menu"` and use a plain group of buttons.
- *Why:* The standard dismissal order for layered UI. Linear treats context menus and popovers as their own layer (`interactions.json` `context-menu-in`, `scale-in`). Today Escape is destructive.
- *Files:* `app/src/routes/+page.svelte`, `app/src/lib/components/BotonAvisar.svelte`
- *Effort:* low. *Android:* yes. The system back gesture needs the same "innermost first" rule.

**E2. Accessible names on every icon button** (A17)
- *Change:* Add `aria-label` alongside `title` on refresh, the star, and the bell. The bell label should include the Servicio, e.g. `Avisarme del tren a {destino} de las {hora}`. Otherwise a row of identical "Avisarme de este tren" buttons can't be told apart.
- *Why:* Screen readers and touch both lose `title`. This is already flagged in `.scratch/android/spec.md` (Hover section).
- *Files:* `+page.svelte`, `BotonAvisar.svelte`
- *Effort:* low. *Android:* yes (TalkBack).

**E3. Pill contrast in light mode** (A6)
- *Change:* Add an on-tint text token per Situación (`--sit-texto`), darker than the fill hue in light mode only. For example, mix `--sit` 75% with `--texto`, then check ≥4.5:1 against the 16% tint *and* against the tinted `ProximoArribo` card. Keep `--sit` itself for bars, borders and the big countdown, which are large or non-text.
- *Why:* WCAG AA for 11px text. Linear's split between an accent colour and its text-on-surface use is the same idea (`colors.json` `when_to_use` lists separate text vs. border roles).
- *Files:* `+page.svelte` (`:root` tokens, `.sit-*` classes), `SituacionPill.svelte`, `FilaArribo.svelte` (`.sit-en_anden .cuenta`)
- *Effort:* low. *Android:* yes. Outdoor phone use makes contrast matter more.

**E4. Popover elevation that works in dark mode** (A10, A11)
- *Change:* Replace the `--texto`-based shadow with a black-alpha token: `--sombra-flotante: 0 7px 32px rgb(0 0 0 / .35)` in dark, a lighter one in light. In dark mode, lift the menu with a one-step-lighter surface (`--superficie-elevada`) plus the translucent border from E6.
- *Why:* Linear's popover shadow is black-alpha (`elevation.json` `/data/shadows/levels/2`), and its dark-mode depth comes from luminance steps (`surfaces.json`, "Secondary Panel" for menus vs. "Panel Background").
- *Files:* `BotonAvisar.svelte`, `+page.svelte` (`:root`)
- *Effort:* low. *Android:* yes.

**E5. Keep the Situación colours for Situación only** (A5)
- *Change:*
  - Give Favoritos their own neutral treatment: a filled star in `--texto-suave`, or `--acento` when active. Don't use `--demorado`.
  - Show available Instalaciones as neutral chips with a phosphor `Check` icon, and unavailable ones as faint text with no strike-through.
  - Give "Desactualizado" a neutral notice variant (`--texto-suave` icon on `--superficie-elevada`). Keep red for real errors (`fuenteCambiada`).
- *Why:* Linear keeps its single brand colour for "Primary buttons, Brand accents, Focus states" and leaves the rest monochrome (`surfaces.json` "Primary Brand"). Colour that always means one thing makes the widget readable at a glance, which is its whole job (ADR 0003).
- *Files:* `+page.svelte` (`.chip svg`, `button.icono.marcado`, `.aviso`, `.servicios li`)
- *Effort:* low. *Android:* yes.

### P2. Foundations

**E6. Token layer: four text levels, surface steps, translucent borders, radii**
- *Change:* In the `:root` block (`+page.svelte:484-518`), add:
  - `--texto-tenue`, a third level below `--texto-suave`. Use it for "Actualizado hace…", "N trenes", distances, and `dt`.
  - `--superficie-elevada` and `--superficie-hover`, so hover stops reusing `--borde`.
  - `--borde` as an alpha colour: `rgb(0 0 0 / .09)` in light, `rgb(255 255 255 / .08)` in dark. It then works on tinted cards, and the special-case borders in `ProximoArribo` can be deleted.
  - Radii `--radio-s: 8px`, `--radio-m: 12px`, `--radio-l: 14px`, `--radio-pill: 999px`.

  Optionally move the tokens to `app/src/app.css` so they stop living in a page's `<style>`.
- *Why:* Linear's primary/muted/faint split and its "translucent white borders (rgba 8-14%)" (`colors.json` `/data/secondary`; `interactions.json` `/data/insights/0`). It also fixes A7, A11 and A12 in one place.
- *Files:* `+page.svelte`, `ProximoArribo.svelte`, `GrupoRamal.svelte`, `FilaArribo.svelte`, `BotonAvisar.svelte`, optionally a new `app/src/app.css` imported from `+layout.ts`
- *Effort:* low–medium (mostly search and replace). *Android:* yes. One token layer serves both platforms, and dark/light keeps working through `prefers-color-scheme`.

**E7. A five-step type scale on Segoe UI Variable's optical sizes**
- *Change:* Replace the ~14 sizes with five tokens:
  - `--t-micro` 11px/14 weight 560, for `dt`, section labels and badges
  - `--t-meta` 12.5px/18, for row meta and timestamps
  - `--t-cuerpo` 14px/20, for Destino, chips and selects
  - `--t-titulo` 18px/24 weight 600, for the Estación name
  - `--t-cuenta` 48px/1 weight 650, for the countdown

  Use `font-family: "Segoe UI Variable Display"` for `--t-titulo` and `--t-cuenta`, and `"Segoe UI Variable Small"` for `--t-micro`, with `font-variation-settings`/`font-weight` at intermediate values. Fallbacks stay `"Segoe UI", system-ui, sans-serif`, which on Android resolves to Roboto, and the scale still holds.
- *Why:* Linear's label/body tiers (12/14 at 510, 13/19.5) and its use of "intermediate variable weights" (`typography.json` `/data/labels`, `/data/insights`). We get the same effect from the system variable font with no font loading, which keeps the native feel the code comment at `+page.svelte:486` asks for.
- *Files:* every `.svelte` with `font-size`
- *Effort:* low–medium. *Android:* yes. Size tokens are the natural place to scale up under `pointer: coarse`.

**E8. Motion tokens and full reduced-motion coverage** (A18)
- *Change:*
  - Define `--mov-ui: 160ms cubic-bezier(.25,.46,.45,.94)` for hover, press and colour, and `--mov-panel: 320ms` for in/out using the existing `EASE` from `movimiento.ts`, mirrored as CSS `cubic-bezier(.25,1,.5,1)`.
  - Under `prefers-reduced-motion: reduce`, zero `transform` transitions (chip and icon press scale, caret).
  - Shorten the 0.5s Situación colour fades to `--mov-ui`.
  - Limit `latido` to the `arribando` class, or to `ProximoArribo` only, instead of every en_anden pill in the lists.
- *Why:* Linear uses one UI easing at .16s and one drawer easing (`interactions.json` `button-transition`, `drawer-transition`). With fewer pulses competing, the one that matters reads as urgent.
- *Files:* `+page.svelte`, `SituacionPill.svelte`, `ProximoArribo.svelte`, `FilaArribo.svelte`, `GrupoRamal.svelte`, `BotonAvisar.svelte`, `app/src/lib/movimiento.ts` (export the CSS curve alongside `EASE`)
- *Effort:* low. *Android:* yes.

### P3. Layout and room for what's planned

**E9. Collapse the selector once an Estación is open** (A1)
- *Change:* While `actual !== null`, render the selector as a single compact row, e.g. `[Sarmiento · Moreno ▸ Cambiar]`, that expands the existing three `<select>`s plus "Solo el ramal elegido" in place with `slide`. Keep the selector expanded on first run (`ultimo_abierto === null`) and when `errorCatalogo` is set. Fold the "OpenTrains" `h1` into a visually hidden heading (the window has no chrome to need it). Together these move `ProximoArribo` up by roughly 150–180px.
- *Why:* Perplexity's "keep secondary functions subordinate". The panel is opened to answer "when is my next train", not to browse the catalogue.
- *Make room for:* The planned **origin→destination filter** belongs in this collapsed row, as a second chip "→ Destino" next to the Estación. Design the row as a flex container of chips now, so that filter slots in without another layout change.
- *Files:* `+page.svelte`
- *Effort:* medium. *Android:* yes. On a phone the selector would otherwise fill the first screen.

**E10. Arribo rows: show the estimated time, say the delay once, protect the Destino** (A13, A14)
- *Change:* In `FilaArribo` and `ProximoArribo`, when `demora_minutos` is set, show `hora(momento)` as the main time and the scheduled one struck through in `--texto-tenue`: `21:55 ~~21:48~~`. Drop the separate `<em>+N min</em>`. In rows, shorten the pill label to the bare Situación ("Demorado"), since the time already carries the delay. Give the `.destino` column priority: `grid-template-columns: 4.2rem minmax(0,1fr) auto auto` and `max-width` with an ellipsis on the pill. `programado` stays label-less, as it is today.
- *Why:* Linear's list row is "scannable", with one status indicator per row and metadata in a muted tier (`data_display.json` `/data/tables/0`; `colors.json` "Muted Text / Metadata"). The time the user needs is the estimated one.
- *Files:* `FilaArribo.svelte`, `ProximoArribo.svelte`, `app/src/lib/arribos.ts` (the `SITUACIONES.demorado.etiqueta` variant for compact use), `app/src/lib/formato.ts` (no change needed, since `hora(momento)` already exists)
- *Effort:* low–medium. *Android:* yes. Narrow widths make column priority essential.

**E11. Chip rows: one labelled pattern, visible overflow, mouse-wheel scroll** (A3, A4)
- *Change:* Treat Favoritos and Estaciones cercanas as one strip component with consistent leading icons (star / pin) and no uppercase label, or a matching label on both. Add an edge fade mask (`mask-image: linear-gradient(to right, #000 85%, transparent)`) only when `scrollWidth > clientWidth`. Translate vertical `wheel` deltas to `scrollLeft` on desktop. Alternatively, cap Favoritos at one wrapped line plus "+N".
- *Why:* An overflow nobody can see is functionally missing. Linear's small, consistent chips use one pattern per role (`actions.json`).
- *Files:* `+page.svelte`, optionally a new `app/src/lib/components/FilaChips.svelte`
- *Effort:* low. *Android:* yes. Touch swipe already works, and the fade shows there is more.

**E12. Dismissible location hint** (A2)
- *Change:* Replace the permanent "Activá la ubicación…" row with a small pin chip at the end of the strip ("Activar ubicación"). It opens Windows location settings (`ms-settings:privacy-location`) and can be dismissed. Store the dismissal in `Preferencias`, which is per-user and persisted (unlike browser storage).
- *Why:* Secondary information that never goes away competes with the primary task (Perplexity principle).
- *Files:* `+page.svelte`. The dismiss flag needs a small `Preferencias` addition across `app/src/lib/api.ts` and the storage crate.
- *Effort:* low (UI), medium if persisted. *Android:* yes. On Android the chip triggers the runtime permission prompt instead.

**E13. Drop the redundant Ramal header when only one Ramal is shown** (A15)
- *Change:* Pass a `soloUno` prop (true when `grupos.length === 1` and the Ramal equals `actual.ramal` or the featured Arribo's Ramal) and render the list without the `h3` header, keeping the "Ver N más" footer.
- *Why:* Less chrome for the same information. Linear's list rows carry their own context.
- *Files:* `GrupoRamal.svelte`, `+page.svelte`
- *Effort:* low. *Android:* yes.

**E14. Sticky Estación header inside the flyout**
- *Change:* Make the `.titulo` block (Estación name, subtitle, star, refresh) and the stale/error notice `position: sticky; top: 0` with `--fondo` behind them and a hairline that appears on scroll. Scrolling through a long list then keeps the station, freshness and refresh visible.
- *Why:* Linear's `sticky_header` global pattern (`interactions.json` `/data/global_patterns`). With E9, the station header is the natural top of the content.
- *Files:* `+page.svelte`
- *Effort:* low–medium. *Android:* yes. It also becomes the app bar on a phone.

**E15. Interaction scaffolding for the planned features (structure only, no feature logic)**
- *Change:* Prepare four slots so the planned work lands without another layout pass:
  1. **Servicio detail sheet.** Make each `FilaArribo` (and the `ProximoArribo` body) a full-width button that opens a bottom sheet over the panel, using Linear's `slide-from-bottom` with `cubic-bezier(.32,.72,0,1)`. The sheet hosts the planned **stop-by-stop service detail** and the **Ramal line diagram**. The bell stays a separate nested button, so it needs `stopPropagation` and the row must not be a `<button>` wrapping a `<button>`. Use a clickable row with an inner "details" button, or a grid with two sibling buttons.
  2. **Notice slot.** Add a single slot above `ProximoArribo` that reuses the `.aviso` component with an `operador` variant, for the planned **operator notices**. Give the **highlighted platform** a dedicated emphasis style in `ProximoArribo`'s Andén cell (larger `--t-titulo` value), ready for when the data arrives.
  3. **Meta badges.** Define one tiny neutral badge style (`--t-micro`, `--superficie-elevada`, no Situación colour) for row metadata. It is meant for **punctuality history**, **cancellation-prone Servicios** and **crowding times**, so those never borrow Situación colours (see E5).
  4. **Station board mode.** Keep `FilaArribo` layout-agnostic (no assumptions about being inside `GrupoRamal`) so a board view can render the same rows in one time-ordered list.
- *Why:* Linear's drawer and dialog patterns (`interactions.json` `drawer-transition`, `dialog-scale-in`). A bottom sheet is also the native Android idiom for detail.
- *Files:* `FilaArribo.svelte`, `ProximoArribo.svelte`, `+page.svelte`, new `app/src/lib/components/Hoja.svelte` (sheet) and `Insignia.svelte` (badge)
- *Effort:* medium. *Android:* yes, strongly. The sheet and the large row tap targets are the phone layout.

Touch targets (44px+ under `pointer: coarse`), `hover: hover` gating, and safe areas are already specified in `.scratch/android/spec.md` (items "Áreas táctiles", "Hover", step 8). Implement them on top of E6/E7 tokens (`--objetivo-min: 44px`) rather than as one-off overrides.

## 4. What not to adopt from taste-library

- **Brand fonts** (Linear's Inter Variable and Berkeley Mono; Nothing's Ndot and NType82). `RIGHTS.md` says no reuse right is granted. Loading web fonts would also break the "native, no font loading" decision (`+page.svelte:486`). Segoe UI Variable's optical sizes and weights give the same hierarchy.
- **Brand hex values and a forced dark theme.** Linear's near-black canvas (`#08090A`) is an identity choice for a marketing page. Keep our own palette and follow `prefers-color-scheme`. The guides themselves warn against mixing brand palettes (`guides/selection.md`).
- **Marketing motion:** `marquee`, `parallax`, `scroll_reveal`, `hover_lift`, image-mask reveals (`linear/interactions.json` `/data/global_patterns`). A glanceable widget should change only when data changes.
- **Glassmorphism / `backdrop-filter`** ("Glass Panel", `linear/surfaces.json`). It costs GPU in WebView2 on a window that refreshes while hidden. If we want translucency, the native Windows Mica/Acrylic window effect is the right layer, not CSS.
- **Pill-shaped inverted primary CTAs.** The widget has no conversion action. Its "primary action" is reading.
- **Generous section spacing** (Linear 96–160px, Perplexity 40vh top space, `layout.json`). It contradicts a 400×640 flyout.
- **Treating extracted values as verified.** Every record is `extracted_unverified`, and hover, focus, timing and responsive behaviour were not checked (`data/reviews/*.json` `not_checked`). Everything above is adapted, not copied.
- **The repo's agent tooling.** The README and `guides/agent-access.md` invite agents to copy `skills/taste-library` into their skills directory and run `scripts/query.py`, `scripts/build.py`, and npm-based e2e checks. None of that was done here. The data was read directly from the shallow clone, and nothing was installed or executed.

## 5. Suggested shipping order

Each step is independent, small enough for one commit, and verifiable with the existing CDP live-check scripts (`d4cda00`) in light and dark mode.

1. **E1 + E2.** Escape order, menu keyboard, aria-labels. Pure behaviour fixes with no visual change.
2. **E4.** Popover shadow fix (one line plus one token).
3. **E6.** Token layer (text levels, surfaces, alpha borders, radii). Visual parity is the goal; only A11's hover and the dark menu should look different.
4. **E3.** On-tint Situación text, measured against the new surfaces from step 3.
5. **E5.** Remove decorative uses of Situación colours.
6. **E7.** Type scale and optical sizes.
7. **E8.** Motion tokens and reduced-motion coverage.
8. **E10.** Estimated time, delay shown once, Destino priority in rows.
9. **E13.** Single-Ramal header removal.
10. **E11 + E12.** Chip strip overflow and the dismissible location hint.
11. **E9.** Collapsible selector (the biggest layout change; it benefits from steps 3–7 being in place).
12. **E14.** Sticky Estación header (after E9 settles the top of the panel).
13. **E15.** Sheet, notice slot, badge and board-ready rows, landed just before the first planned feature that needs each slot.
