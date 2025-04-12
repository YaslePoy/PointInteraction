use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::rect::Point;
use sdl3::render::{FPoint, WindowCanvas};
use std::f32::consts::PI;
use std::ops;
use std::ops::{AddAssign, Mul, Rem, Sub};
use std::time::{Duration, Instant};

const SIZE: u32 = 400;
const SIZE_F: f32 = 400.0;

pub fn main() {
    let mut point_a = Point2D::new(-5.0, 0.0);
    let mut point_b = Point2D::new(5.0, 0.0);

    let line = Line2D::new(&mut point_a, &mut point_b);

    let mut selection = 1;

    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Point interaction", SIZE * 2, SIZE * 2)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.clone().into_canvas();

    sdl_context.mouse().show_cursor(false);
    sdl_context.mouse().capture(true);
    sdl_context
        .mouse()
        .warp_mouse_in_window(&window, SIZE_F, SIZE_F);

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let time = Instant::now();
    let mut count = 0;
    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::_1),
                    ..
                } => {
                    selection = 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::_2),
                    ..
                } => {
                    selection = 2;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    println!("Debug")
                }
                Event::MouseMotion { .. } => {
                    if let Event::MouseMotion {
                        timestamp: _,
                        window_id: _,
                        which: _,
                        mousestate: _,
                        x,
                        y,
                        xrel,
                        yrel,
                    } = event
                    {
                        let mut cursor: &mut Point2D;
                        if selection == 1 {
                            cursor = line.a;
                        } else {
                            cursor = line.b;
                        }

                        cursor += (xrel, yrel);
                        cursor.optimize();
                        cursor = &mut cursor.abs();
                    }
                }
                _ => {}
            }
        }

        sdl_context
            .mouse()
            .warp_mouse_in_window(&window, SIZE_F, SIZE_F);
        // The rest of the game loop goes here...

        count += 1;

        line.draw(&mut canvas, 1.0);

        canvas.present();

        if count == 10000 {
            println!("Time for 10000 is {} ms", time.elapsed().as_millis());
        }

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 600));
    }
}

#[derive(Debug)]
struct Point2D {
    x: f32,
    y: f32,
    spin: bool,
}

impl Point2D {
    pub fn new(x: f32, y: f32) -> Point2D {
        Point2D { x, y, spin: false }
    }

    pub fn new_with_spin(x: f32, y: f32, spin: bool) -> Point2D {
        Point2D { x, y, spin }
    }
    pub fn distance_abs(&self, point: &Point2D) -> f32 {
        let a_abs = self.optimized().abs();
        let b_abs = point.optimized().abs();
        (a_abs.x - b_abs.x).hypot(a_abs.y - b_abs.y)
    }

    pub fn distance_sq(&self, point: &Point2D) -> f32 {
        (self.x - point.x).powi(2) + (self.y - point.y).powi(2)
    }

    pub fn alt_distance(&self, point: &Point2D) -> f32 {
        2.0 * SIZE_F - self.distance_abs(point)
    }

    pub fn to_cartesian(&self) -> Point2D {
        let angle = (self.x / SIZE_F) * PI;
        let cos = angle.cos();
        let sin = angle.sin();
        let scale = (SIZE_F + self.y) / 2.0;
        Point2D::new(cos * scale, sin * scale)
    }

    pub fn to_super_space(&self) -> Point2D {
        let len = self.x.hypot(self.y);
        let y = len * 2.0 - SIZE_F;

        let cos = self.x / len;

        let acos = cos.acos();

        let mut x = acos * SIZE_F / PI;

        if self.y < 0.0 {
            x = -x;
        }

        Point2D::new(x, y)
    }

    pub fn to_sdl(&self) -> FPoint {
        FPoint::new(self.x + SIZE_F, self.y + SIZE_F)
    }

    pub fn from_sdl(x: f32, y: f32) -> Point2D {
        Point2D::new(x - SIZE_F, y - SIZE_F)
    }

    pub fn draw(&self, canvas: &mut WindowCanvas) {
        let polar = self.to_cartesian();
        let sdl_point = polar.to_sdl();
        canvas.draw_point(sdl_point).unwrap();
    }

    fn lapped(l: f32) -> f32 {
        let mut remainder = l.rem(SIZE_F * 2.0);

        if remainder.abs() > SIZE_F {
            let out_delta = remainder.abs() - SIZE_F;
            let re_new = SIZE_F - out_delta;
            remainder = -re_new * l.signum()
        }

        remainder
    }

    pub fn normalized(&self) -> Point2D {
        let len = self.x.hypot(self.y);
        Point2D::new_with_spin(self.x / len, self.y / len, self.spin)
    }

    pub fn optimize(&mut self) {
        if self.x.abs() > SIZE_F {
            self.x = Self::lapped(self.x);
        }

        if self.y.abs() > SIZE_F {
            let side = self.y.signum();
            self.y += -side * 2.0 * (self.y.abs() - SIZE_F);
            self.x = self.x + SIZE_F;
            self.spin = !self.spin;
        }
    }

