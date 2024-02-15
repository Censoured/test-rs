//entity module

use crate::config;

use glam::Vec2;
use sdl2::rect::{Rect, Point};
use sdl2::keyboard::Keycode;

use config::*;
use sdl2::render::WindowCanvas;
use sdl2::render::Texture;

use rand::Rng;

pub trait GameObject {
    fn update(&mut self, dt: f32);
    fn render(&mut self, canvas: &mut WindowCanvas, texture: &Texture);
}

struct Enemy {
    transf: Transform,
    anim: Animation
}

impl GameObject for Enemy {
    fn update(&mut self, dt: f32) {
        self.transf.update(dt);
        self.anim.update(dt);
    }

    fn render(&mut self, canvas: &mut WindowCanvas, texture: &Texture) {
        canvas.copy_ex(&texture, 
            Some(self.anim.get_frame_rect()), 
            Some(Rect::new(self.transf.pos.x() as i32,self.transf.pos.y() as i32,self.transf.scale.x() as u32,self.transf.scale.y() as u32)),
            self.transf.rot as f64,
            None,
            false,
            false
        ).unwrap();
    }
}

struct Transform {
    pub pos: Vec2,
    pub vel: Vec2,
    pub scale: Vec2,
    pub rot: f32
}

impl Transform {
    pub fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
    }
}

struct Animation {
    total_frames: i32,
    current_frame: i32,
    first_row: i32,
    first_col: i32,
    frame_size: Point,
    frames_per_sec: i32,
    timer: f32
}

impl Animation {
    pub fn new(tf: i32, fr: i32, fc:i32, fs: Point, fps: i32) -> Self {
        Animation {
            total_frames: tf,
            current_frame: 0,
            first_row: fr,
            first_col: fc,
            frame_size: fs,
            frames_per_sec: fps,
            timer: 0.0
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.total_frames == 1 {
            return;
        }
        self.timer += dt;
        if self.timer > 1.0/(self.frames_per_sec as f32) {
            self.timer = 0.0;
            self.current_frame += 1;

            if self.current_frame == self.total_frames {
                self.current_frame = 0;
            }
        }
    }

    pub fn get_frame_rect(&self) -> Rect {
        Rect::new(
            self.first_col * self.frame_size.x + self.frame_size.x * self.current_frame,
             self.first_row * self.frame_size.y,
            self.frame_size.x as u32,
        self.frame_size.y as u32)
    }
}


#[derive(Debug, PartialEq)]
pub enum EntityType {
    Player,
    Enemy,
    Bullet,
    EnemyBullet,
    Particle,
    Powerup
}

pub struct Entity {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: i32,
    pub tex_coords: Rect,
    pub size: u32,
    pub rot: f64,
    pub typ: EntityType,
    pub frames: i8,
    frame_count: i8,
    timer: f32
}

impl Entity {
    pub fn new(t: EntityType) -> Self {
        Entity {
            pos: Vec2::new(400.0, 300.0),
            vel: Vec2::new(0.0, 0.0),
            life: 5,
            tex_coords : Rect::new(0, 0, 0, 0),
            size: 50,
            rot: 0.0,
            typ: t,
            frames: 1,
            frame_count: 0,
            timer: 0.0
        }
    }

    pub fn get_rect(&self) -> Rect {
        return Rect::new(self.pos.x() as i32, self.pos.y() as i32, self.size, self.size);
    }

    pub fn update_position(&mut self, keys: &[Keycode], dt: f32) {
        for key in keys {
            match key {
                Keycode::W => self.vel -= Vec2::unit_y(),
                Keycode::A => self.vel -= Vec2::unit_x(),
                Keycode::S => self.vel += Vec2::unit_y(),
                Keycode::D => self.vel += Vec2::unit_x(),
                _ => {}
            }
        }
        self.vel.normalize();
        self.vel *= PLAYER_SPEED * dt;
        self.pos += self.vel;
    }

