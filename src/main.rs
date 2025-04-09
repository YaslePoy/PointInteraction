use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::render::{FPoint, WindowCanvas};
use std::f32::consts::PI;
use std::ops;
use std::ops::Rem;
use std::time::Instant;

const SIZE: u32 = 400;
const SIZE_F: f32 = 400.0;
pub fn main() {
    let point = Point2D::new(0.0, 0.0);

    let mut cycle = 0_f32;
    let radius = 100_f32;
    let mut pasted_points: Vec<FPoint> = vec![point.to_cartesian().to_sdl()];

    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Point interaction", SIZE * 2, SIZE * 2)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();

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
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        let cos = cycle.cos() * radius;
        let sin = cycle.sin() * radius;
        let mut next = Point2D::new(cos + cycle * 2.0 * PI, sin);
        next.optimize();


        pasted_points.push(next.to_cartesian().to_sdl());
        count += 1;
        canvas.set_draw_color(Color::RGB(255, 255, 255));

        Point2D::draw_point(&mut canvas, &pasted_points);

        canvas.present();

        cycle -= 0.01;

        if count == 10000 {
            println!("Time for 10000 is {} ms", time.elapsed().as_millis());
        }

        // std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 200));
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

    pub fn distance(&self, point: &Point2D) -> f32 {
        (self.x - point.x).hypot(self.y - point.y)
    }

    pub fn distance_sq(&self, point: &Point2D) -> f32 {
        (self.x - point.x).powi(2) + (self.y - point.y).powi(2)
    }

    pub fn alt_distance(&self, point: &Point2D) -> f32 {
        2.0 * SIZE_F - self.distance(point)
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
}

impl ops::AddAssign for Point2D {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x * bool_to_int(self.spin) as f32;
        self.y += rhs.y * bool_to_int(self.spin) as f32;
    }
}

impl ops::AddAssign<(f32, f32)> for Point2D {
    fn add_assign(&mut self, rhs: (f32, f32)) {
        self.x += rhs.0 * bool_to_int(self.spin) as f32;
        self.y += rhs.1 * bool_to_int(self.spin) as f32;
    }
}
fn bool_to_int(flag: bool) -> i32 {
    if !flag { 1 } else { -1 }
}