    pub fn optimized(&self) -> Point2D {
        let mut x: f32 = self.x;
        let mut y: f32 = self.y;
        let mut spin: bool = self.spin;
        if self.x.abs() > SIZE_F {
            x = Self::lapped(self.x);
        }

        if self.y.abs() > SIZE_F {
            let side = self.y.signum();
            y += -side * 2.0 * (self.y.abs() - SIZE_F);
            x = self.x + SIZE_F;
            spin = !spin;
        }

        Point2D::new_with_spin(x, y, spin)
    }

    pub fn draw_point(canvas: &mut WindowCanvas, points: &Vec<FPoint>) {
        // canvas.draw_points(&points).unwrap()
        canvas.draw_points(&points[..]).unwrap();
    }

    pub fn eq(&self, point: &Point2D) -> bool {
        (self.x.abs() - point.x.abs()).abs() < 0.1 && (self.y.abs() - point.y.abs()).abs() < 0.1
    }

    pub fn eq_spin(&self, point: &Point2D) -> bool {
        self.eq(point) && self.spin == point.spin
    }

    pub fn abs(&self) -> Point2D {
        let x: f32;
        let y: f32;
        if self.x < 0.0 {
            x = self.x.rem_euclid(SIZE_F * 2.0);
        } else {
            x = self.x;
        }
        if self.y < 0.0 {
            y = self.y.rem_euclid(SIZE_F * 2.0);
        } else {
            y = self.y;
        }

        Point2D::new(x, y)
    }

    pub fn relative(&self) -> Point2D {
        Point2D::new(Self::lapped(self.x), Self::lapped(self.y))
    }

    pub fn add(&mut self, x: f32, y: f32) {
        self.x += x;
        self.y += y;
    }
}

impl ops::AddAssign for Point2D {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x * bool_to_int(self.spin) as f32;
        self.y += rhs.y * bool_to_int(self.spin) as f32;
    }
}

impl AddAssign<(f32, f32)> for Point2D {
    fn add_assign(&mut self, rhs: (f32, f32)) {
        self.x += rhs.0 * bool_to_int(self.spin) as f32;
        self.y += rhs.1 * bool_to_int(self.spin) as f32;
    }
}

impl AddAssign<(f32, f32)> for &mut Point2D {
    fn add_assign(&mut self, rhs: (f32, f32)) {
        self.x += rhs.0 * bool_to_int(self.spin) as f32;
        self.y += rhs.1 * bool_to_int(self.spin) as f32;
    }
}

fn bool_to_int(flag: bool) -> i32 {
    if !flag { 1 } else { -1 }
}

impl Clone for Point2D {
    fn clone(&self) -> Self {
        Point2D::new_with_spin(self.x, self.y, self.spin)
    }
}

struct Line2D<'a> {
    a: &'a mut Point2D,
    b: &'a mut Point2D,
    ending: Color,
    fill: Color,
    segments: u32,
}

impl Sub for Point2D {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Point2D::new_with_spin(self.x - rhs.x, self.y - rhs.y, self.spin)
    }
}

impl Mul<f32> for Point2D {
    type Output = Point2D;

    fn mul(self, rhs: f32) -> Self::Output {
        Point2D::new_with_spin(self.x * rhs, self.y * rhs, self.spin)
    }
}

impl<'a> Line2D<'a> {
    pub fn new(a: &'a mut Point2D, b: &'a mut Point2D) -> Self {
        Self {
            a,
            b,
            ending: Color::WHITE,
            fill: Color::YELLOW,
            segments: 13,
        }
    }

    pub fn draw<'b>(&'a self, canvas: &mut WindowCanvas, pow: f32) {
        let distance: f32;
        if pow < 0.0 {
            distance = self.a.alt_distance(self.b);
        } else {
            distance = self.b.distance_abs(self.a);
        }

        let a_abs = self.a.abs();
        let b_abs = self.b.abs();
        let vec = (b_abs - a_abs).normalized() * (distance / self.segments as f32) * pow;
        let mut cursor = self.a.clone();
        canvas.set_draw_color(self.fill);
        for i in 1..self.segments {
            cursor.add(vec.x, vec.y);
            cursor.optimize();
            cursor.draw(canvas);
        }
        canvas.set_draw_color(self.ending);
        self.a.draw(canvas);
        self.b.draw(canvas);
    }

    pub fn draw_alt<'b>(&'a self, canvas: &mut WindowCanvas) {
        let mut local_a = self.a.clone().abs();
        let mut local_b = self.b.clone().abs();

        let x_step = (local_a.x - local_b.x) / self.segments as f32;
        let y_step = (local_a.y - local_b.y) / self.segments as f32;
        canvas.set_draw_color(self.fill);
        for i in 1..self.segments {
            let pt = Point2D::new(local_b.x + x_step * i as f32, local_b.y + y_step * i as f32);
            pt.draw(canvas);
        }

        canvas.set_draw_color(self.ending);
        local_b.draw(canvas);
        local_a.draw(canvas);
    }
}
