extern crate sdl2;

mod entity;
mod config;
//mod renderer;


use rand::Rng;

use rusty_audio::Audio;

use sdl2::image::{InitFlag, LoadSurface};
use sdl2::render::{Texture, WindowCanvas};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::surface::Surface;

use glam::Vec2;

use config::*;
use entity::Entity;
use entity::EntityType;

fn spawn_enemy() -> Entity {
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

fn spawn_particle(o: &mut Entity) -> Entity {
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

fn spawn_bullet(p: &Entity, v: &Vec2) -> Entity {
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

fn draw_background(canvas: &mut WindowCanvas, texture: &Texture) {
    for x in (0..=SCREEN_WIDTH).step_by(127) {
        for y in (0..=SCREEN_HEIGHT).step_by(255) {
            canvas.copy_ex(&texture, 
                Rect::new(0,0,127,255), 
                Rect::new(x as i32,y as i32,127,255),
                0.0,
                None,
                false,
                false
            ).unwrap(); 
        }
    }
}

/* 
pub fn main() {
    //SDL Init stuff
    let mut renderer = SdlRenderer::init(SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();
    let mut event_pump = renderer.get_sdl_context().event_pump().unwrap();

    let font: sdl2::ttf::Font<'_, '_> = renderer.get_ttf_context().load_font("assets/fonts/Pono_188.ttf", 24).unwrap();

    'running: loop {
        renderer.clear_screen();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,

                Event::KeyDown { keycode: Some(keycode), .. } => {
                    if keycode == Keycode::Escape {
                        break 'running;
                    }
                }

                _ => {}
            }
        }

        renderer.draw_text(&font, format!("hello"), 10, 10);

        renderer.refresh();


        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
*/

pub fn main() {
    //SDL Init stuff
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG).unwrap();
    let window = video_subsystem.window("rust-sdl2 demo", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas()
        .build()
        .expect("failed to build window's canvas");
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();

    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();
    
    let bg_surface = Surface::from_file("assets/SpaceShooterAssetPack_BackGrounds.png").unwrap();
    let bg_texture = texture_creator.create_texture_from_surface(bg_surface).unwrap();
    
    let surface2 = Surface::from_file("assets/SpaceShooterAssetPack_Ships.png").map_err(|err| format!("failed to load cursor image: {}", err)).unwrap();
    let texture = texture_creator.create_texture_from_surface(surface2).unwrap();

    let bullets_surface = Surface::from_file("assets/SpaceShooterAssetPack_Projectiles.png").map_err(|err| format!("failed to load cursor image: {}", err)).unwrap();
    let bullets_texture = texture_creator.create_texture_from_surface(bullets_surface).unwrap();

    let particle_surface = Surface::from_file("assets/SpaceShooterAssetPack_Miscellaneous.png").map_err(|err| format!("failed to load cursor image: {}", err)).unwrap();
    let particle_texture = texture_creator.create_texture_from_surface(particle_surface).unwrap();

    let mut audio = Audio::new();
    audio.add("shoot", "assets/sfx/LASERSHOOT.wav"); // Load the sound, give it a name
    audio.add("explode", "assets/sfx/EXPLOSION.wav"); // Load the sound, give it a name

    // Load a font
    let font: sdl2::ttf::Font<'_, '_> = ttf_context.load_font("assets/fonts/Pono_188.ttf", 24).unwrap();
    //font.set_style(sdl2::ttf::FontStyle::BOLD);

    let mut score = 0;

    let mut score_string = format!("SCORE: {}", score);
    let dead1_string = format!("GAME OVER");
    let dead2_string = format!("PRESS 'R' TO RESTART");

    let mut surface = font
        .render(score_string.as_str())
        .blended(Color::RGBA(255, 0, 0, 255))
        .map_err(|e| e.to_string()).unwrap();
    let mut font_texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string()).unwrap();

    let mut player = Entity::new(EntityType::Player);
    player.size = 24;
    player.tex_coords = Rect::new(8,0, 8,8);

    let mut enemies: Vec<Entity> = Vec::new();
    let mut bullets: Vec<Entity> = Vec::new();
    let mut particles: Vec<Entity> = Vec::new();

    enemies.push(spawn_enemy());
    enemies.push(spawn_enemy());
    enemies.push(spawn_enemy());

    let timer = sdl_context.timer().unwrap();
    let mut ticks = timer.ticks();
    let mut time_elapsed: u32 = 0;

    let mut delta_time;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    'running: loop {

        delta_time = (timer.ticks() as f32 - ticks as f32) / 1000.0;
        // The rest of the game loop goes here...
        time_elapsed += timer.ticks() - ticks;
        ticks = timer.ticks();
        
        //println!("dt={}", delta_time);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,

                Event::KeyDown { keycode: Some(keycode), .. } => {
                    if keycode == Keycode::Escape {
                        break 'running;
                    }
                    if keycode == Keycode::R {
                        if player.life == 0
                        {
                            player.life = 5;
                            player.pos = Vec2::new(400.0, 300.0);
                            enemies.clear();
                            enemies.push(spawn_enemy());
                            enemies.push(spawn_enemy());
                            enemies.push(spawn_enemy());
                            score = 0;
                        }
                    }
                }

                Event::MouseButtonDown { x, y, .. } => {
                    //println!("mouse btn down at ({},{})", x, y);
                   
                    if player.life > 0 {
                        let m_pos = Vec2::new(x as f32,y as f32);
                        bullets.push(spawn_bullet(&player, &m_pos));
                        audio.play("shoot");
                    }
                }

                Event::MouseMotion { x, y, .. } => {
                    let m_pos = Vec2::new(x as f32,y as f32);
                    let d = m_pos - Vec2::new(player.pos.x() + 4.0, player.pos.y() + 4.0);
                    player.rot = 180.0 -(d.x() as f64).atan2(d.y() as f64).to_degrees();
                }

                //Event::MouseButtonUp { x, y, .. } => {
                    //println!("mouse btn up at ({},{})", x, y);
                //}

                _ => {}
            }
        }

        let keys: Vec<Keycode> = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        player.update_position(&keys, delta_time);

        if time_elapsed > SPAWN_RATE {
            enemies.push(spawn_enemy());
            time_elapsed = 0;
        }       

        for e in &mut enemies {
            e.update(delta_time);
        }

        for e in &mut bullets {
            e.update(delta_time);
        }

        for e in &mut particles {
            e.update(delta_time);
        }
        
        let p_rect = player.get_rect();
        for e in &mut enemies {
            let e_rect = e.get_rect();
            if p_rect.has_intersection(e_rect) && player.life > 0 {
                e.life = 0;
                player.life = 0;
                particles.push(spawn_particle(e));
                particles.push(spawn_particle(&mut player));
                audio.play("explode");
            }
            for b in &mut bullets {
                let b_rect = Rect::new(b.pos.x() as i32, b.pos.y() as i32, b.size, b.size);

                // Check if the rectangles collide
                if b_rect.has_intersection(e_rect) {
                    e.life = 0;
                    b.life = 0;
                    particles.push(spawn_particle(e));
                    audio.play("explode");
                    score += 500;
                }
            }
        }

        enemies.retain(|e| e.life > 0);
        bullets.retain(|e| e.life > 0);
        particles.retain(|e| e.life > 0);
       
        // Render

        canvas.clear();
        draw_background(&mut canvas, &bg_texture);

        for e in &mut enemies {
            e.draw(&mut canvas, &texture);
        }

        for e in &mut bullets {
            e.draw(&mut canvas, &bullets_texture);
        }

        for e in &mut particles {
            e.draw(&mut canvas, &particle_texture);
        }

        if player.life > 0 {
            player.draw(&mut canvas, &texture);
        }
        
        score_string = format!("SCORE: {}", score);
        surface = font.render(score_string.as_str()).solid(Color::RGBA(255, 255, 255, 255)).unwrap();
        font_texture = texture_creator.create_texture_from_surface(&surface).unwrap();

        let size = font.size_of(&score_string.as_str()).unwrap();
        let target = Rect::new(10,10, size.0, size.1);
        canvas.copy(&font_texture, None, Some(target)).unwrap();

        if player.life == 0 {
            surface = font.render(dead1_string.as_str()).solid(Color::RGBA(255, 255, 255, 255)).unwrap();
            font_texture = texture_creator.create_texture_from_surface(&surface).unwrap();
            let mut size = font.size_of(&dead1_string.as_str()).unwrap();
            let mut target = Rect::new(300,250, size.0, size.1);
            canvas.copy(&font_texture, None, Some(target)).unwrap();

            
            surface = font.render(dead2_string.as_str()).solid(Color::RGBA(255, 255, 255, 255)).unwrap();
            font_texture = texture_creator.create_texture_from_surface(&surface).unwrap();
            size = font.size_of(&dead2_string.as_str()).unwrap();
            target = Rect::new(180,350, size.0, size.1);
            canvas.copy(&font_texture, None, Some(target)).unwrap();
        }
    
        

        canvas.present();


        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}