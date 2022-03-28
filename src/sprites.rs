use crate::{cfg, palette};
use macroquad::{color, math};

#[rustfmt::skip]
pub const SHIP: &[(&[math::Vec2], color::Color)] = {
    const S: f32 = cfg::SHIP_DRAW_RADIUS;
    &[
        (
            &[
                math::const_vec2!([-2.5 / S, -1.0 / S]),
                math::const_vec2!([-1.0 / S, -2.5 / S]),
                math::const_vec2!([ 1.0 / S, -2.5 / S]),
                math::const_vec2!([ 2.5 / S, -1.0 / S]),
                math::const_vec2!([ 2.5 / S,  1.0 / S]),
                math::const_vec2!([ 1.0 / S,  2.5 / S]),
                math::const_vec2!([-1.0 / S,  2.5 / S]),
                math::const_vec2!([-2.5 / S,  1.0 / S]),
            ],
            palette::BLUE,
        ),
        // (
        //     &[
        //         math::const_vec2!([-1.5 / S, -0.5 / S]),
        //         math::const_vec2!([-0.5 / S, -1.5 / S]),
        //         math::const_vec2!([ 0.5 / S, -1.5 / S]),
        //         math::const_vec2!([ 1.5 / S, -0.5 / S]),
        //         math::const_vec2!([ 1.5 / S,  0.5 / S]),
        //         math::const_vec2!([ 0.5 / S,  1.5 / S]),
        //         math::const_vec2!([-0.5 / S,  1.5 / S]),
        //         math::const_vec2!([-1.5 / S,  0.5 / S]),
        //     ],
        //     palette::BEIGE,
        // ),
        (
            &[
                math::const_vec2!([ 2.5 / S, -0.8 / S]),
                // math::const_vec2!([ 3.7 / S,  0.0 / S]),
                math::const_vec2!([ 3.55/ S, -0.5 / S]),
                math::const_vec2!([ 3.55/ S,  0.5 / S]),
                math::const_vec2!([ 2.5 / S,  0.8 / S]),
            ],
            palette::BLUE,
        ),
        (
            &[
                math::const_vec2!([  0.5 / S, -2.5 / S]),
                math::const_vec2!([ -2.0 / S, -1.5 / S]),
                math::const_vec2!([ -1.5 / S, -3.5 / S]),
                math::const_vec2!([  0.5 / S, -3.5 / S]),
            ],
            palette::LIGHTGRAY,
        ),
        (
            &[
                math::const_vec2!([  0.5 / S,  2.5 / S]),
                math::const_vec2!([ -2.0 / S,  1.5 / S]),
                math::const_vec2!([ -1.5 / S,  3.5 / S]),
                math::const_vec2!([  0.5 / S,  3.5 / S]),
            ],
            palette::LIGHTGRAY,
        ),
        (
            &[
                math::const_vec2!([  1.5 / S,  0.4 / S]),
                math::const_vec2!([ -0.5 / S,  0.3 / S]),
                math::const_vec2!([ -0.5 / S, -0.3 / S]),
                math::const_vec2!([  1.5 / S, -0.4 / S]),
            ],
            palette::LIGHTGRAY,
        ),
    ]
};

#[rustfmt::skip]
pub const ALIEN: &[(&[math::Vec2], color::Color)] = {
    const S: f32 = cfg::ALIEN_DRAW_RADIUS_BY_KIND[0];
    &[
        (
            &[
                math::const_vec2!([ 0.0 / S,  4.5 / S]),
                math::const_vec2!([-4.5 / S,  0.0 / S]),
                math::const_vec2!([ 0.0 / S, -4.5 / S]),
                math::const_vec2!([ 4.5 / S,  0.0 / S]),
            ],
            palette::DARKGREEN,
        ),
        (
            &[
                math::const_vec2!([ 0.0 / S,  3.5 / S]),
                math::const_vec2!([-3.5 / S,  0.0 / S]),
                math::const_vec2!([ 0.0 / S, -3.5 / S]),
                math::const_vec2!([ 3.5 / S,  0.0 / S]),
            ],
            palette::GREEN,
        ),
        (
            &[
                math::const_vec2!([ 1.8 / S,  1.2 / S]),
                math::const_vec2!([-1.8 / S,  1.2 / S]),
                math::const_vec2!([-1.8 / S, -1.2 / S]),
                math::const_vec2!([ 1.8 / S, -1.2 / S]),
            ],
            palette::WHITE,
        ),
        (
            &[
                math::const_vec2!([ 0.5 / S,  1.2 / S]),
                math::const_vec2!([-0.5 / S,  1.2 / S]),
                math::const_vec2!([-0.5 / S, -1.2 / S]),
                math::const_vec2!([ 0.5 / S, -1.2 / S]),
            ],
            palette::BLACK,
        ),
    ]
};
