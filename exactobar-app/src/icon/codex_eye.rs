//! Codex "eye" icon drawing.
//!
//! This module contains the drawing methods for the distinctive Codex
//! eye icon, which shows remaining quota as a fill level and can animate
//! with a blink effect.

use tiny_skia::*;

use super::IconRenderer;
use super::colors::{IconColors, create_paint, with_alpha};

/// Codex eye dimensions.
pub const EYE_WIDTH: f32 = 24.0;
pub const EYE_HEIGHT: f32 = 12.0;
pub const PUPIL_RADIUS: f32 = 2.0;
pub const IRIS_RADIUS: f32 = 4.0;

impl IconRenderer {
    /// Draws the distinctive Codex "eye" icon.
    ///
    /// The eye design shows:
    /// - An almond-shaped eye outline
    /// - A fill level representing remaining quota (fills from bottom to top)
    /// - A central pupil/iris
    /// - Optional blink animation (eyelid closing from top)
    pub(super) fn draw_codex_eye(
        &self,
        pixmap: &mut Pixmap,
        fill_percent: f32,
        blink_phase: f32,
        colors: &IconColors,
        stale: bool,
    ) {
        let center_x = self.width as f32 / 2.0;
        let center_y = self.height as f32 / 2.0;

        // Eye dimensions
        let eye_w = EYE_WIDTH;
        let eye_h = EYE_HEIGHT;
        let half_w = eye_w / 2.0;
        let half_h = eye_h / 2.0;

        // Clamp values
        let fill = fill_percent.clamp(0.0, 100.0);
        let blink = blink_phase.clamp(0.0, 1.0);

        // Calculate the effective eye height based on blink
        // As blink increases, the eye closes from top
        let effective_half_h = half_h * (1.0 - blink);

        // Draw the eye outline (almond shape using bezier curves)
        if effective_half_h > 0.5 {
            // Only draw if not fully closed
            let eye_path = self.create_eye_path(center_x, center_y, half_w, effective_half_h);

            // Draw eye background (track color)
            let bg_paint = create_paint(colors.track);
            pixmap.fill_path(
                &eye_path,
                &bg_paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );

            // Draw fill level (from bottom up)
            if fill > 0.0 {
                self.draw_eye_fill(
                    pixmap,
                    center_x,
                    center_y,
                    half_w,
                    effective_half_h,
                    fill,
                    colors,
                    stale,
                );
            }

            // Draw the iris and pupil (only if eye is mostly open)
            if blink < 0.7 {
                let iris_alpha = 1.0 - (blink / 0.7); // Fade out iris as eye closes
                self.draw_eye_iris_and_pupil(pixmap, center_x, center_y, iris_alpha, colors, stale);
            }

            // Draw eye outline stroke
            self.draw_eye_outline(pixmap, center_x, center_y, half_w, effective_half_h, colors);
        } else {
            // Eye is closed - draw a horizontal line
            self.draw_closed_eye_line(pixmap, center_x, center_y, half_w, colors);
        }
    }

    /// Creates the almond-shaped eye path using bezier curves.
    fn create_eye_path(&self, cx: f32, cy: f32, half_w: f32, half_h: f32) -> Path {
        let mut pb = PathBuilder::new();

        // Start at left point of eye
        pb.move_to(cx - half_w, cy);

        // Top curve (left to right)
        pb.cubic_to(
            cx - half_w * 0.5,
            cy - half_h * 1.2, // control point 1
            cx + half_w * 0.5,
            cy - half_h * 1.2, // control point 2
            cx + half_w,
            cy, // end point
        );

        // Bottom curve (right to left)
        pb.cubic_to(
            cx + half_w * 0.5,
            cy + half_h * 1.2, // control point 1
            cx - half_w * 0.5,
            cy + half_h * 1.2, // control point 2
            cx - half_w,
            cy, // end point (back to start)
        );

        pb.close();
        pb.finish().unwrap()
    }

