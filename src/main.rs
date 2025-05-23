use ggez::conf::WindowMode;
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawParam, Image, ImageFormat};
use ggez::input::keyboard::KeyCode::X;
use ggez::{Context, ContextBuilder, GameResult};
use std::f32::consts::PI;
use std::ops::{Mul, Neg, Rem, Sub};
use std::time::Instant;

const SIZE: u32 = 500;
const SIZE_F: f32 = 500.0;
const ARC: f32 = SIZE_F * 2.0 * PI;

fn main() {
    // Make a Context.
    let (mut ctx, event_loop) = ContextBuilder::new("Point interaction", "YaslePoy")
        .window_mode(
            WindowMode::default()
                .resizable(false)
                .dimensions(SIZE_F * 2.0, SIZE_F * 2.0),
        )
        .build()
        .expect("aieee, could not create ggez context!");

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let my_game = MyGame::new(&mut ctx);

    // Run!
    event::run(ctx, event_loop, my_game);
}

struct MyGame {
    display: Image,
    raw_view: Vec<u8>,
    point_a: Vector2,
    point_b: Vector2,
    require_draw: bool,
}

impl MyGame {
    pub fn new(_ctx: &mut Context) -> MyGame {
        // Load/create resources such as images here.
        let mut raw = vec![0; (SIZE * SIZE * 16) as usize];
        let point_a = Vector2::new(0.0, 0.0);
        let point_b = Vector2::new(200.0, 200.0);

        let a = point_a.to_cartesian().to_sdl();
        let b = point_b.to_cartesian().to_sdl();

        draw_to_raw(a.0, a.1, Color::WHITE, &mut raw);
        draw_to_raw(b.0, b.1, Color::WHITE, &mut raw);

        MyGame {
            display: Image::from_pixels(
                _ctx,
                &raw,
                ImageFormat::Rgba8UnormSrgb,
                SIZE * 2,
                SIZE * 2,
            ),
            raw_view: raw,
            point_a,
            point_b,
            require_draw: false
        }
    }
}

pub fn draw_to_raw(x: u32, y: u32, color: Color, field: &mut Vec<u8>) {
    let (r, g, b, a) = color.to_rgba();
    let offset = (SIZE * 8 * y + x * 4) as usize;
    field[offset] = r;
    field[offset + 1] = g;
    field[offset + 2] = b;
    field[offset + 3] = a;
}

impl EventHandler for MyGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // Update code here...
        if _ctx.keyboard.is_key_just_pressed(X) {

            let mut ready_points: Vec<((u32, u32), u8)> = vec![];

            let a_clone = self.point_a.clone();
            let b_clone = self.point_b.clone();
            let timer = Instant::now();
            ready_points.clear();

            for x in (-(SIZE as i32))..(SIZE as i32) {
                for y in (-(SIZE as i32))..(SIZE as i32) {
                    let pt = Vector2::new(x as f32, y as f32);

                    if x == 0 && y == 0 {
                        println!("0-0");
                    }

                    if pt.eq(&a_clone) || pt.eq(&b_clone) {
                        continue;
                    }
                    let dist_a = pt.distance(&(a_clone));
                    let dist_b = pt.distance(&(b_clone));
                    let delta = (dist_a - dist_b).abs();

                    if delta > 300.0 {
                        continue;
                    }

                    let ratio = (256.0 - (delta / 100.0).powf(0.3) * 256.0) as u8;

                    ready_points.push((pt.to_cartesian().to_sdl(), ratio));
                }
            }
            println!("end");
            println!("{}", timer.elapsed().as_millis());

            for ready_point in ready_points {
                draw_to_raw(
                    ready_point.0.0,
                    ready_point.0.1,
                    Color::from_rgba(u8::MAX, u8::MAX, u8::MAX, ready_point.1),
                    &mut self.raw_view,
                );
                self.require_draw = true
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
        // Draw code here...
        if self.require_draw { 
            self.display = Image::from_pixels(
                ctx,
                &*self.raw_view,
                ImageFormat::Rgba8UnormSrgb,
                SIZE * 2,
                SIZE * 2,
            );
            self.require_draw = false;
        }
        
        canvas.draw(&self.display, DrawParam::default());
        canvas.finish(ctx)
    }
}

