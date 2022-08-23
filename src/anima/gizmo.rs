use std::f32::consts::PI;

use reui::{plugin::Recorder, Color, FillRule, Offset, Path, Solidity, Stroke, Transform};

pub enum Tool {}

pub struct Gizmos<'a> {
    pub recorder: &'a mut Recorder,
    pub viewport: Transform,
    pub transform: Transform,
    pub path: Path,
}

impl<'a> Gizmos<'a> {
    pub fn new(recorder: &'a mut Recorder, viewport: Transform) -> Self {
        Self {
            recorder,
            viewport,
            transform: viewport,
            path: Path::new(),
        }
    }

    pub fn set_transform(&mut self, tx: f32, ty: f32, rotation: f32, scale: f32) {
        self.transform = self.viewport * Transform::compose(tx, ty, rotation, scale);
    }

    pub fn rotate(&mut self, color: Color) {
        use std::f32::consts::TAU;

        let a = f32::to_radians(10.0);
        let b = f32::to_radians(14.0);

        let inner_radius = 19.0;
        let outer_radius = 23.0;

        let center = Offset::zero();

        self.path.clear();
        self.path
            .arc(center, outer_radius, b, TAU - b, Solidity::Hole);
        self.path
            .arc(center, inner_radius, TAU - a, a, Solidity::Solid);

        self.path.move_to(Offset::new(outer_radius - 5.0, 0.0));
        self.path.line_to(Offset::new(outer_radius, 3.0));
        self.path.line_to(Offset::new(outer_radius + 5.0, 0.0));
        self.path.line_to(Offset::new(outer_radius, -3.0));
        self.path.close();

        self.shadow();
        self.fill(color);
    }

    pub fn arrow_translate(&mut self, color: Color) {
        let main = (10.0, 27.0, 28.0, 40.0);
        let cross = (5.0, 8.0);

        let control_a = Offset::new(20.0, -1.0);
        let control_b = Offset::new(20.0, 1.0);

        self.path.clear();
        self.path.move_to(Offset::new(main.3, 0.0));
        self.path.line_to(Offset::new(main.2, -cross.1));
        self.path.line_to(Offset::new(main.1, -cross.0));
        self.path.quad_to(control_a, Offset::new(main.0, -1.0));
        self.path.line_to(Offset::new(main.0, 1.0));
        self.path.quad_to(control_b, Offset::new(main.1, cross.0));
        self.path.line_to(Offset::new(main.2, cross.1));
        self.path.close();

        self.shadow();
        self.fill(color);
    }

    pub fn arrow_scale(&mut self, color: Color) {
        let main = (10.0, 27.0, 28.0, 40.0);
        let cross = (5.0, 8.0);

        let control_a = Offset::new(20.0, -1.0);
        let control_b = Offset::new(20.0, 1.0);

        self.path.clear();
        self.path.move_to(Offset::new(main.3, cross.1));
        self.path.line_to(Offset::new(main.3, -cross.1));
        self.path.line_to(Offset::new(main.2, -cross.1));
        self.path.line_to(Offset::new(main.1, -cross.0));
        self.path.quad_to(control_a, Offset::new(main.0, -1.0));
        self.path.line_to(Offset::new(main.0, 1.0));
        self.path.quad_to(control_b, Offset::new(main.1, cross.0));
        self.path.line_to(Offset::new(main.2, cross.1));
        self.path.close();

        self.shadow();
        self.fill(color);
    }

