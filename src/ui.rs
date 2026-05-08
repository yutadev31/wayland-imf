use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom, Write},
    os::fd::AsFd,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use ab_glyph::{Font, FontVec, PxScale, ScaleFont, point};
use imf_core::Context;
use wayland_client::{
    Connection, Dispatch, QueueHandle,
    protocol::{wl_buffer, wl_compositor, wl_shm, wl_shm_pool, wl_surface},
};
use wayland_protocols_misc::zwp_input_method_v2::client::{
    zwp_input_method_v2, zwp_input_popup_surface_v2,
};

use crate::state::State;

const PANEL_WIDTH: u32 = 420;
const PANEL_PADDING: i32 = 4;
const ROW_HEIGHT: i32 = 34;
const MAX_VISIBLE_CANDIDATES: usize = 8;

#[derive(Default)]
pub struct UiState {
    pub popup: Option<CandidatePopup>,
}

pub struct CandidatePopup {
    surface: wl_surface::WlSurface,
    _popup: zwp_input_popup_surface_v2::ZwpInputPopupSurfaceV2,
    _pool: wl_shm_pool::WlShmPool,
    buffer: wl_buffer::WlBuffer,
    _file: File,
    width: u32,
    height: u32,
    stride: u32,
    font: Option<FontVec>,
    visible: bool,
    last_panel_height: u32,
}

impl CandidatePopup {
    pub fn new(
        compositor: &wl_compositor::WlCompositor,
        shm: &wl_shm::WlShm,
        input_method: &zwp_input_method_v2::ZwpInputMethodV2,
        qh: &QueueHandle<State>,
    ) -> std::io::Result<Self> {
        let surface = compositor.create_surface(qh, ());
        let popup = input_method.get_input_popup_surface(&surface, qh, ());

        let width = PANEL_WIDTH;
        let height = (PANEL_PADDING * 2 + ROW_HEIGHT * MAX_VISIBLE_CANDIDATES as i32) as u32;
        let stride = width * 4;
        let size = stride * height;

        let file = create_shared_file(size as u64)?;
        let pool = shm.create_pool(file.as_fd(), size as i32, qh, ());
        let buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            wl_shm::Format::Argb8888,
            qh,
            (),
        );

        Ok(Self {
            surface,
            _popup: popup,
            _pool: pool,
            buffer,
            _file: file,
            width,
            height,
            stride,
            font: load_ui_font(),
            visible: false,
            last_panel_height: 0,
        })
    }

    pub fn render(&mut self, ctx: &Context) {
        let candidate_count = if ctx.preedit_buf.is_empty() {
            0
        } else {
            ctx.candidates.len()
        };

        if candidate_count == 0 {
            self.hide();
            return;
        }

        let visible_count = candidate_count.min(MAX_VISIBLE_CANDIDATES);
        let start_index = visible_start_index(candidate_count, ctx.selected_index);
        let panel_height = (PANEL_PADDING * 2 + ROW_HEIGHT * visible_count as i32) as u32;

        let mut pixels = vec![0; (self.stride * self.height) as usize];

        draw_rect(
            &mut pixels,
            self.width,
            self.height,
            0,
            0,
            self.width as i32,
            panel_height as i32,
            [0x22, 0x24, 0x33, 0xF8],
        );

        for (row, (candidate_index, candidate)) in ctx
            .candidates
            .iter()
            .enumerate()
            .skip(start_index)
            .take(visible_count)
            .enumerate()
        {
            let top = PANEL_PADDING + row as i32 * ROW_HEIGHT;
            let selected = ctx.selected_index == Some(candidate_index);

            if selected {
                draw_rect(
                    &mut pixels,
                    self.width,
                    self.height,
                    8,
                    top,
                    self.width as i32 - 16,
                    ROW_HEIGHT,
                    [0x2B, 0x69, 0xB1, 0xFF],
                );
            }

            let label = format!("{}. {}", candidate_index + 1, candidate);
            let text_color = if selected {
                [0x3b, 0x3e, 0x57, 0xFF]
            } else {
                [0xb8, 0xc3, 0xff, 0xFF]
            };
            draw_text(
                &mut pixels,
                self.width,
                self.height,
                self.font.as_ref(),
                18.0,
                (top + 24) as f32,
                22.0,
                &label,
                text_color,
            );
        }

        if self
            ._file
            .seek(SeekFrom::Start(0))
            .and_then(|_| self._file.write_all(&pixels))
            .is_ok()
        {
            self.surface.attach(Some(&self.buffer), 0, 0);
            let damage_height = self.last_panel_height.max(panel_height);
            self.surface
                .damage(0, 0, self.width as i32, damage_height as i32);
            self.surface.commit();
            self.visible = true;
            self.last_panel_height = panel_height;
        }
    }

    pub fn hide(&mut self) {
        if !self.visible {
            return;
        }

        self.surface.attach(None, 0, 0);
        self.surface.commit();
        self.visible = false;
        self.last_panel_height = 0;
    }
}

