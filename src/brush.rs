use crate::pixels::PixelsMut;
use crate::view::{ViewCoords, ViewExtent};

use crate::gfx::math::{Point2, Vector2};
use crate::gfx::rect::Rect;
use crate::gfx::shape2d::{Circle, Fill, Rotation, Shape, Stroke};
use crate::gfx::{Rgba8, ZDepth};

use crate::util::vector_angle;
use std::collections::BTreeSet;
use std::f32::consts::PI;
use std::fmt::Display;
use std::{f32, fmt};

/// Input state of the brush.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum BrushState {
    /// Not currently drawing.
    NotDrawing,
    /// Drawing has just started.
    DrawStarted(ViewExtent),
    /// Drawing.
    Drawing(ViewExtent),
    /// Drawing has just ended.
    DrawEnded(ViewExtent),
}

/// Brush mode. Any number of these modes can be active at once.
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub enum BrushMode {
    /// Erase pixels.
    Erase,
    /// Draw on all frames at once.
    Multi,
    /// Pixel-perfect mode.
    Perfect,
    /// X-Symmetry mode.
    XSym,
    /// Y-Symmetry mode.
    YSym,
    /// X-Ray mode.
    XRay,
    /// Confine stroke to a straight line from the starting point
    Line(
        /// snap angle (degrees)
        Option<u32>,
    ),
    /// Confine stroke to a circle with radius from starting point
    Circle, // TODO: Could allow to set a fixed radius here
    Rectangle,
}

impl fmt::Display for BrushMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Erase => "erase".fmt(f),
            Self::Multi => "multi".fmt(f),
            Self::Perfect => "perfect".fmt(f),
            Self::XSym => "xsym".fmt(f),
            Self::YSym => "ysym".fmt(f),
            Self::XRay => "xray".fmt(f),
            Self::Line(Some(snap)) => write!(f, "{} degree snap line", snap),
            Self::Line(None) => write!(f, "line"),
            Self::Circle => write!(f, "circle"),
            Self::Rectangle => write!(f, "rect"),
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Align {
    Center,
    BottomLeft,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum BrushHead {
    Square,
    Circle,
}

impl Display for BrushHead {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrushHead::Square => write!(f, "square"),
            BrushHead::Circle => write!(f, "circle"),
        }
    }
}

/// Brush context.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Brush {
    /// Brush size in pixels.
    pub size: usize,
    /// Current brush state.
    pub state: BrushState,
    /// Current brush stroke.
    pub stroke: Vec<Point2<i32>>,
    /// Current stroke color.
    pub color: Rgba8,
    /// Current head to use for strokes
    pub head: BrushHead,

    /// Currently active brush modes.
    modes: BTreeSet<BrushMode>,
    /// Current brush position.
    curr: Point2<i32>,
    /// Previous brush position.
    prev: Point2<i32>,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            size: 1,
            state: BrushState::NotDrawing,
            stroke: Vec::with_capacity(32),
            color: Rgba8::TRANSPARENT,
            head: BrushHead::Square,
            modes: BTreeSet::new(),
            curr: Point2::new(0, 0),
            prev: Point2::new(0, 0),
        }
    }
}

impl Brush {
    /// Check whether the given mode is active.
    pub fn is_set(&self, m: BrushMode) -> bool {
        self.modes.contains(&m)
    }

    /// Activate the given brush mode.
    pub fn set(&mut self, m: BrushMode) -> bool {
        if let BrushMode::Line(_) = m {
            // only one line sub-mode may be active at a time
            if let Some(line_mode) = self.line_mode() {
                self.unset(line_mode);
            }
        }
        self.modes.insert(m)
    }

    /// De-activate the given brush mode.
    pub fn unset(&mut self, m: BrushMode) -> bool {
        match self.line_mode() {
            Some(line_mode) if matches!(m, BrushMode::Line(_)) => self.modes.remove(&line_mode),
            _ => self.modes.remove(&m),
        }
    }

    /// Toggle the given brush mode.
    pub fn toggle(&mut self, m: BrushMode) {
        if self.is_set(m) {
            self.unset(m);
        } else {
            self.set(m);
        }
    }

    /// Check whether the brush is currently drawing.
    pub fn is_drawing(&self) -> bool {
        !matches!(self.state, BrushState::NotDrawing)
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.modes.clear();
    }