    pub fn length(&mut self, color: Color) {
        let inner_radius = 4.0;
        let outer_radius = 5.0;
        let extra_radius = 6.0;

        self.path.clear();
        self.path.solidity(Solidity::Solid);
        self.path.circle(Offset::zero(), outer_radius);
        self.path.solidity(Solidity::Hole);
        self.path.circle(Offset::zero(), inner_radius);

        self.path.move_to(Offset::new(11.0, 0.0));
        self.path.line_to(Offset::new(extra_radius, 3.0));
        self.path.quad_to(
            Offset::new(extra_radius + 1.0, 0.0),
            Offset::new(extra_radius, -3.0),
        );
        self.path.close();

        self.path.move_to(Offset::new(-11.0, 0.0));
        self.path.line_to(Offset::new(-extra_radius, 3.0));
        self.path.quad_to(
            Offset::new(-extra_radius - 1.0, 0.0),
            Offset::new(-extra_radius, -3.0),
        );
        self.path.close();

        self.shadow();
        self.fill(color);
    }

    pub fn pose(&mut self, color: Color) {
        let inner_radius = 4.0;
        let outer_radius = 5.0;
        let extra_radius = 5.0;
        let end = 9.0;
        let cross = 2.0;

        self.path.clear();
        self.path.solidity(Solidity::Solid);

        self.path.move_to(Offset::new(end, 0.0));
        self.path.line_to(Offset::new(extra_radius, cross));
        self.path.line_to(Offset::new(extra_radius, -cross));

        self.path.move_to(-Offset::new(end, 0.0));
        self.path.line_to(-Offset::new(extra_radius, cross));
        self.path.line_to(-Offset::new(extra_radius, -cross));

        self.path.move_to(Offset::new(0.0, end));
        self.path.line_to(Offset::new(-cross, extra_radius));
        self.path.line_to(Offset::new(cross, extra_radius));

        self.path.move_to(-Offset::new(0.0, end));
        self.path.line_to(-Offset::new(-cross, extra_radius));
        self.path.line_to(-Offset::new(cross, extra_radius));

        self.path.circle(Offset::zero(), outer_radius);

        self.path.solidity(Solidity::Hole);
        self.path.circle(Offset::zero(), inner_radius);

        self.shadow();
        self.fill(color);
    }

    pub fn shear(&mut self, color: Color, value: f32) {
        let min = 15.0;
        let max = 60.0;

        let w = 1.0;
        let s = 3.0;

        {
            self.path.clear();

            self.path.move_to(Offset::new(0.0, 0.0));
            let a = if value > 0.0 {
                (0.0, -value % PI)
            } else {
                (-value % PI, 0.0)
            };
            self.path
                .arc(Offset::zero(), 38.0, a.0, a.1, Solidity::Solid);
            self.path.close();

            let mut color = color;
            color.alpha *= 0.25;

            self.fill(color);

            self.path.clear();
            self.path.move_to(Offset::new(min, 0.0));
            self.path.line_to(Offset::new(min + 1.0, 1.0));
            self.path.line_to(Offset::new(48.0, 1.0));
            self.path.line_to(Offset::new(48.0, -1.0));
            self.path.line_to(Offset::new(min + 1.0, -1.0));

            self.fill(color);
        }

        let tx = Transform::rotation(-value);

        self.path.clear();

        self.path.move_to(tx.apply(Offset::new(min, 0.0)));

        self.path.line_to(tx.apply(Offset::new(min + w, w)));
        self.path.line_to(tx.apply(Offset::new(max, w)));
        self.path.line_to(tx.apply(Offset::new(max + s, s)));
        self.path.line_to(tx.apply(Offset::new(max + s * 2.0, 0.0)));
        self.path.line_to(tx.apply(Offset::new(max + s, -s)));
        self.path.line_to(tx.apply(Offset::new(max, -w)));
        self.path.line_to(tx.apply(Offset::new(min + w, -w)));

        self.shadow();
        self.fill(color);
    }

    fn fill(&mut self, color: Color) {
        self.recorder
            .fill(&self.path, color, self.transform, FillRule::EvenOdd, true);
    }

    fn shadow(&mut self) {
        self.recorder.stroke(
            &self.path,
            Color::bgra(0x99000000),
            Stroke::width(2.0),
            self.transform,
            true,
        );
    }
}