    /// Draws the fill level inside the eye (from bottom up).
    fn draw_eye_fill(
        &self,
        pixmap: &mut Pixmap,
        cx: f32,
        cy: f32,
        half_w: f32,
        half_h: f32,
        fill_percent: f32,
        colors: &IconColors,
        stale: bool,
    ) {
        // Create a clipping path that represents the fill level
        // Fill goes from bottom of eye to a certain height based on percentage
        let fill_height = half_h * 2.0 * (fill_percent / 100.0);
        let fill_top = cy + half_h - fill_height;

        // Create a filled rectangle that we'll clip to the eye shape
        let fill_rect = Rect::from_xywh(cx - half_w, fill_top, half_w * 2.0, fill_height);
        if fill_rect.is_none() {
            return;
        }

        // Create eye mask for clipping
        let mut mask_pixmap = Pixmap::new(self.width, self.height).unwrap();
        let eye_path = self.create_eye_path(cx, cy, half_w, half_h);
        let white = create_paint(Color::WHITE);
        mask_pixmap.fill_path(
            &eye_path,
            &white,
            FillRule::Winding,
            Transform::identity(),
            None,
        );

        // Determine fill color based on percentage and stale state
        let fill_color = if stale {
            colors.fill_stale
        } else {
            self.percent_to_color(fill_percent, colors)
        };

        // Draw the fill with masking
        let fill_paint = create_paint(fill_color);

        // Create a temporary pixmap for the fill
        let mut fill_pixmap = Pixmap::new(self.width, self.height).unwrap();
        fill_pixmap.fill_rect(fill_rect.unwrap(), &fill_paint, Transform::identity(), None);

        // Apply mask: multiply fill by eye shape
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) as usize * 4;
                let mask_alpha = mask_pixmap.data()[idx + 3] as f32 / 255.0;

                if mask_alpha > 0.0 {
                    let fill_r = fill_pixmap.data()[idx];
                    let fill_g = fill_pixmap.data()[idx + 1];
                    let fill_b = fill_pixmap.data()[idx + 2];
                    let fill_a = fill_pixmap.data()[idx + 3];

                    if fill_a > 0 {
                        // Blend with existing pixel
                        let src_a = (fill_a as f32 * mask_alpha) as u8;
                        let color = Color::from_rgba8(fill_r, fill_g, fill_b, src_a);
                        let mut paint = Paint::default();
                        paint.set_color(color);
                        if let Some(rect) = Rect::from_xywh(x as f32, y as f32, 1.0, 1.0) {
                            pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                        }
                    }
                }
            }
        }
    }

    /// Draws the iris and pupil in the center of the eye.
    fn draw_eye_iris_and_pupil(
        &self,
        pixmap: &mut Pixmap,
        cx: f32,
        cy: f32,
        alpha: f32,
        colors: &IconColors,
        stale: bool,
    ) {
        // Iris (outer ring)
        let iris_color = if stale {
            with_alpha(colors.good, 0.5 * alpha as f64)
        } else {
            with_alpha(colors.good, 0.8 * alpha as f64)
        };

        let mut pb = PathBuilder::new();
        pb.push_circle(cx, cy, IRIS_RADIUS);
        if let Some(iris_path) = pb.finish() {
            let paint = create_paint(iris_color);
            pixmap.fill_path(
                &iris_path,
                &paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }

        // Pupil (inner black circle)
        let pupil_color = Color::from_rgba8(0, 0, 0, (200.0 * alpha) as u8);
        let mut pb = PathBuilder::new();
        pb.push_circle(cx, cy, PUPIL_RADIUS);
        if let Some(pupil_path) = pb.finish() {
            let paint = create_paint(pupil_color);
            pixmap.fill_path(
                &pupil_path,
                &paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }

        // Highlight dot (makes the eye look alive)
        let highlight_color = Color::from_rgba8(255, 255, 255, (180.0 * alpha) as u8);
        let mut pb = PathBuilder::new();
        pb.push_circle(cx - 1.0, cy - 1.0, 1.0);
        if let Some(highlight_path) = pb.finish() {
            let paint = create_paint(highlight_color);
            pixmap.fill_path(
                &highlight_path,
                &paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    /// Draws the eye outline stroke.
    fn draw_eye_outline(
        &self,
        pixmap: &mut Pixmap,
        cx: f32,
        cy: f32,
        half_w: f32,
        half_h: f32,
        colors: &IconColors,
    ) {
        let eye_path = self.create_eye_path(cx, cy, half_w, half_h);

        // Create stroke for outline
        let stroke = Stroke {
            width: 1.5,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Default::default()
        };

        let outline_color = colors.good;
        let paint = create_paint(outline_color);

        pixmap.stroke_path(&eye_path, &paint, &stroke, Transform::identity(), None);
    }

    /// Draws a horizontal line for a fully closed eye.
    fn draw_closed_eye_line(
        &self,
        pixmap: &mut Pixmap,
        cx: f32,
        cy: f32,
        half_w: f32,
        colors: &IconColors,
    ) {
        let mut pb = PathBuilder::new();
        pb.move_to(cx - half_w, cy);
        pb.line_to(cx + half_w, cy);

        if let Some(line_path) = pb.finish() {
            let stroke = Stroke {
                width: 2.0,
                line_cap: LineCap::Round,
                ..Default::default()
            };

            let paint = create_paint(colors.good);
            pixmap.stroke_path(&line_path, &paint, &stroke, Transform::identity(), None);
        }
    }
}