    /// Run every frame by the session.
    pub fn update(&mut self) {
        if let BrushState::DrawEnded(_) = self.state {
            self.state = BrushState::NotDrawing;
            self.stroke.clear();
        }
    }

    /// Start drawing. Called when input is first pressed.
    pub fn start_drawing(&mut self, p: ViewCoords<i32>, color: Rgba8, extent: ViewExtent) {
        self.state = BrushState::DrawStarted(extent);
        self.color = color;
        self.stroke = Vec::with_capacity(32);
        self.draw(p);
    }

    /// Check if any primitive drawing moe is currently active
    fn has_primitive_mode(&self) -> bool {
        self.modes.iter().any(|mode| {
            matches!(
                mode,
                BrushMode::Line(_) | BrushMode::Circle | BrushMode::Rectangle
            )
        })
    }

    /// If any primitive drawing modes are active, they will be returned like f.e.
    /// - Line
    /// - Circle
    /// - etc.
    fn primitive_modes(&self) -> impl Iterator<Item = BrushMode> + use<'_> {
        let is_primitive = |mode: &&BrushMode| match mode {
            BrushMode::Line(_) | BrushMode::Circle | BrushMode::Rectangle => true,
            _ => false,
        };

        self.modes.iter().filter(is_primitive).map(|m| *m)
    }

    /// If a circle mode is active, return it
    fn circle_mode(&self) -> Option<BrushMode> {
        self.modes
            .iter()
            .filter(|mode| matches!(mode, BrushMode::Circle))
            .cloned()
            .next()
    }

    /// If a line mode is active, return it
    fn line_mode(&self) -> Option<BrushMode> {
        self.modes
            .iter()
            .filter(|mode| matches!(mode, BrushMode::Line(_)))
            .cloned()
            .next()
    }

    /// Draw. Called while input is pressed.
    pub fn draw(&mut self, p: ViewCoords<i32>) {
        let has_primitive_mode = self.has_primitive_mode();
        self.prev = if let BrushState::DrawStarted(_) = self.state {
            *p
        } else {
            if has_primitive_mode {
                // If we are drawing primitves, we NEVER update prev
                // since we want the origin of the draw operation to
                // consist across mouse moves
                self.prev
            } else {
                self.curr
            }
        };
        self.curr = *p;

        if !has_primitive_mode {
            Brush::line(self.prev, self.curr, &mut self.stroke);
            self.stroke.dedup();
        } else {
            self.stroke.clear();
            let modes = self.primitive_modes().collect::<Vec<_>>();
            for mode in modes {
                match mode {
                    BrushMode::Line(snap) => self.stroke_line(snap),
                    BrushMode::Circle => self.stroke_circle(),
                    BrushMode::Rectangle => self.stroke_rect(),
                    _ => unreachable!(),
                }
            }
        }

        if self.is_set(BrushMode::Perfect) {
            self.stroke = Brush::filter(&self.stroke);
        }

        match self.state {
            BrushState::Drawing(_) => {}
            BrushState::DrawStarted(extent) => {
                self.state = BrushState::Drawing(extent);
            }
            _ => unreachable!(),
        }
    }

    fn stroke_line(&mut self, snap: Option<u32>) {
        let start = self.prev;

        let end = match snap {
            None => self.curr,
            Some(snap) => {
                let snap_rad = snap as f32 * PI / 180.0;
                let curr: Vector2<f32> = self.curr.map(|x| x as f32).into();
                let start: Vector2<f32> = start.map(|x| x as f32).into();
                let dist = curr.distance(start);
                let angle = vector_angle(&curr, &start) - PI / 2.0;
                let round_angle = (angle / snap_rad).round() * snap_rad;
                let end = start + Vector2::new(round_angle.cos(), round_angle.sin()) * dist;
                Point2::new(end.x.round() as i32, end.y.round() as i32)
            }
        };

        Brush::line(start, end, &mut self.stroke);
    }

    fn stroke_circle(&mut self) {
        Brush::circle(self.prev, self.curr, &mut self.stroke);
    }

    fn stroke_rect(&mut self) {
        Brush::rectangle(self.prev, self.curr, &mut self.stroke);
    }

    /// Stop drawing. Called when input is released.
    pub fn stop_drawing(&mut self) {
        match self.state {
            BrushState::DrawStarted(ex) | BrushState::Drawing(ex) => {
                self.state = BrushState::DrawEnded(ex);
            }
            _ => unreachable!(),
        }
    }

    /// Expand a point into all brush heads.
    pub fn expand(&self, p: ViewCoords<i32>, extent: ViewExtent) -> Vec<ViewCoords<i32>> {
        let mut pixels = vec![*p];
        let ViewExtent { fw, fh, nframes } = extent;

        if self.is_set(BrushMode::XSym) {
            for p in pixels.clone() {
                let frame_index = p.x / fw as i32;

                pixels.push(Point2::new(
                    (frame_index + 1) * fw as i32 - (p.x - frame_index * fw as i32) - 1,
                    p.y,
                ));
            }
        }
        if self.is_set(BrushMode::YSym) {
            for p in pixels.clone() {
                pixels.push(Point2::new(p.x, fh as i32 - p.y - 1));
            }
        }
        if self.is_set(BrushMode::Multi) {
            for p in pixels.clone() {
                let frame_index = p.x / fw as i32;
                for i in 0..nframes as i32 - frame_index {
                    let offset = Vector2::new((i as u32 * fw) as i32, 0);
                    pixels.push(p + offset);
                }
            }
        }
        pixels.iter().map(|p| ViewCoords::new(p.x, p.y)).collect()
    }

    /// Return the brush's output strokes as shapes.
    pub fn output(&self, stroke: Stroke, fill: Fill, scale: f32, align: Align) -> Vec<Shape> {
        match self.state {
            BrushState::DrawStarted(extent)
            | BrushState::Drawing(extent)
            | BrushState::DrawEnded(extent) => {
                let mut pixels = Vec::new();

                for p in &self.stroke {
                    pixels.extend_from_slice(
                        self.expand(ViewCoords::new(p.x, p.y), extent).as_slice(),
                    );
                }
                pixels
                    .iter()
                    .map(|p| {
                        self.shape(
                            Point2::new(p.x as f32, p.y as f32),
                            ZDepth::ZERO,
                            stroke,
                            fill,
                            scale,
                            align,
                        )
                    })
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    /// Return the shape that should be painted when the brush is at the given
    /// position with the given parameters. Takes an `Origin` which describes
    /// whether to align the position to the bottom-left of the shape, or the
    /// center.
    pub fn shape(
        &self,
        p: Point2<f32>,
        z: ZDepth,
        stroke: Stroke,
        fill: Fill,
        scale: f32,
        align: Align,
    ) -> Shape {
        match self.head {
            BrushHead::Square => self.square_head_shape(p, z, stroke, fill, scale, align),
            BrushHead::Circle => self.circle_head_shape(p, z, stroke, fill, scale, align),
        }
    }

    fn square_head_shape(
        &self,
        p: Point2<f32>,
        z: ZDepth,
        stroke: Stroke,
        fill: Fill,
        scale: f32,
        align: Align,
    ) -> Shape {
        let x = p.x;
        let y = p.y;

        let size = self.size as f32;

        let offset = match align {
            Align::Center => size * scale / 2.,
            Align::BottomLeft => (self.size / 2) as f32 * scale,
        };

        Shape::Rectangle(
            Rect::new(x, y, x + size * scale, y + size * scale) - Vector2::new(offset, offset),
            z,
            Rotation::ZERO,
            stroke,
            fill,
        )
    }

    fn circle_head_shape(
        &self,
        p: Point2<f32>,
        z: ZDepth,
        stroke: Stroke,
        fill: Fill,
        scale: f32,
        _align: Align,
    ) -> Shape {
        // TODO: Offset here is not right, when drawing with xsym/ysym mode
        // this problem is the same though, when drawing with the square brush...
        let origin = p;

        // We keep sides a bit dynamic, as lower counts look better on lower brush sizes
        // For large brush sizes, 16 sides looks pretty good always
        let sides = (self.size * 4).min(16);
        Shape::Circle(
            Circle {
                position: origin.into(),
                radius: (self.size as f32 * scale) / 2.0,
                sides: sides as u32,
            },
            z,
            stroke,
            fill,
        )
    }

    ///////////////////////////////////////////////////////////////////////////

    /// Draw a line between two points. Uses Bresenham's line algorithm.
    pub fn line(mut p0: Point2<i32>, p1: Point2<i32>, canvas: &mut Vec<Point2<i32>>) {
        let dx = i32::abs(p1.x - p0.x);
        let dy = i32::abs(p1.y - p0.y);
        let sx = if p0.x < p1.x { 1 } else { -1 };
        let sy = if p0.y < p1.y { 1 } else { -1 };

        let mut err1 = (if dx > dy { dx } else { -dy }) / 2;
        let mut err2;

        loop {
            canvas.push(p0);

            if p0 == p1 {
                break;
            }

            err2 = err1;

            if err2 > -dx {
                err1 -= dy;
                p0.x += sx;
            }
            if err2 < dy {
                err1 += dx;
                p0.y += sy;
            }
        }
    }

    /// Draw a circle from origin and a pt on the circle. Uses Bresenham's circle algorithm.
    pub fn circle(origin: Point2<i32>, pt_on: Point2<i32>, canvas: &mut Vec<Point2<i32>>) {
        let draw_circle_quadrants = |x: i32, y: i32, canvas: &mut Vec<Point2<i32>>| {
            canvas.push(origin + Point2::new(x, y));
            canvas.push(origin + Point2::new(-x, y));
            canvas.push(origin + Point2::new(x, -y));
            canvas.push(origin + Point2::new(-x, -y));
            canvas.push(origin + Point2::new(y, x));
            canvas.push(origin + Point2::new(-y, x));
            canvas.push(origin + Point2::new(y, -x));
            canvas.push(origin + Point2::new(-y, -x));
        };

        let origin_f = Point2::new(origin.x as f32, origin.y as f32);
        let pt_on = Point2::new(pt_on.x as f32, pt_on.y as f32);
        let radius = (pt_on - origin_f).magnitude() as i32;

        // TODO: Draw a proper circle using bresenham algo
        let mut x = 0;
        let mut y = radius;
        let mut d = 3 - 2 * radius;
        draw_circle_quadrants(x, y, canvas);

        while y >= x {
            if d > 0 {
                y -= 1;
                d = d + 4 * (x - y) + 10;
            } else {
                d = d + 4 * x + 6;
            }

            x += 1;

            draw_circle_quadrants(x, y, canvas);
        }
    }

    /// Draw a rectangle from 2 corner points. If the points are equal, one of them will be drawn
    pub fn rectangle(origin: Point2<i32>, other: Point2<i32>, canvas: &mut Vec<Point2<i32>>) {
        if origin == other {
            canvas.push(origin);
            return;
        }

        let a = origin;
        let b = Point2::new(other.x, origin.y);
        let c = other;
        let d = Point2::new(origin.x, other.y);

        Self::line(a, b, canvas);
        Self::line(b, c, canvas);
        Self::line(c, d, canvas);
        Self::line(d, a, canvas);
    }

    /// Paint a circle into a pixel buffer.
    /// TODO: This could be used to implement a circular brush head maybe
    /// it is currently unused outside of tests
    #[allow(dead_code)]
    fn paint(
        pixels: &mut [Rgba8],
        w: usize,
        h: usize,
        position: Point2<f32>,
        diameter: f32,
        color: Rgba8,
    ) {
        let mut grid = PixelsMut::new(pixels, w, h);
        let bias = if diameter <= 2. {
            0.0
        } else if diameter <= 3. {
            0.5
        } else {
            0.0
        };
        let radius = diameter / 2. - bias;

        for (x, y, c) in grid.iter_mut() {
            let (x, y) = (x as f32, y as f32);

            let dx = (x - position.x).abs();
            let dy = (y - position.y).abs();
            let d = (dx.powi(2) + dy.powi(2)).sqrt();

            if d <= radius {
                *c = color;
            }
        }
    }

    /// Filter a brush stroke to remove 'L' shapes. This is often called
    /// *pixel perfect* mode.
    fn filter(stroke: &[Point2<i32>]) -> Vec<Point2<i32>> {
        let mut filtered = Vec::with_capacity(stroke.len());

        filtered.extend(stroke.first().cloned());

        let mut triples = stroke.windows(3);
        while let Some(triple) = triples.next() {
            let (prev, curr, next) = (triple[0], triple[1], triple[2]);
            if (prev.y == curr.y && next.x == curr.x) || (prev.x == curr.x && next.y == curr.y) {
                filtered.push(next);
                triples.next();
            } else {
                filtered.push(curr);
            }
        }

        filtered.extend(stroke.last().cloned());

        filtered
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_paint() {
        let z = Rgba8::TRANSPARENT;
        let w = Rgba8::WHITE;

        #[rustfmt::skip]
        let brush1 = vec![
            z, z, z,
            z, w, z,
            z, z, z,
        ];

        #[rustfmt::skip]
        let brush2 = vec![
            z, z, z, z,
            z, w, w, z,
            z, w, w, z,
            z, z, z, z,
        ];

        #[rustfmt::skip]
        let brush3 = vec![
            z, z, z, z, z,
            z, z, w, z, z,
            z, w, w, w, z,
            z, z, w, z, z,
            z, z, z, z, z,
        ];

        #[rustfmt::skip]
        let brush5 = vec![
            z, z, z, z, z, z, z,
            z, z, w, w, w, z, z,
            z, w, w, w, w, w, z,
            z, w, w, w, w, w, z,
            z, w, w, w, w, w, z,
            z, z, w, w, w, z, z,
            z, z, z, z, z, z, z,
        ];

        #[rustfmt::skip]
        let brush7 = vec![
            z, z, z, z, z, z, z, z, z,
            z, z, z, w, w, w, z, z, z,
            z, z, w, w, w, w, w, z, z,
            z, w, w, w, w, w, w, w, z,
            z, w, w, w, w, w, w, w, z,
            z, w, w, w, w, w, w, w, z,
            z, z, w, w, w, w, w, z, z,
            z, z, z, w, w, w, z, z, z,
            z, z, z, z, z, z, z, z, z
        ];

        #[rustfmt::skip]
        let brush15 = vec![
            z, z, z, z, z, w, w, w, w, w, z, z, z, z, z,
            z, z, z, w, w, w, w, w, w, w, w, w, z, z, z,
            z, z, w, w, w, w, w, w, w, w, w, w, w, z, z,
            z, w, w, w, w, w, w, w, w, w, w, w, w, w, z,
            z, w, w, w, w, w, w, w, w, w, w, w, w, w, z,
            w, w, w, w, w, w, w, w, w, w, w, w, w, w, w,
            w, w, w, w, w, w, w, w, w, w, w, w, w, w, w,
            w, w, w, w, w, w, w, w, w, w, w, w, w, w, w,
            w, w, w, w, w, w, w, w, w, w, w, w, w, w, w,
            w, w, w, w, w, w, w, w, w, w, w, w, w, w, w,
            z, w, w, w, w, w, w, w, w, w, w, w, w, w, z,
            z, w, w, w, w, w, w, w, w, w, w, w, w, w, z,
            z, z, w, w, w, w, w, w, w, w, w, w, w, z, z,
            z, z, z, w, w, w, w, w, w, w, w, w, z, z, z,
            z, z, z, z, z, w, w, w, w, w, z, z, z, z, z,
        ];

        {
            let mut canvas = vec![Rgba8::TRANSPARENT; 3 * 3];
            Brush::paint(&mut canvas, 3, 3, Point2::new(1., 1.), 1., Rgba8::WHITE);
            assert_eq!(canvas, brush1);
        }

        {
            let mut canvas = vec![Rgba8::TRANSPARENT; 4 * 4];
            Brush::paint(&mut canvas, 4, 4, Point2::new(1.5, 1.5), 2., Rgba8::WHITE);
            assert_eq!(canvas, brush2);
        }

        {
            let mut canvas = vec![Rgba8::TRANSPARENT; 5 * 5];
            Brush::paint(&mut canvas, 5, 5, Point2::new(2., 2.), 3., Rgba8::WHITE);
            assert_eq!(canvas, brush3);
        }

        {
            let mut canvas = vec![Rgba8::TRANSPARENT; 7 * 7];
            Brush::paint(&mut canvas, 7, 7, Point2::new(3., 3.), 5., Rgba8::WHITE);
            assert_eq!(canvas, brush5);
        }

        {
            let mut canvas = vec![Rgba8::TRANSPARENT; 9 * 9];
            Brush::paint(&mut canvas, 9, 9, Point2::new(4., 4.), 7., Rgba8::WHITE);
            assert_eq!(canvas, brush7);
        }

        {
            let mut canvas = vec![Rgba8::TRANSPARENT; 15 * 15];
            Brush::paint(&mut canvas, 15, 15, Point2::new(7., 7.), 15., Rgba8::WHITE);
            assert_eq!(canvas, brush15);
        }
    }
}
