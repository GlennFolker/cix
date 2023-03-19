use bevy::prelude::*;

pub trait ColorExt {
    fn lerp(self, dest: Self, f: f32) -> Self;
}

impl ColorExt for Color {
    #[inline]
    fn lerp(self, dest: Self, f: f32) -> Self {
        let [sr, sg, sb, sa] = self.as_linear_rgba_f32();
        let [dr, dg, db, da] = dest.as_linear_rgba_f32();
        Self::rgba_linear(
            sr + (dr - sr) * f,
            sg + (dg - sg) * f,
            sb + (db - sb) * f,
            sa + (da - sa) * f,
        )
    }
}