    pub fn update(&mut self, dt: f32) {
        let mut out_of_bounds = false;
        self.timer += dt;
        self.pos += self.vel * dt;
        self.rot = -(self.vel.x() as f64).atan2(self.vel.y() as f64).to_degrees();
        if self.pos.x() < -((self.size * 2) as f32) {
            self.vel = Vec2::new(-self.vel.x(), self.vel.y());
            self.pos = Vec2::new(-((self.size * 2) as f32), self.pos.y());
            out_of_bounds = true;
        }
        if self.pos.y() < -((self.size * 2) as f32) {
            self.vel = Vec2::new(self.vel.x(), -self.vel.y());
            self.pos = Vec2::new(self.pos.x(), -((self.size * 2) as f32));
            out_of_bounds = true;
        }
        if self.pos.x() as u32 > SCREEN_WIDTH + self.size * 2 {
            self.vel = Vec2::new(-self.vel.x(), self.vel.y());
            self.pos = Vec2::new((SCREEN_WIDTH - self.size) as f32, self.pos.y());
            out_of_bounds = true;
        }
        if self.pos.y() as u32 > SCREEN_HEIGHT + self.size * 2 {
            self.vel = Vec2::new(self.vel.x(), -self.vel.y());
            self.pos = Vec2::new(self.pos.x(), (SCREEN_HEIGHT - self.size) as f32);
            out_of_bounds = true;
        }
        if (self.typ == EntityType::Bullet || self.typ == EntityType::EnemyBullet) && out_of_bounds {
            self.life = 0;
        }

        if self.typ == EntityType::Enemy && out_of_bounds {
            self.life -= 1;
        }
        if self.timer > 0.5/(self.frames as f32) && self.typ == EntityType::Particle {
            self.timer = 0.0;
            self.frame_count += 1;
            //println!("Frame: {}", self.frame_count);
            if self.frame_count == self.frames {
                self.frame_count = 0;
                self.life = 0;
            }
            self.tex_coords = Rect::new(
                self.tex_coords.x() + self.tex_coords.width() as i32,
                self.tex_coords.y(),
                self.tex_coords.width(),
                self.tex_coords.height()
            ); 
            //println!("TexCoords: {},{},{},{}", self.tex_coords.x(), self.tex_coords.y(), self.tex_coords.width(), self.tex_coords.height());
        }
        
    }

    pub fn draw(&mut self, canvas: &mut WindowCanvas, texture: &Texture) {
        canvas.copy_ex(&texture, 
            Some(self.tex_coords), 
            Some(Rect::new(self.pos.x() as i32,self.pos.y() as i32,self.size,self.size)),
            self.rot as f64,
            None,
            false,
            false
        ).unwrap();
    }

}

pub fn spawn_enemy() -> Entity {
    let mut e = Entity::new(EntityType::Enemy);
    e.pos = Vec2::new(rand::thread_rng().gen_range(100..=SCREEN_WIDTH-108) as f32, rand::thread_rng().gen_range(100..=SCREEN_HEIGHT-108) as f32);
    while e.vel == Vec2::zero() {
        e.vel = Vec2::new(rand::thread_rng().gen_range(-ENEMY_SPEED..=ENEMY_SPEED), rand::thread_rng().gen_range(-ENEMY_SPEED..=ENEMY_SPEED));
    }
    e.tex_coords = Rect::new(rand::thread_rng().gen_range(4..=9)*8,rand::thread_rng().gen_range(0..=5)*8,8,8);
    e.size = 24;
    //println!("Spawned enemy @[{}, {}] with velocity[{}, {}]", e.pos.x(), e.pos.y(), e.vel.x(),e.vel.y());
    e
}

pub fn spawn_powerup() -> Entity {
    let mut e = Entity::new(EntityType::Powerup);
    e.pos = Vec2::new(rand::thread_rng().gen_range(100..=SCREEN_WIDTH-108) as f32, rand::thread_rng().gen_range(100..=SCREEN_HEIGHT-108) as f32);
    e.tex_coords = Rect::new(0, 4*8,8,8);
    e.size = 24;
    e
}

pub fn spawn_particle(o: &mut Entity) -> Entity {
    let mut e = Entity::new(EntityType::Particle);
    e.pos = o.pos;
    let anim = rand::thread_rng().gen_range(1..=3);
    match anim {
        1 => {e.tex_coords = Rect::new(9*8, 6*8,8,8); },
        2 => {e.tex_coords = Rect::new(10*8, 6*8,8,8); },
        3 => {e.tex_coords = Rect::new(10*8, 7*8,8,8); },
        _ => {e.tex_coords = Rect::new(9*8, 6*8,8,8); }
    }
    //e.tex_coords = Rect::new(9*8, 6*8,8,8);
    e.size = 24;
    e.frames = 4;
    //println!("Spawned enemy @[{}, {}] with velocity[{}, {}]", e.pos.x(), e.pos.y(), e.vel.x(),e.vel.y());
    e
}

pub fn spawn_bullet(p: &Entity, v: &Vec2) -> Entity {
    let mut e = Entity::new(EntityType::Bullet);
    e.pos = p.pos;
    e.vel = *v - p.pos;
    e.vel = e.vel.normalize();
    e.vel *= BULLET_SPEED;
    e.tex_coords = Rect::new(8,8,8,8);
    e.size = 24;
    e.rot = p.rot;
    //println!("Spawned bullet @[{}, {}] with velocity[{}, {}]", e.pos.x(), e.pos.y(), e.vel.x(),e.vel.y());
    e
}

pub fn spawn_enemy_bullet(p: &Entity, v: &Vec2) -> Entity {
    let mut e = Entity::new(EntityType::EnemyBullet);
    e.pos = p.pos;
    e.vel = *v - p.pos;
    e.vel = e.vel.normalize();
    e.vel *= BULLET_SPEED;
    e.tex_coords = Rect::new(0,8,8,8);
    e.size = 24;
    e.rot = p.rot;
    //println!("Spawned bullet @[{}, {}] with velocity[{}, {}]", e.pos.x(), e.pos.y(), e.vel.x(),e.vel.y());
    e
}