fn visible_start_index(candidate_count: usize, selected_index: Option<usize>) -> usize {
    if candidate_count <= MAX_VISIBLE_CANDIDATES {
        return 0;
    }

    let max_start = candidate_count - MAX_VISIBLE_CANDIDATES;
    let selected_index = selected_index.unwrap_or(0).min(candidate_count - 1);

    selected_index
        .saturating_sub(MAX_VISIBLE_CANDIDATES - 1)
        .min(max_start)
}

fn create_shared_file(size: u64) -> std::io::Result<File> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = format!("/tmp/ime-candidate-popup-{}-{}", std::process::id(), unique);

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&path)?;
    file.set_len(size)?;
    let _ = std::fs::remove_file(path);
    Ok(file)
}

fn load_ui_font() -> Option<FontVec> {
    let font_path = Command::new("fc-match")
        .args(["-f", "%{file}\n", "Noto Sans CJK JP"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|path| path.trim().to_string())
        .filter(|path| !path.is_empty())?;

    let bytes = std::fs::read(font_path).ok()?;
    FontVec::try_from_vec_and_index(bytes, 0).ok()
}

fn draw_rect(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    rect_width: i32,
    rect_height: i32,
    color: [u8; 4],
) {
    for py in y.max(0)..(y + rect_height).min(height as i32) {
        for px in x.max(0)..(x + rect_width).min(width as i32) {
            blend_pixel(pixels, width, px as u32, py as u32, color, 1.0);
        }
    }
}

fn draw_text(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    font: Option<&FontVec>,
    x: f32,
    baseline_y: f32,
    size: f32,
    text: &str,
    color: [u8; 4],
) {
    let Some(font) = font else {
        return;
    };

    let scaled = font.as_scaled(PxScale::from(size));
    let mut caret = x;
    let mut previous = None;

    for ch in text.chars() {
        let glyph_id = scaled.glyph_id(ch);
        if let Some(prev) = previous {
            caret += scaled.kern(prev, glyph_id);
        }

        let glyph = glyph_id.with_scale_and_position(size, point(caret, baseline_y));
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|gx, gy, coverage| {
                let px = bounds.min.x as i32 + gx as i32;
                let py = bounds.min.y as i32 + gy as i32;
                if px >= 0 && py >= 0 && (px as u32) < width && (py as u32) < height {
                    blend_pixel(pixels, width, px as u32, py as u32, color, coverage);
                }
            });
        }

        caret += scaled.h_advance(glyph_id);
        previous = Some(glyph_id);
    }
}

fn blend_pixel(pixels: &mut [u8], width: u32, x: u32, y: u32, color: [u8; 4], coverage: f32) {
    let index = ((y * width + x) * 4) as usize;
    let src_alpha = (color[3] as f32 / 255.0) * coverage;
    let dst_alpha = pixels[index + 3] as f32 / 255.0;
    let out_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);

    let blend_channel = |src: u8, dst: u8| -> u8 {
        if out_alpha <= f32::EPSILON {
            return 0;
        }

        let src = src as f32 / 255.0;
        let dst = dst as f32 / 255.0;
        let out = (src * src_alpha + dst * dst_alpha * (1.0 - src_alpha)) / out_alpha;
        (out * 255.0).round() as u8
    };

    pixels[index] = blend_channel(color[2], pixels[index]);
    pixels[index + 1] = blend_channel(color[1], pixels[index + 1]);
    pixels[index + 2] = blend_channel(color[0], pixels[index + 2]);
    pixels[index + 3] = (out_alpha * 255.0).round() as u8;
}

impl Dispatch<wl_shm::WlShm, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm::WlShm,
        _event: wl_shm::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm_pool::WlShmPool,
        _event: wl_shm_pool::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &wl_buffer::WlBuffer,
        _event: wl_buffer::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwp_input_popup_surface_v2::ZwpInputPopupSurfaceV2, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &zwp_input_popup_surface_v2::ZwpInputPopupSurfaceV2,
        _event: zwp_input_popup_surface_v2::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}
