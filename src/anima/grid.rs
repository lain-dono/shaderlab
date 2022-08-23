use super::Matrix;
use egui::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Bounds {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

#[derive(Clone, Copy)]
pub struct GridViewport {
    pub frame: Rect,
    pub offset: Pos2,
    pub zoom: f32,
    pub pointer: Option<Pos2>,
}

impl GridViewport {
    pub fn matrix(&self) -> Matrix {
        Matrix {
            a: self.zoom,
            b: 0.0,
            c: 0.0,
            d: -self.zoom,
            tx: self.offset.x,
            ty: self.offset.y,
        }
    }
}

pub struct Grid {
    pub offset: Vec2,
    pub zoom: f32,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

impl Grid {
    pub fn viewport(&self, pointer: Option<Pos2>, frame: Rect) -> GridViewport {
        GridViewport {
            offset: frame.center() + self.offset * vec2(-self.zoom, self.zoom),
            zoom: self.zoom,
            pointer,
            frame,
        }
    }

    pub fn update(&mut self, ui: &mut Ui, frame: Rect) {
        let input = ui.input();

        let cursor = input.pointer.hover_pos().filter(|&p| frame.contains(p));

        if let Some(cursor) = cursor {
            let mut zoom_delta = 1.0;
            for event in &input.events {
                if let Event::Scroll(scroll) = event {
                    zoom_delta *= (scroll.y / 500.0).exp();
                }
            }

            let prev = self.zoom;
            self.zoom *= zoom_delta;
            self.zoom = self.zoom.clamp(0.25, 8.0);
            let next = self.zoom;

            let size = frame.size();
            let cursor = (cursor - frame.center()) * vec2(1.0, -1.0);
            self.offset += cursor / size * (size / prev - size / next);

            if input.pointer.secondary_down() {
                let delta = input.pointer.delta();
                self.offset += delta * vec2(-1.0, 1.0) * self.zoom.recip();
            }
        }
    }

    pub fn paint(&self, ui: &Ui, frame: Rect, cursor: Option<Pos2>, shapes: &mut Vec<Shape>) {
        self.paint_axis::<0>(ui, frame, cursor, shapes);
        self.paint_axis::<1>(ui, frame, cursor, shapes);
    }

    pub fn paint_axis<const AXIS: usize>(
        &self,
        ui: &Ui,
        frame: Rect,
        cursor: Option<Pos2>,
        shapes: &mut Vec<Shape>,
    ) {
        let zoom = self.zoom;
        let ppi = ui.ctx().pixels_per_point();
        let px = ppi.recip();

        let main_axis = AXIS;
        let cross_axis = 1 - AXIS;
        let x_frame = frame.left()..=frame.right();
        let y_frame = frame.bottom()..=frame.top(); // negated y axis!

        let bounds = self.zoom_bounds(frame);

        /// Fill in all values between [min, max] which are a multiple of `step_size`
        fn fill_marks_between(min: f32, max: f32, step: f32) -> impl Iterator<Item = (f32, f32)> {
            assert!(max > min);
            let a = (min / step).ceil() as i32;
            let b = (max / step).ceil() as i32;
            (a..b).map(move |i| (step * i as f32, step))
        }

        let font_id = FontId::monospace(10.0);

        // Where on the cross-dimension to show the label values
        let value_cross = f32::clamp(0.0, bounds.min[cross_axis], bounds.max[cross_axis]);

        let x_bound = bounds.min[0]..=bounds.max[0];
        let y_bound = bounds.min[1]..=bounds.max[1];

        let steps = {
            let base_step_size = zoom.recip();

            // The distance between two of the thinnest grid lines is "rounded" up to the next-bigger power of base
            let unit = f32::powi(10.0, base_step_size.abs().log10().ceil() as i32);

            let (min, max) = (bounds.min[main_axis], bounds.max[main_axis]);

            let mut steps = vec![];
            steps.extend(fill_marks_between(min, max, unit));
            steps.extend(fill_marks_between(min, max, unit * 10.0));
            steps.extend(fill_marks_between(min, max, unit * 20.0));
            steps.extend(fill_marks_between(min, max, unit * 100.0));
            //steps.dedup_by_key(|(v, _)| *v);
            steps
        };

        for (value_main, step_size) in steps {
            let value = vec2(value_main, value_cross);
            let screen_pos = pos2(
                remap(value[main_axis], x_bound.clone(), x_frame.clone()),
                remap(value[cross_axis], y_bound.clone(), y_frame.clone()),
            );

            let screen_pos =
                crate::util::map_to_pixel_pos(screen_pos, ppi, f32::round) + vec2(px, px) / 2.0;

            let spacing_in_points = zoom * step_size;

            let line_alpha = remap_clamp(spacing_in_points, 1.0..=300.0, 0.0..=0.15);
            let text_alpha = remap_clamp(spacing_in_points, 15.0..=150.0, 0.0..=0.4);

            if line_alpha > 0.0 {
                let mut color = Rgba::from_white_alpha(line_alpha);
                if value_main == 0.0 {
                    color[main_axis] = 0.0;
                    color[cross_axis] = 1.0;
                    color[2] = 0.0;
                    color[3] = 1.0;
                }

                let mut points = [screen_pos, screen_pos];
                points[0][cross_axis] = frame.min[cross_axis];
                points[1][cross_axis] = frame.max[cross_axis];
                shapes.push(Shape::line_segment(points, Stroke::new(px, color)));
            }

            if text_alpha > 0.0 {
                let color = Rgba::from_white_alpha(text_alpha).into();
                let text = format!("{:>+.0}", value_main);

                if value_main != 0.0 {
                    let galley = ui.painter().layout_no_wrap(text, font_id.clone(), color);

                    let mut pos = screen_pos;
                    pos[main_axis] += match (main_axis, value_main > 0.0) {
                        (0, true) => -galley.size()[main_axis] - 1.0,
                        (0, false) => 1.0,

                        (_, false) => -galley.size()[main_axis],
                        (_, true) => 0.0,
                    };

                    pos[cross_axis] = frame.min[cross_axis] + px / 2.0 + 1.0;
                    shapes.push(Shape::galley(pos, galley.clone()));
                    pos[cross_axis] =
                        frame.max[cross_axis] - galley.size()[cross_axis] - px / 2.0 - 1.0;
                    shapes.push(Shape::galley(pos, galley));
                } else {
                    /*
                    let text = String::from("0");
                    let galley = ui.painter().layout_no_wrap(text, font_id.clone(), color);

                    let mut pos = screen_pos;
                    pos[main_axis] -= galley.size()[main_axis] / 2.0;

                    pos[cross_axis] = frame.min[cross_axis] + 1.0;
                    shapes.push(Shape::galley(pos, galley.clone()));
                    pos[cross_axis] = frame.max[cross_axis] - galley.size()[cross_axis] - 2.0;
                    shapes.push(Shape::galley(pos, galley));
                    */
                }
            }
        }

        if let Some(cursor) = cursor {
            let painter = ui.painter();
            let font_id = FontId::monospace(10.0);
            let color = Color32::from_gray(0xFF);
            let bg = Color32::from_rgba_unmultiplied(61, 133, 224, 128);

            let [mut min, mut max] = [frame.min, frame.max];
            min[cross_axis] = cursor[cross_axis] - px / 2.0;
            max[cross_axis] = cursor[cross_axis] - px / 2.0;
            painter.line_segment([min, max], (px, bg));

            let pos = pos2(
                remap(cursor.x, x_frame, x_bound),
                remap(cursor.y, y_frame, y_bound),
            )[cross_axis];

            let text = if pos != 0.0 {
                format!("{:>+.0}", pos)
            } else {
                String::from("0")
            };
            let galley = painter.layout_no_wrap(text, font_id, color);
            let size = galley.size();

            min[cross_axis] -= size[cross_axis] / 2.0 + px / 2.0;
            max[cross_axis] -= size[cross_axis] / 2.0 + px / 2.0;

            min[main_axis] += px / 2.0 + 1.0;
            max[main_axis] -= size[main_axis] + px / 2.0 + 1.0;

            let rect = Rect::from_min_size(min, size).expand(1.0);
            painter.rect_filled(rect, 2.0, bg);
            let rect = Rect::from_min_size(max, size).expand(1.0);
            painter.rect_filled(rect, 2.0, bg);

            painter.galley(min, galley.clone());
            painter.galley(max, galley);
        }
    }

    /// Zoom by a relative factor with the given screen position as center.
    fn zoom_bounds(&self, frame: Rect) -> Bounds {
        let center = frame.center();
        let size = frame.size();

        let bounds_min = size / -2.0 + self.offset;
        let bounds_max = size / 2.0 + self.offset;

        let x_frame = frame.left()..=frame.right();
        let y_frame = frame.bottom()..=frame.top(); // negated y axis!

        let x_bound = bounds_min.x..=bounds_max.x;
        let y_bound = bounds_min.y..=bounds_max.y;

        let center = vec2(
            remap(center.x, x_frame, x_bound),
            remap(center.y, y_frame, y_bound),
        );

        let min = center + (bounds_min - center) / self.zoom;
        let max = center + (bounds_max - center) / self.zoom;

        let delta_is_valid = (max.x - min.x) > 0.0 && (max.y - min.y) > 0.0;

        let (min, max) = if min.is_finite() && max.is_finite() && delta_is_valid {
            (min.into(), max.into())
        } else {
            (bounds_min.into(), bounds_max.into())
        };

        Bounds { min, max }
    }
}
