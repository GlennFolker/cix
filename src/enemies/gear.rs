use bevy::{
    prelude::*,
    utils::HashMap,
};

#[derive(Resource, Default, Deref, DerefMut)]
pub struct EnemyGears(pub HashMap<String, Entity>);

#[derive(Component)]
pub struct EnemyGear {
    pub diameter: f32,
    pub link: Option<Entity>,
    pub link_iid: Option<String>,
}

pub fn enemy_gear_init_sys(gears: Res<EnemyGears>, mut added: Query<&mut EnemyGear, Added<EnemyGear>>) {
    for mut gear in &mut added {
        if let Some(ref iid) = gear.link_iid {
            gear.link = gears.get(iid).copied();
            println!("{:?}", gear.link);
        }
    }
}
