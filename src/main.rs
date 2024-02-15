extern crate sdl2;

mod entity;
mod config;
mod asset_manager;
//mod renderer;

use asset_manager::{FontManager, TextureManager};

use rusty_audio::Audio;

use sdl2::image::InitFlag;
use sdl2::mouse::MouseButton;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;

use glam::Vec2;

use config::*;
use entity::*;
use entity::EntityType;
use sdl2::ttf::Font;
use sdl2::video::WindowContext;

fn draw_background(canvas: &mut WindowCanvas, texture: &Texture) -> Result <(), String> {
    for x in (0..=SCREEN_WIDTH).step_by(127) {
        for y in (0..=SCREEN_HEIGHT).step_by(255) {
            canvas.copy_ex(&texture, 
                Rect::new(0,0,127,255), 
                Rect::new(x as i32,y as i32,127,255),
                0.0,
                None,
                false,
                false
            )?; 
        }
    }
    Ok(())
}

fn draw_string(str: String, x: i32, y: i32, canvas: &mut WindowCanvas, font: &Font, texture_creator: &TextureCreator<WindowContext>) -> Result <(), String> {
    let surface = font.render(str.as_str()).solid(Color::RGBA(255, 255, 255, 255)).map_err(|e| e.to_string()).map_err(|e| e.to_string())?;
    let font_texture = texture_creator.create_texture_from_surface(&surface).map_err(|e| e.to_string()).map_err(|e| e.to_string())?;
    let size = font.size_of(&str.as_str()).unwrap();
    let target = Rect::new(x,y, size.0, size.1);
    canvas.copy(&font_texture, None, Some(target))?;
    Ok(())
}