#[derive(Debug)]
struct Vector2 {
    x: f32,
    y: f32,
    orientation: Vector3,
}
impl Vector2 {
    pub fn new(x: f32, y: f32) -> Vector2 {
        Vector2 {
            x,
            y,
            orientation: Vector3::vector_z(),
        }
    }

    pub fn new_with_spin(x: f32, y: f32, spin: Vector3) -> Vector2 {
        Vector2 {
            x,
            y,
            orientation: spin,
        }
    }

    pub fn to_3d_raw(&self) -> Vector3 {
        Vector3::new(self.x, self.y, 0.0)
    }

    pub fn to_3d_placed(&self) -> Vector3 {
        let v_angle = (self.y / SIZE_F) * PI / 2.0;
        let z = v_angle.sin() * SIZE_F;
        let xy_scale = v_angle.cos();
        let angle = (self.x / SIZE_F) * PI / 2.0;
        let x = angle.cos() * SIZE_F * xy_scale;
        let y = angle.sin() * SIZE_F * xy_scale;

        Vector3::new(x, y, z)
    }

    pub fn distance(&self, point: &Vector2) -> f32 {
        let self_3d = self.to_3d_placed();
        let other_3d = point.to_3d_placed();
        let dot = Vector3::dot(self_3d.normalized(), other_3d.normalized());
        let angle = dot.acos();
        angle / PI * ARC
    }

    pub fn alt_distance(&self, point: &Vector2) -> f32 {
        ARC - self.distance(point)
    }

    pub fn rotate_to(&mut self, point: Vector2) {
        let self_3d = self.to_3d_placed();
        let other_3d = point.to_3d_placed();
        let mut normal = other_3d * self_3d;
        normal.normalize();
        self.orientation = -normal;
    }

    pub fn move_to(&mut self, distance: f32) {
        let angle = distance / ARC * 2.0 * PI;
        let rotation = Quaternion::from_axis_angle(self.orientation.clone(), angle);
        let self_3d = self.to_3d_placed();
        let rotated = self_3d.rotated(rotation);
        let projected = rotated.to_2d_placed();

        self.x = projected.x;
        self.y = projected.y;
    }

    pub fn to_cartesian(&self) -> Vector2 {
        let angle = (self.x / SIZE_F) * PI;
        let cos = angle.cos();
        let sin = angle.sin();
        let scale = (SIZE_F + self.y) / 2.0;
        Vector2::new(cos * scale, sin * scale)
    }

    pub fn to_super_space(&self) -> Vector2 {
        let len = self.x.hypot(self.y);
        let y = len * 2.0 - SIZE_F;

        let cos = self.x / len;

        let acos = cos.acos();

        let mut x = acos * SIZE_F / PI;

        if self.y < 0.0 {
            x = -x;
        }

        Vector2::new(x, y)
    }

    pub fn to_sdl(&self) -> (u32, u32) {
        ((self.x + SIZE_F) as u32, (self.y + SIZE_F) as u32)
    }

