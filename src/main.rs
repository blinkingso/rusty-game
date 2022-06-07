use rand::prelude::*;
use rusty_engine::prelude::*;
use std::{default::Default, time::Instant};

/// Car speed rate controller.
const PLAYER_SPEED: f32 = 250.0;
/// Road speed
const ROAD_SPEED: f32 = 400.0;
/// Obstacles speed
const OBSTACLES_SPEED: f32 = 500.0;

/// store game states in the struct.
struct GameState {
    // keep track of the player's health.
    health_amout: u8,
    // game finished if true
    lost: bool,
    start_time: Instant,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            health_amout: 5,
            lost: false,
            start_time: Instant::now(),
        }
    }
}

fn main() {
    let mut game = Game::new();
    let player = game.add_sprite("player", SpritePreset::RacingCarBlue);
    player.translation.x = -500.0;
    player.layer = 10.0;
    player.collision = true;
    for i in 0..20 {
        let roadline = game.add_sprite(format!("roadline{}", i), SpritePreset::RacingBarrierWhite);
        roadline.scale = 0.1;
        roadline.translation.x = -600.0 + 150.0 * i as f32;
    }
    game.audio_manager
        .play_music(MusicPreset::WhimsicalPopsicle, 0.2);

    // obstacles.
    let obstacle_presets = vec![
        SpritePreset::RacingCarBlue,
        SpritePreset::RacingCarRed,
        SpritePreset::RacingConeStraight,
    ];
    for (i, preset) in obstacle_presets.into_iter().enumerate() {
        let obstacle = game.add_sprite(format!("obstacle{}", i), preset);
        obstacle.layer = 5.0;
        obstacle.collision = true;
        obstacle.translation.x = thread_rng().gen_range(800.0..1600.0);
        obstacle.translation.y = thread_rng().gen_range(-300.0..300.0);
    }

    // health message
    let health_message = game.add_text("health_message", "Health: 5");
    health_message.translation = Vec2::new(550.0, 320.0);
    // game logic
    game.add_logic(game_logic);

    // Run the game, with an initial state
    game.run(GameState::default());
}

fn get_obstacles_speed(start_time: &Instant) -> f32 {
    let time = start_time.elapsed().as_secs();
    if time > 0 {
        return OBSTACLES_SPEED + (10 * time) as f32;
    }
    OBSTACLES_SPEED
}

fn game_logic(engine: &mut Engine, game_state: &mut GameState) {
    for sprite in engine.sprites.values_mut() {
        if sprite.label.starts_with("roadline") {
            sprite.translation.x -= ROAD_SPEED * engine.delta_f32;
            if sprite.translation.x < -675.0 {
                sprite.translation.x += 1500.0;
            }
        }
        if sprite.label.starts_with("obstacle") {
            sprite.translation.x -= get_obstacles_speed(&game_state.start_time) * engine.delta_f32;
            if sprite.translation.x < -800.0 {
                // reset like new obstacle recreated.
                sprite.translation.x = thread_rng().gen_range(800.0..1600.0);
                sprite.translation.y = thread_rng().gen_range(-300.0..300.0);
            }
        }
    }

    // Deal with collisions
    let health_message = engine.texts.get_mut("health_message").unwrap();
    for event in engine.collision_events.drain(..) {
        if !event.pair.either_contains("player") || event.state.is_end() {
            continue;
        }
        if game_state.health_amout > 0 {
            game_state.health_amout -= 1;
            health_message.value = format!("Health: {}", game_state.health_amout);
            engine.audio_manager.play_sfx(SfxPreset::Impact3, 0.5);
        }
    }
    let (mut x_direction, mut y_direction) = (0.0, 0.0);
    if engine.keyboard_state.pressed(KeyCode::Up) {
        y_direction += 1.0;
    }
    if engine.keyboard_state.pressed(KeyCode::Down) {
        y_direction -= 1.0;
    }
    if engine.keyboard_state.pressed(KeyCode::Right) {
        x_direction += 1.0;
    }
    if engine.keyboard_state.pressed(KeyCode::Left) {
        x_direction -= 1.0;
    }
    let player = engine.sprites.get_mut("player").unwrap();
    player.translation.y += y_direction * PLAYER_SPEED * engine.delta_f32;
    player.rotation = y_direction * 0.15;
    player.translation.x += x_direction * PLAYER_SPEED * engine.delta_f32;
    if player.translation.x < -600.0 || player.translation.x > 1500.0 {
        game_state.health_amout = 0;
    }
    if player.translation.y <= -360.0 || player.translation.y > 360.0 {
        game_state.health_amout = 0;
    }
    if game_state.health_amout == 0 {
        game_state.lost = true;
        let game_over = engine.add_text("gameover", "Game Over!");
        game_over.font_size = 128.0;
        engine.audio_manager.stop_music();
        engine.audio_manager.play_sfx(SfxPreset::Jingle3, 0.5);
    }
    if game_state.lost {
        return;
    }
}
