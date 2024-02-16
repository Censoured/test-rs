//entity module

use crate::config;

use glam::Vec2;
use sdl2::rect::{Rect, Point};
use sdl2::keyboard::Keycode;

use config::*;
use sdl2::render::WindowCanvas;
use sdl2::render::Texture;

use rand::Rng;

#[derive(Clone)]
pub struct Transform {
    pub pos: Vec2,
    pub vel: Vec2,
    pub scale: Vec2,
    pub rot: f64
}

impl Transform {
    pub fn new() -> Self {
        Transform {
            pos: Vec2::new(0.0, 0.0),
            vel: Vec2::new(0.0, 0.0),
            scale: Vec2::new(0.0, 0.0),
            rot: 0.0
        }
    }
    pub fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
    }

    pub fn rotate_to_velocity(&mut self) {
        self.rot = 180.0 - (self.vel.x() as f64).atan2(self.vel.y() as f64).to_degrees();
    }

    pub fn rotate_to_vec2(&mut self, target: Vec2) {
        let d = target - self.pos;
        self.rot = 180.0 -(d.x() as f64).atan2(d.y() as f64).to_degrees();
    }

    pub fn get_rect(&self) -> Rect {
        return Rect::new(self.pos.x() as i32, self.pos.y() as i32, self.scale.x() as u32, self.scale.y() as u32);
    }
}

#[derive(Clone)]
pub struct Animation {
    total_frames: i32,
    current_frame: i32,
    first_row: i32,
    first_col: i32,
    frame_size: Point,
    frames_per_sec: i32,
    timer: f32,
    repeated: bool,
    finished: bool
}

impl Animation {
    pub fn new() -> Self {
        Animation {
            total_frames: 1,
            current_frame: 0,
            first_row: 0,
            first_col: 0,
            frame_size: Point::new(0,0),
            frames_per_sec: 0,
            timer: 0.0,
            repeated: true,
            finished: false
        }
    }

    pub fn construct(tf: i32, fr: i32, fc:i32, fs: Point, fps: i32, repeat: bool) -> Self {
        Animation {
            total_frames: tf,
            current_frame: 0,
            first_row: fr,
            first_col: fc,
            frame_size: fs,
            frames_per_sec: fps,
            timer: 0.0,
            repeated: repeat,
            finished: false
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
                if self.repeated {
                    self.current_frame = 0;
                }
                else
                {
                   self.finished = true; 
                }
            }
        }
    }

    pub fn draw(&mut self, canvas: &mut WindowCanvas, texture: &Texture, trans: &Transform) {
        canvas.copy_ex(&texture, 
            self.get_frame_rect(), 
            trans.get_rect(),
            trans.rot,
            None,
            false,
            false
        ).unwrap();
    }

    pub fn get_frame_rect(&self) -> Rect {
        Rect::new(
            self.first_col * self.frame_size.x + self.frame_size.x * self.current_frame,
             self.first_row * self.frame_size.y,
            self.frame_size.x as u32,
        self.frame_size.y as u32)
    }
}


#[derive(Debug, PartialEq, Clone)]
pub enum EntityType {
    Player,
    Enemy,
    Bullet,
    EnemyBullet,
    Particle,
    PowerupHealth,
    PowerupShield,
    PowerupBulletSpeed,
    PowerupNuke
}

#[derive(Clone)]
pub struct Entity {
    pub typ: EntityType,
    pub trans: Transform,
    pub anim: Animation,
    pub life: i32,
    pub shield: i32
}

impl Entity {
    pub fn new(t: EntityType) -> Self {
        Entity {
            trans: Transform::new(),
            anim: Animation::new(),
            life: 5,
            shield: 3,
            typ: t,
        }
    }

    pub fn get_rect(&self) -> Rect {
        self.trans.get_rect()
    }

    pub fn update_position(&mut self, keys: &[Keycode], dt: f32) {
        self.trans.vel = Vec2::zero();
        for key in keys {
            match key {
                Keycode::W => self.trans.vel -= Vec2::unit_y(),
                Keycode::A => self.trans.vel -= Vec2::unit_x(),
                Keycode::S => self.trans.vel += Vec2::unit_y(),
                Keycode::D => self.trans.vel += Vec2::unit_x(),
                _ => {}
            }
        }
        if self.trans.vel != Vec2::zero() {
            self.trans.vel = self.trans.vel.normalize();
            self.trans.vel *= PLAYER_SPEED;
            self.trans.update(dt);
        }
        
        //self.trans.pos += self.trans.vel;
    }

    pub fn update(&mut self, dt: f32) {
        let mut out_of_bounds = false;
        self.trans.update(dt);
        self.trans.rotate_to_velocity();
        if self.trans.pos.x() < -(self.trans.scale.x() * 2.0) {
            self.trans.vel = Vec2::new(-self.trans.vel.x(), self.trans.vel.y());
            self.trans.pos = Vec2::new(-self.trans.scale.x() * 2.0, self.trans.pos.y());
            out_of_bounds = true;
        }
        if self.trans.pos.y() < -(self.trans.scale.y() * 2.0) {
            self.trans.vel = Vec2::new(self.trans.vel.x(), -self.trans.vel.y());
            self.trans.pos = Vec2::new(self.trans.pos.x(), -self.trans.scale.y() * 2.0);
            out_of_bounds = true;
        }
        if self.trans.pos.x() > SCREEN_WIDTH as f32 + self.trans.scale.x() * 2.0 {
            self.trans.vel = Vec2::new(-self.trans.vel.x(), self.trans.vel.y());
            self.trans.pos = Vec2::new(SCREEN_WIDTH as f32 + self.trans.scale.x(), self.trans.pos.y());
            out_of_bounds = true;
        }
        if self.trans.pos.y() > SCREEN_HEIGHT as f32 + self.trans.scale.y() * 2.0 {
            self.trans.vel = Vec2::new(self.trans.vel.x(), -self.trans.vel.y());
            self.trans.pos = Vec2::new(self.trans.pos.x(), SCREEN_HEIGHT as f32 + self.trans.scale.y());
            out_of_bounds = true;
        }
        if (self.typ == EntityType::Bullet || self.typ == EntityType::EnemyBullet) && out_of_bounds {
            self.life = 0;
        }

        if self.typ == EntityType::Enemy && out_of_bounds {
            self.life -= 1;
        }
        self.anim.update(dt);
        if self.anim.finished && self.typ == EntityType::Particle {
            self.life = 0;
        }
        
    }

