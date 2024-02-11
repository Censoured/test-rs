//entity module

use crate::config;

use glam::Vec2;
use sdl2::rect::Rect;
use sdl2::keyboard::Keycode;

use config::*;
use sdl2::render::WindowCanvas;
use sdl2::render::Texture;



#[derive(Debug, PartialEq)]
pub enum EntityType {
    Player,
    Enemy,
    Bullet,
    Particle
}

pub struct Entity {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: i8,
    pub tex_coords: Rect,
    pub size: u32,
    pub rot: f64,
    typ: EntityType,
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

    pub fn update_position(&mut self, keys: &[Keycode], dt: f32) {
        for key in keys {
            match key {
                Keycode::W => self.pos = self.pos - Vec2::unit_y() * PLAYER_SPEED * dt,
                Keycode::A => self.pos = self.pos - Vec2::unit_x() * PLAYER_SPEED * dt,
                Keycode::S => self.pos = self.pos + Vec2::unit_y() * PLAYER_SPEED * dt,
                Keycode::D => self.pos = self.pos + Vec2::unit_x() * PLAYER_SPEED * dt,
                _ => {}
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        let mut out_of_bounds = false;
        self.timer += dt;
        self.pos += self.vel * dt;
        self.rot = -(self.vel.x() as f64).atan2(self.vel.y() as f64).to_degrees();
        if self.pos.x() < 0.0 {
            self.vel = Vec2::new(-self.vel.x(), self.vel.y());
            self.pos = Vec2::new(0.0, self.pos.y());
            out_of_bounds = true;
        }
        if self.pos.y() < 0.0 {
            self.vel = Vec2::new(self.vel.x(), -self.vel.y());
            self.pos = Vec2::new(self.pos.x(), 0.0);
            out_of_bounds = true;
        }
        if self.pos.x() as u32 > SCREEN_WIDTH - self.size {
            self.vel = Vec2::new(-self.vel.x(), self.vel.y());
            self.pos = Vec2::new((SCREEN_WIDTH - self.size) as f32, self.pos.y());
            out_of_bounds = true;
        }
        if self.pos.y() as u32 > SCREEN_HEIGHT - self.size {
            self.vel = Vec2::new(self.vel.x(), -self.vel.y());
            self.pos = Vec2::new(self.pos.x(), (SCREEN_HEIGHT - self.size) as f32);
            out_of_bounds = true;
        }
        if self.typ == EntityType::Bullet && out_of_bounds {
            self.life = 0;
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