    pub fn from_sdl(x: f32, y: f32) -> Vector2 {
        Vector2::new(x - SIZE_F, y - SIZE_F)
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

    pub fn normalized(&self) -> Vector2 {
        let len = self.x.hypot(self.y);
        Vector2::new_with_spin(self.x / len, self.y / len, self.orientation.clone())
    }

    pub fn optimize(&mut self) {
        if self.x.abs() > SIZE_F {
            self.x = Self::lapped(self.x);
        }

        if self.y.abs() > SIZE_F {
            let side = self.y.signum();
            self.y += -side * 2.0 * (self.y.abs() - SIZE_F);
            self.x = self.x + SIZE_F;
        }
    }

    pub fn optimized(&self) -> Vector2 {
        let mut x: f32 = self.x;
        let mut y: f32 = self.y;
        if self.x.abs() > SIZE_F {
            x = Self::lapped(self.x);
        }

        if self.y.abs() > SIZE_F {
            let side = self.y.signum();
            y += -side * 2.0 * (self.y.abs() - SIZE_F);
            x = self.x + SIZE_F;
        }

        Vector2::new_with_spin(x, y, self.orientation.clone())
    }

    pub fn eq(&self, point: &Vector2) -> bool {
        (self.x.abs() - point.x.abs()).abs() < 0.1 && (self.y.abs() - point.y.abs()).abs() < 0.1
    }

    pub fn abs(&self) -> Vector2 {
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

        Vector2::new(x, y)
    }

    pub fn relative(&self) -> Vector2 {
        Vector2::new(Self::lapped(self.x), Self::lapped(self.y))
    }

    pub fn add(&mut self, x: f32, y: f32) {
        self.x += x;
        self.y += y;
    }
}

#[derive(Debug)]
struct Vector3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3 { x, y, z }
    }

    pub fn normalize(&mut self) {
        let len = self.x.hypot(self.y).hypot(self.z);
        self.x /= len;
        self.y /= len;
        self.z /= len;
    }

    pub fn normalized(&self) -> Vector3 {
        let len = self.x.hypot(self.y).hypot(self.z);
        Vector3::new(self.x / len, self.y / len, self.z / len)
    }

    pub fn vector_x() -> Vector3 {
        Vector3::new(1.0, 0.0, 0.0)
    }

    pub fn vector_y() -> Vector3 {
        Vector3::new(0.0, 1.0, 0.0)
    }

    pub fn vector_z() -> Vector3 {
        Vector3::new(0.0, 0.0, 1.0)
    }

    pub fn dot(a: Vector3, b: Vector3) -> f32 {
        a.x * b.x + a.y * b.y + a.z * b.z
    }

    pub fn rotated(&self, rotation: Quaternion) -> Vector3 {
        let x2 = rotation.x + rotation.x;
        let y2 = rotation.y + rotation.y;
        let z2 = rotation.z + rotation.z;

        let wx2 = rotation.w * x2;
        let wy2 = rotation.w * y2;
        let wz2 = rotation.w * z2;
        let xx2 = rotation.x * x2;
        let xy2 = rotation.x * y2;
        let xz2 = rotation.x * z2;
        let yy2 = rotation.y * y2;
        let yz2 = rotation.y * z2;
        let zz2 = rotation.z * z2;

        Vector3::new(
            self.x * (1.0 - yy2 - zz2) + self.y * (xy2 - wz2) + self.z * (xz2 + wy2),
            self.x * (xy2 + wz2) + self.y * (1.0 - xx2 - zz2) + self.z * (yz2 - wx2),
            self.x * (xz2 - wy2) + self.y * (yz2 + wx2) + self.z * (1.0 - xx2 - yy2),
        )
    }

    pub fn to_2d_placed(self) -> Vector2 {
        let normalized = self.normalized();
        let y = normalized.z.sin();
        let x = normalized.y.atan2(normalized.x) / PI;
        Vector2::new(x * SIZE_F, y * SIZE_F)
    }
}

impl Neg for Vector3 {
    type Output = Vector3;
    fn neg(self) -> Vector3 {
        Vector3::new(-self.x, -self.y, -self.z)
    }
}

impl Clone for Vector3 {
    fn clone(&self) -> Self {
        Vector3::new(self.x, self.y, self.z)
    }
}
impl Mul for Vector3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Vector3::new(
            self.y * rhs.z + self.z * rhs.y,
            self.x * rhs.z + self.z * rhs.x,
            self.x * rhs.y + self.y * rhs.x,
        )
    }
}

struct Quaternion {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl Quaternion {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Quaternion {
        Quaternion { x, y, z, w }
    }

    pub fn from_axis_angle(axis: Vector3, angle: f32) -> Quaternion {
        let half_angle = angle / 2.0;
        let s = half_angle.sin();
        let c = half_angle.cos();
        Quaternion::new(axis.x * s, axis.y * s, axis.z * s, c)
    }
}

impl Clone for Vector2 {
    fn clone(&self) -> Self {
        Vector2::new_with_spin(self.x, self.y, self.orientation.clone())
    }
}

impl Sub for Vector2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Vector2::new_with_spin(self.x - rhs.x, self.y - rhs.y, self.orientation)
    }
}

impl Mul<f32> for Vector2 {
    type Output = Vector2;

    fn mul(self, rhs: f32) -> Self::Output {
        Vector2::new_with_spin(self.x * rhs, self.y * rhs, self.orientation)
    }
}
