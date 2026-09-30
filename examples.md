# Examples

Ask an agent to reshape `src/main.rs` toward one of these examples (or anything else you think of), then reinstall from the clone. 

## 1. Default (this repo)

Black on white, `Ctrl T` for white on black. IBM Plex Mono, word wrap, absolute line numbers with the current line in full ink. Asks to stay on top.

## 2. Permanent pin on Wayland

No source edit. Tell your compositor once.

KDE: System Settings, Window Management, Window Rules. Add a rule for window class `wisp`, set Keep above to Force.

GNOME: no rules. Right-click the title bar, Always on Top, each launch.

Sway:

```
for_window [app_id="wisp"] floating enable, sticky enable
```

niri (floating windows sit above tiled ones):

```kdl
window-rule {
    match app-id="^wisp$"
    open-floating true
}
```

labwc (`rc.xml`):

```xml
<windowRules>
  <windowRule identifier="wisp">
    <action name="ToggleAlwaysOnTop"/>
  </windowRule>
</windowRules>
```

Hyprland: a `float` and `pin` window rule matching class `wisp`. The rule syntax changes between releases; check the wiki for yours.

## 3. Starts dark, bigger

```rust
const FONT: f32 = 20.0;
const SIZE: [f32; 2] = [520.0, 360.0];
const START: Theme = Theme::Dark;
```

## 4. Another typeface

Drop any TTF or OTF into `fonts/` (with its license) and point at it.

```rust
const TYPEFACE: &[u8] = include_bytes!("../fonts/JetBrainsMono-Regular.ttf");
```

## 5. Never on top

Drop `.with_always_on_top()` from `main`. Now Windows behaves like Wayland.

## 6. X11 too

Add `"x11"` to the eframe features and the winit features in `Cargo.toml`.
