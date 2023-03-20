use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::CixSprites;

pub const CIX_ATTIRE_ALL: &'static [CixAttire] = &[
    CixAttire::RedCollar,
    CixAttire::BlueCape,
    CixAttire::PinkCollar,
    CixAttire::RedScarf,
    CixAttire::PinkScarf,
];

#[derive(Component, Copy, Clone)]
pub enum CixAttire {
    RedCollar,
    BlueCape,
    PinkCollar,
    RedScarf,
    PinkScarf,
}

impl CixAttire {
    #[inline]
    pub fn sprite(self, sprites: &CixSprites) -> &Handle<Image> {
        use CixAttire::*;
        match self {
            RedCollar => &sprites.red_collar,
            BlueCape => &sprites.blue_cape,
            PinkCollar => &sprites.pink_collar,
            RedScarf => &sprites.red_scarf,
            PinkScarf => &sprites.pink_scarf,
        }
    }

    #[inline]
    pub fn anchor(self) -> Vec2 {
        use CixAttire::*;
        match self {
            RedCollar => Vec2::new(2., 30.),
            BlueCape => Vec2::new(-12., 27.),
            PinkCollar => Vec2::new(5., 27.),
            RedScarf => Vec2::new(16., 22.),
            PinkScarf => Vec2::new(8., 23.),
        }
    }

    #[inline]
    pub fn joint_angle(self) -> f32 {
        use CixAttire::*;
        (match self {
            RedCollar => 12.,
            BlueCape => 30.,
            PinkCollar => 12.,
            RedScarf => 30.,
            PinkScarf => 30.,
        } as f32).to_radians()
    }

    #[inline]
    pub fn collider(self) -> ((f32, f32), Collider) {
        use CixAttire::*;
        match self {
            RedCollar => ((0., 30.), Collider::cuboid(54. / 2., 16. / 2.)),
            BlueCape => ((-15., 11.), Collider::cuboid(8. / 2., 26. / 2.)),
            PinkCollar => ((5., 27.), Collider::cuboid(33. / 2., 13. / 2.)),
            RedScarf => ((18., -8.), Collider::cuboid(8. / 2., 55. / 2.)),
            PinkScarf => ((10., 6.), Collider::cuboid(8. / 2., 36. / 2.)),
        }
    }
}