    pub fn draw(&mut self, canvas: &mut WindowCanvas, texture: &Texture) {
        self.anim.draw(canvas, texture, &self.trans);
    }

}

pub fn spawn_enemy() -> Entity {
    let mut e = Entity::new(EntityType::Enemy);
    e.trans.pos = Vec2::new(rand::thread_rng().gen_range(100..=SCREEN_WIDTH-108) as f32, rand::thread_rng().gen_range(100..=SCREEN_HEIGHT-108) as f32);
    while e.trans.vel == Vec2::zero() {
        e.trans.vel = Vec2::new(rand::thread_rng().gen_range(-ENEMY_SPEED..=ENEMY_SPEED), rand::thread_rng().gen_range(-ENEMY_SPEED..=ENEMY_SPEED));
    }
    e.anim = Animation::construct(1, rand::thread_rng().gen_range(0..=5), rand::thread_rng().gen_range(4..=9), Point::new(8,8), 1, true);
    e.trans.scale = Vec2::new(24.0, 24.0);
    e
}

pub fn spawn_powerup() -> Entity {
    let ptype = rand::thread_rng().gen_range(0..4);
    println!("ptype={}", ptype);
    let mut e = Entity::new(EntityType::PowerupHealth);
    e.trans.pos = Vec2::new(rand::thread_rng().gen_range(100..=SCREEN_WIDTH-108) as f32, rand::thread_rng().gen_range(100..=SCREEN_HEIGHT-108) as f32);
    e.trans.scale = Vec2::new(24.0, 24.0);
    match ptype {
        0 => {
            e.typ = EntityType::PowerupHealth;
            e.anim = Animation::construct(1, 0, 2, Point::new(8,8), 1, true);
        }
        1 => {
            e.typ = EntityType::PowerupShield;
            e.anim = Animation::construct(1, 0, 3, Point::new(8,8), 1, true);
        }
        2 => {
            e.typ = EntityType::PowerupNuke;
            e.anim = Animation::construct(1, 6, 11, Point::new(8,8), 1, true);
        }
        3 => {
            e.typ = EntityType::PowerupBulletSpeed;
            e.anim = Animation::construct(1, 1, 3, Point::new(8,8), 1, true);
        }
        _ => {}
    }
    e
}

pub fn spawn_particle(o: &mut Entity) -> Entity {
    let mut e = Entity::new(EntityType::Particle);
    e.trans.pos = o.trans.pos;
    let anim = rand::thread_rng().gen_range(1..=3);
    e.anim = Animation::construct(4, 0, 0, Point::new(8,8), 4, false);
    match anim {
        1 => { e.anim.first_col = 8; e.anim.first_row = 6; },
        2 => { e.anim.first_col = 9; e.anim.first_row = 6; },
        3 => { e.anim.first_col = 9; e.anim.first_row = 7; },
        _ => { e.anim.first_col = 8; e.anim.first_row = 6;  }
    }
    e.trans.scale = Vec2::new(24.0, 24.0);
    e
}

pub fn spawn_particle_shield(o: &Entity) -> Entity {
    let mut e = Entity::new(EntityType::Particle);
    e.trans.pos = Vec2::new(o.trans.pos.x() - 8.0, o.trans.pos.y() - 8.0);
    e.anim = Animation::construct(4, 2, 2, Point::new(16,16), 16, false);
    e.trans.scale = Vec2::new(40.0, 40.0);
    e
}

pub fn spawn_bullet(p: &Entity, v: &Vec2) -> Entity {
    let mut e = Entity::new(EntityType::Bullet);
    e.trans.pos = p.trans.pos;
    e.trans.vel = *v - p.trans.pos;
    e.trans.vel = e.trans.vel.normalize();
    e.trans.vel *= BULLET_SPEED;
    e.anim = Animation::construct(1, 1, 1, Point::new(8,8), 1, true);
    e.trans.scale = Vec2::new(24.0, 24.0);
    e.trans.rot = p.trans.rot;
    e
}

pub fn spawn_enemy_bullet(p: &Entity, v: &Vec2) -> Entity {
    let mut e = Entity::new(EntityType::EnemyBullet);
    e.trans.pos = p.trans.pos;
    e.trans.vel = *v - p.trans.pos;
    e.trans.vel = e.trans.vel.normalize();
    e.trans.vel *= BULLET_SPEED;
    e.anim = Animation::construct(1, 1, 0, Point::new(8,8), 1, true);
    e.trans.scale = Vec2::new(24.0, 24.0);
    e.trans.rot = p.trans.rot;
    e
}