pub fn main() -> Result<(), String> {
    //SDL Init stuff
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let window = video_subsystem.window("rust-sdl2 demo", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .expect("failed to build window");
    let mut canvas = window.into_canvas()
        .build()
        .expect("failed to build window's canvas");
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();

    // Load a font
    let font: sdl2::ttf::Font<'_, '_> = ttf_context.load_font("assets/fonts/Pono_188.ttf", 24)?;
    //font.set_style(sdl2::ttf::FontStyle::BOLD);

    let mut score = 0;

    let mut event_pump = sdl_context.event_pump()?;

    let mut texture_manager = TextureManager::new(&texture_creator);
    let mut _font_manager = FontManager::new(&ttf_context);
    

    let bg_texture = texture_manager.load("assets/SpaceShooterAssetPack_BackGrounds.png")?;
    let texture = texture_manager.load("assets/SpaceShooterAssetPack_Ships.png")?;
    let bullets_texture = texture_manager.load("assets/SpaceShooterAssetPack_Projectiles.png")?;
    let particle_texture = texture_manager.load("assets/SpaceShooterAssetPack_Miscellaneous.png")?;

    let mut audio = Audio::new();
    audio.add("shoot", "assets/sfx/LASERSHOOT.wav"); // Load the sound, give it a name
    audio.add("explode", "assets/sfx/EXPLOSION.wav"); // Load the sound, give it a name
    audio.add("powerup_spawn", "assets/sfx/POWERUP.wav"); // Load the sound, give it a name
    audio.add("powerup_collect", "assets/sfx/POWER_UP3.wav"); // Load the sound, give it a name

    let mut player = Entity::new(EntityType::Player);
    player.size = 24;
    player.tex_coords = Rect::new(8,0, 8,8);

    let mut enemies: Vec<Entity> = Vec::new();
    let mut bullets: Vec<Entity> = Vec::new();
    let mut particles: Vec<Entity> = Vec::new();
    let mut powerups: Vec<Entity> = Vec::new();

    enemies.push(spawn_enemy());
    enemies.push(spawn_enemy());
    enemies.push(spawn_enemy());

    let timer = sdl_context.timer()?;
    let mut ticks = timer.ticks();
    let mut time_elapsed: u32 = 0;
    let mut time_powerup: u32 = 0;
    let mut delta_time;


    canvas.set_draw_color(Color::RGB(0, 0, 0));
    'running: loop {

        delta_time = (timer.ticks() as f32 - ticks as f32) / 1000.0;
        // The rest of the game loop goes here...
        time_elapsed += timer.ticks() - ticks;
        time_powerup += timer.ticks() - ticks;
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
                            powerups.clear();
                            enemies.push(spawn_enemy());
                            enemies.push(spawn_enemy());
                            enemies.push(spawn_enemy());
                            score = 0;
                        }
                    }
                }

                Event::MouseButtonDown { x, y, .. } => {               
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

                _ => {}
            }
        }

        if event_pump.mouse_state().is_mouse_button_pressed(MouseButton::Left) {
            let m_pos = Vec2::new(event_pump.mouse_state().x() as f32, event_pump.mouse_state().y() as f32);
            if ticks % 450 < 4 {
                bullets.push(spawn_bullet(&player, &m_pos));
                audio.play("shoot");
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

        if time_powerup > POWERUP_RATE {
            powerups.push(spawn_powerup());
            audio.play("powerup_spawn");
            time_powerup = 0;
        }     

        for e in &mut enemies {
            e.update(delta_time);
            if player.life > 0 {
                let dist = e.pos - player.pos;
                if dist.length_squared() < 200.0 * 200.0 {
                    let d = player.pos - Vec2::new(e.pos.x() + 4.0, e.pos.y() + 4.0);
                    e.rot = 180.0 -(d.x() as f64).atan2(d.y() as f64).to_degrees();
                    if ticks % 800 < 2 {
                        bullets.push(spawn_enemy_bullet(&e, &player.pos));
                        audio.play("shoot");
                    }
                }
            }
        }

        for e in &mut bullets {
            e.update(delta_time);
        }

        for e in &mut particles {
            e.update(delta_time);
        }
        
        let p_rect = player.get_rect();
        for p in &mut powerups {
            let e_rect = p.get_rect();
            if p_rect.has_intersection(e_rect) && player.life > 0 {
                p.life = 0;
                player.life += 1;
                audio.play("powerup_collect");
            }
        }

        for e in &mut enemies {
            let e_rect = e.get_rect();
            if p_rect.has_intersection(e_rect) && player.life > 0 {
                e.life = 0;
                player.life -= 1;
                particles.push(spawn_particle(e));
                if player.life == 0 {
                    particles.push(spawn_particle(&mut player));
                }
                audio.play("explode");
            }
            for b in &mut bullets {
                let b_rect = Rect::new(b.pos.x() as i32, b.pos.y() as i32, b.size, b.size);
                // Check if the rectangles collide
                if b.typ == EntityType::Bullet && b_rect.has_intersection(e_rect)  {
                    e.life = 0;
                    b.life = 0;
                    particles.push(spawn_particle(e));
                    audio.play("explode");
                    score += 500;
                }
            }
        }

        for b in &mut bullets {
            let b_rect = Rect::new(b.pos.x() as i32, b.pos.y() as i32, b.size, b.size);

            if b.typ == EntityType::EnemyBullet && b_rect.has_intersection(p_rect)  {
                if player.life > 0 {
                    player.life -= 1;
                    b.life = 0;
                    if player.life == 0 {
                        particles.push(spawn_particle(&mut player));
                        audio.play("explode");
                    }
                }
            }
        }

        enemies.retain(|e| e.life > 0);
        bullets.retain(|e| e.life > 0);
        particles.retain(|e| e.life > 0);
        powerups.retain(|e| e.life > 0);
       
        // Render

        canvas.clear();
        draw_background(&mut canvas, &bg_texture)?;

        for e in &mut enemies {
            e.draw(&mut canvas, &texture);
        }

        for e in &mut bullets {
            e.draw(&mut canvas, &bullets_texture);
        }

        for e in &mut particles {
            e.draw(&mut canvas, &particle_texture);
        }

        for e in &mut powerups {
            e.draw(&mut canvas, &particle_texture);
        }

        if player.life > 0 {
            player.draw(&mut canvas, &texture);
        }
        
        draw_string(format!("SCORE: {}", score), 10, 10, &mut canvas, &font, &texture_creator)?;

        for i in 0..player.life {
            canvas.copy_ex(&particle_texture, 
                Rect::new(16,0,8,8), 
                Rect::new((((i % 8 )*22) + 10) as i32 ,40 + ((i / 8) * 22),20,20),
                0.0, 
                None,
                false,
                false
            )?;
        }

        if player.life == 0 {
            draw_string(format!("GAME OVER"), 300, 250, &mut canvas, &font, &texture_creator)?;
            draw_string(format!("PRESS 'R' TO RESTART"), 180, 350, &mut canvas, &font, &texture_creator)?;
        }

        canvas.present();

        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}