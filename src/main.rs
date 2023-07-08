use bevy::{math::Vec3Swizzles, prelude::*, sprite::collide_aabb::collide};

const TIME_STEP: f32 = 1.0 / 60.0;
const BOUNDS: Vec2 = Vec2::new(600.0, 600.0);
const SHOT_SPEED: f32 = 10.0;
const SHOT_INTERVAL: u8 = 4; // actual shot interval = (time_step / shot_interval)
const ENEMY_SPEED: f32 = 200.0;
const METEOR_SPEED: f32 = 4.0;
const BG_COLOR_HEX: &'static str = "#222034";
const BORDER_COLOR_HEX: &'static str = "#FBF236";

fn main() {
    let bg_color = Color::hex(BG_COLOR_HEX).unwrap(); // fuchsia

    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_systems(
            (
                player_movement_system,
                snap_to_player_system,
                maybe_spawn_enemy_system,
                spawn_shots_system,
                check_for_collisions_system,
                apply_movement_vector_system,
                outside_removal_system,
            )
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .insert_resource(ClearColor(bg_color))
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .insert_resource(ShotCounter { value: 0 })
        .insert_resource(Score { value: 0 })
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Resource)]
struct ShotCounter {
    value: u8,
}

#[derive(Resource)]
struct Score {
    value: u64,
}

#[derive(Component)]
struct Shot;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct SimpleMovement {
    movement_vector: Vec3,
}

#[derive(Component)]
struct Player {
    movement_speed: f32,
}

#[derive(Component)]
struct MeteorEnemy;

#[derive(Component)]
struct DrillEnemy;

fn check_for_collisions_system(
    mut commands: Commands,
    player_query: Query<(&Player, &Transform)>,
    shot_query: Query<(Entity, &Shot, &Transform)>,
    enemies_query: Query<(Entity, &Enemy, &Transform)>,
    focused_windows_query: Query<'_, '_, (Entity, &Window), ()>,
) {
    let shot_size_vec = Vec2::new(2., 4.);
    let enemy_size_vec = Vec2::new(64., 64.);
    let player_size_vec = Vec2::new(20., 20.);
    let (_, player_transform) = player_query.single();

    for (enemy_entity, _enemy, enemy_transform) in &enemies_query {
        match collide(
            player_transform.translation,
            player_size_vec,
            enemy_transform.translation,
            enemy_size_vec,
        ) {
            Some(_) => {
                for (window, _) in &focused_windows_query {
                    commands.entity(window).despawn();
                }
            }
            None => {}
        }

        for (shot_entity, _shot, shot_transform) in &shot_query {
            match collide(
                shot_transform.translation,
                shot_size_vec,
                enemy_transform.translation,
                enemy_size_vec,
            ) {
                Some(_) => {
                    // TODO: the entity might not exist when the command is executed,
                    // due to multiple collisions between same entities
                    commands.entity(enemy_entity).despawn_recursive();
                    commands.entity(shot_entity).despawn_recursive();
                }
                None => {}
            }
        }
    }
}

fn maybe_spawn_enemy_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let n = rand::random::<u8>();
    if n > 240 {
        let drill_handle = asset_server.load("textures/v0idp/drill.png");
        let x = (BOUNDS.x / 2.0) - (rand::random::<f32>() * (BOUNDS.x));
        let y = BOUNDS.y;

        commands.spawn((
            SpriteBundle {
                texture: drill_handle,
                transform: Transform::from_xyz(x, y, 0.0).with_scale(Vec3::new(2.0, 2.0, 1.0)),
                ..default()
            },
            DrillEnemy,
            Enemy,
        ));
    } else if n > 220 {
        let meteor_handle = asset_server.load("textures/v0idp/m2.png");
        let x = (rand::random::<f32>() * BOUNDS.x) - (BOUNDS.x * 0.5);
        let y = BOUNDS.y / 2.0;

        let movement_x = (rand::random::<f32>() - 0.5) * METEOR_SPEED;
        let movement_y = f32::max(0.5, rand::random::<f32>()) * METEOR_SPEED * -1.0;

        let movement_vec = Vec3::new(movement_x, movement_y, 0.0);

        commands.spawn((
            SpriteBundle {
                texture: meteor_handle,
                transform: Transform::from_xyz(x, y, 0.0),
                ..default()
            },
            SimpleMovement {
                movement_vector: movement_vec,
            },
            MeteorEnemy,
            Enemy,
        ));
    }
}

