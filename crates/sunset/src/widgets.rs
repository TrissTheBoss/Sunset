//! Modular UI widgets: a runtime registry of custom textures and widgets
//! (buttons first) so mods can extend the interface without touching the core
//! HUD/screen renderer. Mirrors the block/item/recipe registries in core: every
//! family is addressed by a namespaced id and registered at runtime.

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use crate::ui::{self, Screen};

// ---------------------------------------------------------------------------
// Textures
// ---------------------------------------------------------------------------

/// Runtime-registered UI textures, merged into the UI atlas at load time and
/// referenced by name from widgets.
static TEXTURES: LazyLock<RwLock<HashMap<String, image::RgbaImage>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Register a custom UI texture (mod content). It is packed into the UI atlas
/// automatically and becomes addressable by `name` from widgets.
pub fn register_texture(name: &str, image: image::RgbaImage) {
    TEXTURES
        .write()
        .expect("ui texture registry")
        .insert(name.to_string(), image);
}

/// Snapshot of every registered custom texture, for atlas building.
pub fn custom_textures() -> HashMap<String, image::RgbaImage> {
    TEXTURES.read().expect("ui texture registry").clone()
}

// ---------------------------------------------------------------------------
// Widgets
// ---------------------------------------------------------------------------

/// Effects a widget callback can request, applied by the main loop after the
/// callback returns. This keeps widget callbacks pure functions of their
/// arguments — no direct access to window, GPU, or simulation state.
#[derive(Debug, Default)]
pub struct UiContext {
    /// Exit the game.
    pub quit: bool,
    /// Toggle fullscreen.
    pub toggle_fullscreen: bool,
    /// Open a screen.
    pub open_screen: Option<Screen>,
    /// Close the currently-open screen.
    pub close_screen: bool,
}

/// A clickable button with an automatically-applied texture.
pub struct Button {
    /// Namespaced id, e.g. `"mymod:settings"`. Informational identity matching
    /// the other content families; read by mod tooling and future lookups.
    #[allow(dead_code)]
    pub id: &'static str,
    /// Texture name to draw (a jar-registered name or one added via
    /// [`register_texture`]).
    pub texture: &'static str,
    /// Optional texture drawn while the cursor hovers over the button.
    pub hover_texture: Option<&'static str>,
    /// Top-left position in GUI px (scale 1).
    pub x: f32,
    pub y: f32,
    /// Size in GUI px (scale 1).
    pub width: f32,
    pub height: f32,
    /// Optional label, drawn centred on the button.
    pub label: Option<&'static str>,
    /// Invoked on a left-click over the button.
    pub on_click: fn(&mut UiContext),
}

/// A registered UI widget. Buttons are the first kind; labels, panels, and
/// images extend this enum without changing the registry or the render loop.
pub enum Widget {
    Button(Button),
}

static WIDGETS: LazyLock<RwLock<Vec<Widget>>> = LazyLock::new(|| RwLock::new(Vec::new()));

/// Register a widget (mod content), returning its index. It is drawn and
/// hit-tested automatically.
pub fn register_widget(widget: Widget) -> usize {
    let mut widgets = WIDGETS.write().expect("ui widget registry");
    widgets.push(widget);
    widgets.len() - 1
}

/// Register a button widget (convenience wrapper), returning its index.
pub fn register_button(button: Button) -> usize {
    register_widget(Widget::Button(button))
}

fn button_rect(btn: &Button, screen: (u32, u32)) -> (f32, f32, f32, f32) {
    let s = ui::gui_scale(screen.0, screen.1) as f32;
    (btn.x * s, btn.y * s, btn.width * s, btn.height * s)
}

