use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Entity;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::Single;
use bevy::prelude::Time;
use bevy::prelude::Transform;
use bevy::prelude::Vec2;
use bevy::prelude::Visibility;

//use crate::PlanetEntities;
//use crate::Planet;
use crate::Explorer;

/*pub fn explorer_movement_system(
    time: Res<Time>,
    mut explorer_query: Query<(&mut Transform, &mut Explorer)>,
    planet_query: Query<&Planet>,
    planet_entities: Res<PlanetEntities>,
) {
    for (mut transform, mut explorer) in explorer_query.iter_mut() {
        if let Some(target_entity) = explorer.target_planet {
            if let Ok(target_planet) = planet_query.get(target_entity) {
                let current_pos = Vec2::new(transform.translation.x, transform.translation.y);
                let direction = target_planet.position() - current_pos;
                let distance = direction.length();

                if distance > 5.0 {
                    // Still traveling
                    let movement =
                        direction.normalize() * explorer.travel_speed * time.delta_secs();
                    transform.translation.x += movement.x;
                    transform.translation.y += movement.y;
                } else {
                    // Arrived, switch to other planet
                    let other_planet = planet_entities
                        .planets
                        .iter()
                        .find(|&&p| p != target_entity)
                        .copied();
                    explorer.target_planet = other_planet;
                    println!("Explorer arrived at {}!", target_planet.name());
                }
            }
        }
    }
}*/

use bevy::prelude::ButtonInput;
use bevy::prelude::KeyCode;
use bevy::prelude::With;

use crate::Planet;
use crate::PlanetDialog;
use crate::explorer::Roaming;

#[derive(Component)]
pub(crate) struct ReachedPlanet(pub(crate) bool);

pub fn explorer_movement_system_wasd(
    time: Res<Time>,
    mut explorer_query: Query<&mut Transform, With<Explorer>>,
    _roaming: Single<Entity, With<Roaming>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    reached: Query<&ReachedPlanet>,
) {
    let mut transform = explorer_query.single_mut().unwrap();
    let speed = 150.0;
    let mut direction = Vec2::ZERO;

    //if keyboard_input.pressed(KeyCode::KeyW) { direction.y += 1.0; }
    //if keyboard_input.pressed(KeyCode::KeyS) { direction.y -= 1.0; }
    if keyboard_input.pressed(KeyCode::KeyA) {
        if reached.single().map_or(true, |r| r.0) {
            direction.x -= 1.0;
        }
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        if !reached.single().map_or(false, |r| r.0) {
            direction.x += 1.0;
        }
    }

    if direction.length() > 0.0 {
        let movement = direction.normalize() * speed * time.delta_secs();
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;
    }
}

pub fn check_explorer_reach(
    mut commands: Commands,
    explorer_transform: Single<&Transform, With<Explorer>>,
    planet_query: Query<&Transform, With<Planet>>,
    reached: Query<Entity, With<ReachedPlanet>>,
    mut dialog_query: Query<&mut Visibility, With<PlanetDialog>>,
) {
    let mut in_range = false;
    let mut is_left = false;

    for planet_transform in &planet_query {
        let distance = explorer_transform
            .translation
            .distance(planet_transform.translation);

        if distance < 50.0 {
            is_left = explorer_transform.translation.x < planet_transform.translation.x;
            in_range = true;
            break;
        }
    }

    if in_range && reached.is_empty() {
        println!("Explorer reached a planet!");
        for mut visibility in &mut dialog_query {
            *visibility = Visibility::Visible;
        }
        commands.spawn(ReachedPlanet(is_left));
    } else if !in_range {
        for mut visibility in &mut dialog_query {
            *visibility = Visibility::Hidden;
        }
        if let Ok(entity) = reached.single() {
            commands.entity(entity).despawn();
        }
    }
}
