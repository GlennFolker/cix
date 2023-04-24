use bevy::prelude::*;
use num_traits::{
    NumAssign,
    float::Float,
};

pub trait AngleExt: Float + NumAssign + Copy + PartialOrd<Self> {
    const PI: Self;

    #[inline]
    fn angle_wrap(self) -> Self {
        let pi2 = Self::PI * Self::from(2.).unwrap();
        (self % pi2 + pi2) % pi2
    }

    #[inline]
    fn angle_forward_dst(self, other: Self) -> Self {
        (self - other).abs()
    }

    #[inline]
    fn angle_backward_dst(self, other: Self) -> Self {
        Self::PI * Self::from(2.).unwrap() - (self - other).abs()
    }

    #[inline]
    fn angle_dist(self, other: Self) -> Self {
        let a = self.angle_wrap();
        let b = other.angle_wrap();

        let pi2 = Self::PI * Self::from(2.).unwrap();
        let zero = Self::from(0.).unwrap();
        (if (a - b) < zero { a - b + pi2 } else { a - b }).min(if (b - a) < zero { b - a + pi2 } else { b - a })
    }

    #[inline]
    fn angle_move_toward(self, to: Self, speed: Self) -> Self {
        if self.angle_dist(to).abs() < speed {
            return to;
        }

        let angle = self.angle_wrap();
        let to = to.angle_wrap();

        if (angle > to) == (angle.angle_backward_dst(to) > angle.angle_forward_dst(to)) {
            angle - speed
        } else {
            angle + speed
        }
    }

    #[inline]
    fn angle_clamp_range(self, dest: Self, range: Self) -> Self {
        let dst = self.angle_dist(dest);
        if dst <= range { self } else { self.angle_move_toward(dest, dst - range) }
    }

    fn angle_dist_avoid(self, to: Self, wall: Self) -> Self {
        // Define the size of a full circle.
        let pi2 = Self::PI * Self::from(2.).unwrap();

        // Boundary clamping.
        let from = (self % pi2 + pi2) % pi2;
        let to = (to % pi2 + pi2) % pi2;
        let wall = (wall % pi2 + pi2) % pi2;
    
        // Calculate the distance between the two angles in both clockwise and counter-clockwise directions.
        let clockwise_dist = ((to - from) + pi2) % pi2;
        let counter_clockwise_dist = ((from - to) + pi2) % pi2;

        // Calculate the signed distance between the two angles and the shortest path between them.
        let mut signed_dist = clockwise_dist - counter_clockwise_dist;
        let mut shortest_path = clockwise_dist;

        // Check if the wall angle is equal to either the 'from' or 'to' angles.
        if wall == from || wall == to {
            // If it is, return the shortest distance to the wall.
            return if counter_clockwise_dist < clockwise_dist {
                counter_clockwise_dist
            } else {
                clockwise_dist * Self::from(-1.).unwrap()
            };
        }

        // If the clockwise distance is greater than the counter-clockwise distance, the signed distance needs
        // to be flipped to be negative and the shortest path needs to be set to the counter-clockwise distance.
        if clockwise_dist > counter_clockwise_dist {
            signed_dist *= Self::from(-1.0).unwrap();
            shortest_path = counter_clockwise_dist;
        }

        // Calculate the distance between the wall angle and the 'from' and 'to' angles.
        let diff1 = ((wall - from) + pi2) % pi2;
        let diff2 = ((wall - to) + pi2) % pi2;

        // Check if the wall angle is closer to the 'to' angle than the shortest path between 'from' and 'to'.
        if diff1 < clockwise_dist && diff2 < clockwise_dist {
            // If it is, check if it's also closer to 'from' than the shortest path.
            if diff1 > shortest_path && diff2 > shortest_path {
                // If it is, return the negative of the shortest path.
                return shortest_path * Self::from(-1.).unwrap();
            } else {
                // If it's not, return the signed distance.
                return signed_dist;
            }
        }

        // Check if the wall angle is closer to the 'from' angle than the counter-clockwise distance.
        if diff1 > counter_clockwise_dist && diff2 > counter_clockwise_dist {
            // If it is, check if it's also closer to 'to' than the shortest path.
            if diff1 < shortest_path && diff2 < shortest_path {
                // If it is, return the signed distance.
                return signed_dist;
            } else {
                // If it's not, return the negative of the shortest path.
                return shortest_path * Self::from(-1.).unwrap();
            }
        }

        // If the wall angle is between the 'from' and 'to' angles, return the distance to the closer angle.
        if diff1 < diff2 {
            counter_clockwise_dist * Self::from(-1.).unwrap()
        } else {
            clockwise_dist
        }
    }
}

impl AngleExt for f32 {
    const PI: Self = std::f32::consts::PI;
}

impl AngleExt for f64 {
    const PI: Self = std::f64::consts::PI;
}

pub trait LerpExt {
    fn lerp(self, dest: Self, f: f32) -> Self;
}

impl LerpExt for Color {
    #[inline]
    fn lerp(self, dest: Self, f: f32) -> Self {
        let [sr, sg, sb, sa] = self.as_linear_rgba_f32();
        let [dr, dg, db, da] = dest.as_linear_rgba_f32();
        Self::rgba_linear(
            sr + (dr - sr) * f,
            sg + (dg - sg) * f,
            sb + (db - sb) * f,
            sa + (da - sa) * f,
        ).as_rgba()
    }
}

impl LerpExt for f32 {
    #[inline]
    fn lerp(self, dest: Self, f: f32) -> Self {
        self + (dest - self) * f
    }
}

impl LerpExt for Vec2 {
    #[inline]
    fn lerp(self, dest: Self, f: f32) -> Self {
        self + (dest - self) * f
    }
}
