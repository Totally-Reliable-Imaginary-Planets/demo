use bevy::prelude::Component;
use bevy::prelude::Vec2;

#[derive(Component)]
pub struct Planet {
    name: String,
    position: Vec2,
}

impl Planet {
    pub fn new(name: &str, position: Vec2) -> Self {
        Self {
            name: name.to_string(),
            position,
        }
    }

    pub fn name(&self) -> String {
        String::from(&self.name)
    }

    pub fn position(&self) -> &Vec2 {
        &self.position
    }
}
