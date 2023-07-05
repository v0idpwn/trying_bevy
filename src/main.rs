use bevy::{math::Vec3Swizzles, prelude::*, sprite::collide_aabb::collide};

const TIME_STEP: f32 = 1.0 / 60.0;
const MAX_X: f32 = 400.0;
const MAX_Y: f32 = 600.0;
const BOUNDS: Vec2 = Vec2::new(MAX_X, MAX_Y);
const SHOT_SPEED: f32 = 10.0;
const SHOT_INTERVAL: u8 = 4; // actual shot interval = (time_step / shot_interval)
const ENEMY_SPEED: f32 = 200.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_systems(
            (
                player_movement_system,
                snap_to_player_system,
                maybe_spawn_enemy_system,
                spawn_shots_system,
                shot_movement_system,
                check_for_collisions_system,
                shot_removal_system,
            )
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .insert_resource(ShotCounter { value: 0 })
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Resource)]
struct ShotCounter {
    value: u8,
}

#[derive(Component)]
struct Shot;

#[derive(Component)]
struct Player {
    movement_speed: f32,
}

#[derive(Component)]
struct RegularEnemy;

fn check_for_collisions_system(
    mut commands: Commands,
    player_query: Query<(&Player, &Transform)>,
    shot_query: Query<(Entity, &Shot, &Transform)>,
    enemies_query: Query<(Entity, &RegularEnemy, &Transform)>,
    focused_windows_query: Query<'_, '_, (Entity, &Window), ()>,
) {
    let shot_size_vec = Vec2::new(2., 4.);
    let enemy_size_vec = Vec2::new(40., 40.);
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
                    commands.entity(enemy_entity).despawn();
                    commands.entity(shot_entity).despawn();
                }
                None => {}
            }
        }
    }
}

fn maybe_spawn_enemy_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let enemy_a_handle = asset_server.load("textures/simplespace/enemy_A.png");

    let n = rand::random::<u8>();

    if n > 200 {
        // spawn at random x, top 10% y
        let x = (MAX_X / 2.0) - (rand::random::<f32>() * (MAX_X));
        let y = (MAX_Y - (MAX_Y / 5.0)) + (rand::random::<f32>() * MAX_Y / 5.0);

        commands.spawn((
            SpriteBundle {
                texture: enemy_a_handle.clone(),
                transform: Transform::from_xyz(x, y, 0.0),
                ..default()
            },
            RegularEnemy,
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

        commands.spawn((
            Shot,
            SpriteBundle {
                texture: shot_handle.clone(),
                transform: transform_a,
                ..default()
            },
        ));

        commands.spawn((
            Shot,
            SpriteBundle {
                texture: shot_handle.clone(),
                transform: transform_b,
                ..default()
            },
        ));

        shot_counter.value = 0;
    }
}

fn shot_movement_system(mut query: Query<(&Shot, &mut Transform)>) {
    // TODO: mut vec in place instead of creating new
    for (_, mut transform) in &mut query {
        transform.translation = Vec3::new(
            transform.translation.x,
            transform.translation.y + SHOT_SPEED,
            transform.translation.z,
        )
    }
}

fn shot_removal_system(mut commands: Commands, mut query: Query<(Entity, &Shot, &mut Transform)>) {
    for (shot_entity, _, transform) in &mut query {
        if transform.translation.y > MAX_Y / 2.0 {
            commands.entity(shot_entity).despawn();
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ship_handle = asset_server.load("textures/v0idp/apu_apustaja_ship.png");

    commands.spawn(Camera2dBundle::default());

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

    if keyboard_input.pressed(KeyCode::Left) {
        movement_factor_x -= ship.movement_speed;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        movement_factor_x += ship.movement_speed;
    }

    if keyboard_input.pressed(KeyCode::Up) {
        movement_factor_y += ship.movement_speed;
    }

    if keyboard_input.pressed(KeyCode::Down) {
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
    mut query: Query<&mut Transform, (With<RegularEnemy>, Without<Player>)>,
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