/// Draw every registered widget. `cursor` is the mouse position (screen px).
pub fn draw_widgets(
    b: &mut ui::UiBuilder,
    a: &ui::UiAssets,
    screen: (u32, u32),
    cursor: (f32, f32),
) {
    let widgets = WIDGETS.read().expect("ui widget registry");
    for w in widgets.iter() {
        let Widget::Button(btn) = w;
        let (x, y, w, h) = button_rect(btn, screen);
        let hovering = cursor.0 >= x && cursor.0 < x + w && cursor.1 >= y && cursor.1 < y + h;
        let tex = match (hovering, btn.hover_texture) {
            (true, Some(hover)) => hover,
            _ => btn.texture,
        };
        if let Some(uv) = a.uvs.get(tex) {
            b.quad(x, y, w, h, *uv, ui::pack_rgba(255, 255, 255, 255));
        }
        if let Some(label) = btn.label {
            // Standard Minecraft font size (8 GUI px tall), centred in the button.
            let fs = ui::gui_scale(screen.0, screen.1) as f32;
            let tw = ui::text_width(label, fs);
            let lx = x + (w - tw) / 2.0;
            let ly = y + (h - 8.0 * fs) / 2.0;
            ui::draw_text(b, a, lx, ly, fs, label, ui::pack_rgba(255, 255, 255, 255));
        }
    }
}

/// The index of the topmost widget whose rect contains `(x, y)`, if any.
pub fn hit_widget(x: f32, y: f32, screen: (u32, u32)) -> Option<usize> {
    let widgets = WIDGETS.read().expect("ui widget registry");
    for (i, w) in widgets.iter().enumerate() {
        let Widget::Button(btn) = w;
        let (bx, by, bw, bh) = button_rect(btn, screen);
        if x >= bx && x < bx + bw && y >= by && y < by + bh {
            return Some(i);
        }
    }
    None
}

/// Invoke the callback of widget `index`, mutating `ctx` with its effects.
pub fn click_widget(index: usize, ctx: &mut UiContext) {
    let widgets = WIDGETS.read().expect("ui widget registry");
    let Some(w) = widgets.get(index) else {
        return;
    };
    let Widget::Button(btn) = w;
    (btn.on_click)(ctx);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(w: u32, h: u32, px: image::Rgba<u8>) -> image::RgbaImage {
        let mut img = image::RgbaImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                img.put_pixel(x, y, px);
            }
        }
        img
    }

    #[test]
    fn register_texture_and_button() {
        register_texture("test:tex", square(16, 16, image::Rgba([255, 0, 0, 255])));
        assert!(custom_textures().contains_key("test:tex"));

        let idx = register_button(Button {
            id: "test:btn",
            texture: "test:tex",
            hover_texture: None,
            x: 10.0,
            y: 10.0,
            width: 32.0,
            height: 16.0,
            label: None,
            on_click: |_| {},
        });
        // A second registration gets the next index (indices are dense/stable).
        let idx2 = register_button(Button {
            id: "test:btn2",
            texture: "test:tex",
            hover_texture: None,
            x: 0.0,
            y: 0.0,
            width: 8.0,
            height: 8.0,
            label: None,
            on_click: |_| {},
        });
        assert_eq!(idx2, idx + 1);
    }

    #[test]
    fn hit_widget_hits_button_rect() {
        register_button(Button {
            id: "test:hit",
            texture: "test:tex",
            hover_texture: None,
            x: 0.0,
            y: 0.0,
            width: 20.0,
            height: 20.0,
            label: None,
            on_click: |_| {},
        });
        // gui_scale(1920,1080) == 3, so the 20x20 button spans 60x60 px.
        let screen = (1920, 1080);
        assert!(hit_widget(10.0, 10.0, screen).is_some());
        assert!(hit_widget(100.0, 100.0, screen).is_none());
    }

    #[test]
    fn click_widget_runs_callback() {
        let idx = register_button(Button {
            id: "test:quit",
            texture: "test:tex",
            hover_texture: None,
            x: 0.0,
            y: 0.0,
            width: 20.0,
            height: 20.0,
            label: None,
            on_click: |ctx| ctx.quit = true,
        });
        let mut ctx = UiContext::default();
        click_widget(idx, &mut ctx);
        assert!(ctx.quit);
    }
}
