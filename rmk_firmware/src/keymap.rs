use rmk::types::action::{EncoderAction, KeyAction};
use rmk::{a, encoder, k, layer};

pub(crate) const COL: usize = 15;
pub(crate) const ROW: usize = 2;
pub(crate) const NUM_LAYER: usize = 5;
pub(crate) const NUM_ENCODER: usize = 3;
pub(crate) const SCROLL_LAYER: u8 = 1;

// USER key IDs (0..=7 are reserved by rmk for BLE profile control).
// Not used by the default keymap — exposed via Vial for remapping.
pub(crate) const USER_CPI_UP: u8 = 8;
pub(crate) const USER_CPI_DOWN: u8 = 9;
pub(crate) const USER_CURSOR_TOGGLE: u8 = 10;
pub(crate) const USER_SNIPE: u8 = 11;
pub(crate) const USER_SCROLL_SPEED_UP: u8 = 12;
pub(crate) const USER_SCROLL_SPEED_DOWN: u8 = 13;
pub(crate) const USER_SCROLL_INVERT_X: u8 = 14;
pub(crate) const USER_SCROLL_INVERT_Y: u8 = 15;
pub(crate) const USER_SAVE_SLOT_1: u8 = 16;
pub(crate) const USER_LOAD_SLOT_1: u8 = 17;
pub(crate) const USER_SAVE_SLOT_2: u8 = 18;
pub(crate) const USER_LOAD_SLOT_2: u8 = 19;

const TRANSPARENT_LAYER: [[KeyAction; COL]; ROW] = layer!([
    [
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
        a!(Transparent), a!(Transparent), a!(Transparent)
    ],
    [
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
        a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
        a!(Transparent), a!(Transparent), a!(Transparent)
    ]
]);

const TRANSPARENT_ENCODERS: [EncoderAction; NUM_ENCODER] = [
    encoder!(a!(Transparent), a!(Transparent)),
    encoder!(a!(Transparent), a!(Transparent)),
    encoder!(a!(Transparent), a!(Transparent)),
];

#[rustfmt::skip]
pub const fn get_default_keymap() -> [[[KeyAction; COL]; ROW]; NUM_LAYER] {
    #[cfg(feature = "sensor-rotated-180")]
    {
        return [
            layer!([
                [
                    k!(A), k!(B), k!(MouseBtn1), k!(C),
                    k!(E), k!(G), k!(H), k!(I),
                    k!(MouseBtn2), k!(J), k!(K), k!(L),
                    k!(Kc5), k!(Kc6), k!(Kc7)
                ],
                [
                    k!(N), k!(M), k!(P), k!(O),
                    k!(D), k!(F), k!(S), k!(Q),
                    k!(R), k!(T), k!(V), k!(U),
                    k!(Kc0), k!(Kc9), k!(Kc8)
                ]
            ]),
            TRANSPARENT_LAYER,
            TRANSPARENT_LAYER,
            TRANSPARENT_LAYER,
            TRANSPARENT_LAYER,
        ];
    }

    #[cfg(not(feature = "sensor-rotated-180"))]
    {
    [
        layer!([
            [
                k!(MouseBtn2), k!(A), k!(B), k!(C),
                k!(E), k!(G), k!(MouseBtn1), k!(H),
                k!(I), k!(J), k!(K), k!(L),
                k!(Kc5), k!(Kc6), k!(Kc7)
            ],
            [
                k!(N), k!(M), k!(P), k!(O),
                k!(D), k!(F), k!(S), k!(Q),
                k!(R), k!(T), k!(V), k!(U),
                k!(Kc0), k!(Kc9), k!(Kc8)
            ]
        ]),
        TRANSPARENT_LAYER,
        TRANSPARENT_LAYER,
        TRANSPARENT_LAYER,
        TRANSPARENT_LAYER,
    ]
    }
}

pub const fn get_default_encoder_map() -> [[EncoderAction; NUM_ENCODER]; NUM_LAYER] {
    #[cfg(feature = "sensor-rotated-180")]
    {
        return [
            [
                encoder!(k!(W), k!(X)),                        // head
                encoder!(k!(Z), k!(Y)),                        // chest
                encoder!(k!(MouseWheelDown), k!(MouseWheelUp)), // leg: vertical wheel
            ],
            TRANSPARENT_ENCODERS,
            TRANSPARENT_ENCODERS,
            TRANSPARENT_ENCODERS,
            TRANSPARENT_ENCODERS,
        ];
    }

    #[cfg(not(feature = "sensor-rotated-180"))]
    {
        [
            [
                encoder!(k!(W), k!(X)),                        // head
                encoder!(k!(MouseWheelDown), k!(MouseWheelUp)), // chest: vertical wheel
                encoder!(k!(Y), k!(Z)),                        // leg
            ],
            TRANSPARENT_ENCODERS,
            TRANSPARENT_ENCODERS,
            TRANSPARENT_ENCODERS,
            TRANSPARENT_ENCODERS,
        ]
    }
}
