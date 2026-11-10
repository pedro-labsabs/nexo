//! Definições de tema e tokens visuais para o NEXO.

use rune::Color;
use rune::tok;

pub struct NexoTheme {
    pub bg: Color,
    pub card: Color,
    pub card_border: Color,
    pub text: Color,
    pub text_secondary: Color,
    pub control_off: Color,
    pub accent: Color,
    pub destructive: Color,
}

pub fn get_theme(dark: bool) -> NexoTheme {
    let t = tok::theme(dark);
    NexoTheme {
        bg: t.bg,
        card: t.card,
        card_border: t.card_border,
        text: t.text,
        text_secondary: t.text_secondary,
        control_off: t.control_off,
        accent: tok::accent(0, dark),
        destructive: tok::accent(2, dark), // Accent 2 is red/destructive hue
    }
}