fn spawn_shots_system(
    mut commands: Commands,
    mut shot_counter: ResMut<ShotCounter>,
    asset_server: Res<AssetServer>,
    mut query: Query<(&Player, &Transform)>,
) {
    shot_counter.value += 1;
    if shot_counter.value == SHOT_INTERVAL {
        let shot_handle = asset_server.load("textures/v0idp/shot.png");
        let (_player, player_transform) = query.single_mut();

        let transform_a: Transform = player_transform
            .with_translation(player_transform.translation + Vec3::new(6.0, 20.0, 1.0));
        let transform_b: Transform = player_transform
            .with_translation(player_transform.translation + Vec3::new(-6.0, 20.0, 1.0));
        let movement_vector = Vec3::new(0.0, SHOT_SPEED, 0.0);

        commands.spawn((
            Shot,
            SimpleMovement {
                movement_vector: movement_vector,
            },
            SpriteBundle {
                texture: shot_handle.clone(),
                transform: transform_a,
                ..default()
            },
        ));

        commands.spawn((
            Shot,
            SimpleMovement {
                movement_vector: movement_vector,
            },
            SpriteBundle {
                texture: shot_handle.clone(),
                transform: transform_b,
                ..default()
            },
        ));

        shot_counter.value = 0;
    }
}

fn apply_movement_vector_system(mut query: Query<(&SimpleMovement, &mut Transform)>) {
    for (simple_movement, mut transform) in &mut query {
        transform.translation = transform.translation + simple_movement.movement_vector;
    }
}

fn outside_removal_system(
    mut commands: Commands,
    mut query: Query<(Entity, &SimpleMovement, &mut Transform)>,
) {
    let bx = (BOUNDS.x / 2.0) + 10.0;
    let by = (BOUNDS.y / 2.0) + 10.0;

    for (entity, _, transform) in &mut query {
        if f32::abs(transform.translation.y) > by {
            commands.entity(entity).despawn_recursive();
        } else if f32::abs(transform.translation.x) > bx {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ship_handle = asset_server.load("textures/v0idp/apu_apustaja_ship.png");
    let background_handle = asset_server.load("textures/v0idp/background2.png");

    commands.spawn(Camera2dBundle::default());

    commands.spawn(SpriteBundle {
        texture: background_handle,
        ..default()
    });

    let border_color = Color::hex(BORDER_COLOR_HEX).unwrap();
    let border_width = 40.0;

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(BOUNDS.x, border_width)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
        ..default()
    });

    commands.spawn((
        SpriteBundle {
            texture: ship_handle,
            ..default()
        },
        Player {
            movement_speed: 10.0,
        },
    ));
}

fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Player, &mut Transform)>,
) {
    let (ship, mut transform) = query.single_mut();

    let mut movement_factor_x = 0.0;
    let mut movement_factor_y = 0.0;

    if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
        movement_factor_x -= ship.movement_speed;
    }

    if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
        movement_factor_x += ship.movement_speed;
    }

    if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
        movement_factor_y += ship.movement_speed;
    }

    if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
        movement_factor_y -= ship.movement_speed;
    }

    let extents = Vec3::from((BOUNDS / 2.0, 0.0));
    transform.translation = transform.translation.min(extents).max(-extents);

    transform.translation = Vec3::new(
        transform.translation.x + movement_factor_x,
        transform.translation.y + movement_factor_y,
        transform.translation.z,
    )
}

fn snap_to_player_system(
    mut query: Query<&mut Transform, (With<DrillEnemy>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_transform = player_query.single();

    let player_translation = player_transform.translation.xy();

    for mut enemy_transform in &mut query {
        let to_player = (player_translation - enemy_transform.translation.xy()).normalize();

        let rotate_to_player = Quat::from_rotation_arc(Vec3::Y, to_player.extend(0.));

        enemy_transform.rotation = rotate_to_player;

        let movement_direction = enemy_transform.rotation * Vec3::Y;
        let movement_distance = ENEMY_SPEED * TIME_STEP;
        let translation_delta = movement_direction * movement_distance;
        enemy_transform.translation += translation_delta;
        let extents = Vec3::from((BOUNDS / 2.0, 0.0));
        enemy_transform.translation = enemy_transform.translation.min(extents).max(-extents);
    }
}
