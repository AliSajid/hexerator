mod hex_conv;

use egui_inspect::{derive::Inspect, inspect};
use egui_sfml::{
    egui::{self, color::rgb_from_hsv, Button, Window},
    SfEgui,
};
use sfml::{
    graphics::{
        Color, Font, PrimitiveType, Rect, RectangleShape, RenderStates, RenderTarget, RenderWindow,
        Shape, Vertex,
    },
    system::Vector2,
    window::{ContextSettings, Event, Key, Style},
};

#[derive(PartialEq, Debug, Inspect)]
enum EditTarget {
    Hex,
    Text,
}

impl EditTarget {
    fn switch(&mut self) {
        *self = match self {
            EditTarget::Hex => EditTarget::Text,
            EditTarget::Text => EditTarget::Hex,
        }
    }
}

fn main() {
    let path = std::env::args_os()
        .nth(1)
        .expect("Need file path as argument");
    let mut data = std::fs::read(&path).unwrap();
    let mut w = RenderWindow::new(
        (1920, 1080),
        "hello",
        Style::NONE,
        &ContextSettings::default(),
    );
    w.set_vertical_sync_enabled(true);
    w.set_position(Vector2::new(0, 0));
    let mut sf_egui = SfEgui::new(&w);
    let f = Font::from_memory(include_bytes!("../DejaVuSansMono.ttf")).unwrap();
    let mut vertices = Vec::new();
    let mut rows = 67;
    let mut cols = 48;
    let mut starting_offset = 0;
    let mut colorize = true;
    let mut cursor: usize = 0;
    let mut edit_target = EditTarget::Hex;
    let mut dirty = false;

    while w.is_open() {
        // region: event handling
        while let Some(event) = w.poll_event() {
            sf_egui.add_event(&event);
            match event {
                Event::Closed => w.close(),
                Event::KeyPressed { code, shift, .. } => match code {
                    Key::Up => {
                        cursor = cursor.saturating_sub(cols);
                        if cursor < starting_offset {
                            starting_offset -= cols;
                        }
                    }
                    Key::Down => {
                        cursor += cols;
                        if cursor >= starting_offset + rows * cols {
                            starting_offset += cols;
                        }
                    }
                    Key::Left => cursor = cursor.saturating_sub(1),
                    Key::Right => cursor += 1,
                    Key::PageUp => {
                        let amount = rows * cols;
                        if starting_offset >= amount {
                            starting_offset -= amount;
                            cursor -= amount;
                        } else {
                            starting_offset = 0
                        }
                    }
                    Key::PageDown => {
                        let amount = rows * cols;
                        if starting_offset + amount < data.len() {
                            starting_offset += amount;
                            cursor += amount;
                        }
                    }
                    Key::Home => {
                        starting_offset = 0;
                        cursor = 0;
                    }
                    Key::End => {
                        let pos = data.len() - rows * cols;
                        starting_offset = pos;
                        cursor = pos;
                    }
                    Key::Tab if shift => {
                        edit_target.switch();
                    }
                    _ => {}
                },
                Event::TextEntered { unicode } => match edit_target {
                    EditTarget::Hex => todo!(),
                    EditTarget::Text => {
                        if unicode.is_ascii() {
                            data[cursor] = unicode as u8;
                            dirty = true;
                            cursor += 1;
                        }
                    }
                },
                _ => {}
            }
        }
        // endregion
        w.clear(Color::BLACK);
        let mut rs = RenderStates::default();
        vertices.clear();
        sf_egui.do_frame(|ctx| {
            Window::new("Hexerator").show(ctx, |ui| {
                // region: debug panel
                inspect! {
                    ui,
                    rows,
                    cols,
                    starting_offset,
                    cursor,
                    colorize,
                    edit_target
                }
                if ui.add_enabled(dirty, Button::new("Reload")).clicked() {
                    data = std::fs::read(&path).unwrap();
                    dirty = false;
                }
                if ui.add_enabled(dirty, Button::new("Save")).clicked() {
                    std::fs::write(&path, &data).unwrap();
                }
                // endregion
            });
        });
        // region: hex display
        let mut idx = starting_offset;
        'display: for y in 0..rows {
            for x in 0..cols {
                if idx == data.len() {
                    break 'display;
                }
                let byte = data[idx];
                if idx == cursor {
                    draw_cursor(
                        x as f32 * 26.0,
                        y as f32 * 16.0,
                        &mut w,
                        edit_target == EditTarget::Hex,
                    );
                }
                let [g1, g2] = hex_conv::byte_to_hex_digits(byte);
                let [r, g, b] = rgb_from_hsv((byte as f32 / 255.0, 1.0, 1.0));
                let c = if colorize {
                    Color::rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
                } else {
                    Color::WHITE
                };
                draw_glyph(
                    &f,
                    &mut vertices,
                    x as f32 * 26.0,
                    y as f32 * 16.0,
                    g1 as u32,
                    c,
                );
                draw_glyph(
                    &f,
                    &mut vertices,
                    x as f32 * 26.0 + 11.0,
                    y as f32 * 16.0,
                    g2 as u32,
                    c,
                );
                idx += 1;
            }
        }
        // endregion
        // region: ascii display
        idx = starting_offset;
        'asciidisplay: for y in 0..rows {
            for x in 0..cols {
                if idx == data.len() {
                    break 'asciidisplay;
                }
                let byte = data[idx];
                let [r, g, b] = rgb_from_hsv((byte as f32 / 255.0, 1.0, 1.0));
                let c = if colorize {
                    Color::rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
                } else {
                    Color::WHITE
                };
                if idx == cursor {
                    draw_cursor(
                        (x + cols * 2 + 1) as f32 * 13.0,
                        y as f32 * 16.0,
                        &mut w,
                        edit_target == EditTarget::Text,
                    );
                }
                draw_glyph(
                    &f,
                    &mut vertices,
                    (x + cols * 2 + 1) as f32 * 13.0,
                    y as f32 * 16.0,
                    byte as u32,
                    c,
                );
                idx += 1;
            }
        }
        // endregion
        rs.set_texture(Some(f.texture(10)));
        w.draw_primitives(&vertices, PrimitiveType::QUADS, &rs);
        rs.set_texture(None);
        sf_egui.draw(&mut w, None);
        w.display();
    }
}

