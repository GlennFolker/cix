use bevy::prelude::*;

use crate::CixSprites;

pub const CIX_ATTIRE_ALL: &'static [CixAttire] = &[
    CixAttire::RedCollar,
    CixAttire::BlueCape,
    CixAttire::PinkCollar,
    CixAttire::RedScarf,
    CixAttire::PinkScarf,
];

pub const CIX_ATTIRE_SIZE: Vec2 = Vec2::new(40., 60.);

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
            RedCollar => Vec2::new(2. / 64., 30. / 96.),
            BlueCape => Vec2::new(-12. / 64., 27. / 96.),
            PinkCollar => Vec2::new(5. / 64., 27. / 96.),
            RedScarf => Vec2::new(16. / 64., 22. / 96.),
            PinkScarf => Vec2::new(8. / 64., 23. / 96.),
        }
    }
}
