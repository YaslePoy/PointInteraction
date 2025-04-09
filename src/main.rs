use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::render::{FPoint, WindowCanvas};
use std::f32::consts::PI;
use std::ops;
use std::ops::Add;
use std::time::Duration;

const SIZE: u32 = 400;
pub fn main() {
    let mut point = Point2D::new(0.0, 0.0);

    let mut pasted_points: Vec<FPoint> = vec![point.to_cartesian().to_sdl()];
    // let cartestian = point.to_cartesian();
    // let re_polar = Point2D::to_polar(cartestian);
    // let plane = point.to_cartesian();
    //
    // let mut test_point = Point2D::new(700.0, 100.0);
    // test_point.optimize();
    // println!("{:?}", test_point);

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
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
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

        point += (0.0, 1.0);

        point.optimize();
        pasted_points.push(point.to_cartesian().to_sdl());
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        Point2D::draw_point(&mut canvas, &pasted_points);

        canvas.present();
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 100));
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
        2.0 * SIZE as f32 - self.distance(point)
    }

    pub fn to_cartesian(&self) -> Point2D {
        let angle = (self.x / SIZE as f32) * PI;
        let cos = angle.cos();
        let sin = angle.sin();
        let scale = (SIZE as f32 + self.y) / 2.0;
        Point2D::new(cos * scale, sin * scale)
    }

    pub fn to_polar(p: Point2D) -> Point2D {
        let len = p.y.hypot(p.x);
        let cos = p.x / len;
        let sin = p.y / len;
        let x = cos.acos();
        let mut y = sin.asin();
        let mb_sin = x.sin();
        if y.signum() != mb_sin.signum() {
            y = -y;
        }
        Point2D::new(x, y)
    }

    pub fn to_sdl(&self) -> FPoint {
        FPoint::new(self.x + SIZE as f32, self.y + SIZE as f32)
    }

    pub fn from_sdl(x: f32, y: f32) -> Point2D {
        Point2D::new(x - SIZE as f32, y - SIZE as f32)
    }

    pub fn draw(&self, canvas: &mut WindowCanvas) {
        let mut polar = self.to_cartesian();
        let sdl_point = polar.to_sdl();
        canvas.draw_point(sdl_point).unwrap();
    }

    fn lapped(l: f32) -> f32 {
        let lep = ((SIZE as f32) - (l.abs() - SIZE as f32).abs()) * -l.signum();
        println!("Lap {} to {}", l, lep);
        lep
    }

    pub fn optimize(&mut self) {
        if self.x.abs() > SIZE as f32 {
            self.x = Self::lapped(self.x);
        }

        if self.y.abs() > SIZE as f32 {
            let side = self.y.signum();
            self.y += -side * 2.0 * (self.y.abs() - SIZE as f32);
            self.x = self.x + SIZE as f32;
            self.spin = !self.spin;
        }
    }

    pub fn draw_point(canvas: &mut WindowCanvas, points: &Vec<FPoint>) {
        // canvas.draw_points(&points).unwrap()
        canvas.draw_points(&points[..]).unwrap();
    }
    fn requires_slice<'a, T: Into<&'a [FPoint]>>(arg: T) {
        let slice: &[FPoint] = arg.into();
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