fn draw_cursor(x: f32, y: f32, w: &mut RenderWindow, active: bool) {
    let mut rs = RectangleShape::from_rect(Rect {
        left: x,
        top: y,
        width: 10.0,
        height: 10.0,
    });
    rs.set_fill_color(Color::TRANSPARENT);
    if active {
        rs.set_outline_thickness(2.0);
        rs.set_outline_color(Color::YELLOW);
    } else {
        rs.set_outline_thickness(1.0);
        rs.set_outline_color(Color::BLUE);
    }
    w.draw(&rs);
}

fn draw_glyph(font: &Font, vertices: &mut Vec<Vertex>, x: f32, y: f32, glyph: u32, color: Color) {
    let g = font.glyph(glyph, 10, false, 0.0);
    let r = g.texture_rect();
    vertices.push(Vertex {
        position: Vector2::new(x, y),
        color,
        tex_coords: Vector2::new(r.left as f32, r.top as f32),
    });
    vertices.push(Vertex {
        position: Vector2::new(x, y + 10.0),
        color,
        tex_coords: Vector2::new(r.left as f32, (r.top + r.height) as f32),
    });
    vertices.push(Vertex {
        position: Vector2::new(x + 10.0, y + 10.0),
        color,
        tex_coords: Vector2::new((r.left + r.width) as f32, (r.top + r.height) as f32),
    });
    vertices.push(Vertex {
        position: Vector2::new(x + 10.0, y),
        color,
        tex_coords: Vector2::new((r.left + r.width) as f32, r.top as f32),
    });
}
