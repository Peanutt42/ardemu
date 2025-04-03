use iced::{font::Family, widget::svg, Font};
use std::sync::LazyLock;

pub static ARROW_RIGHT_SVG: LazyLock<svg::Handle> =
	LazyLock::new(|| svg::Handle::from_memory(include_bytes!("../assets/arrow-right-short.svg")));

pub static ARDUINO_UNO_SVG: LazyLock<svg::Handle> =
	LazyLock::new(|| svg::Handle::from_memory(include_bytes!("../assets/ArduinoUno.svg")));

pub static ARDUINO_UNO_LED_BUILTIN_ON_SVG: LazyLock<svg::Handle> = LazyLock::new(|| {
	svg::Handle::from_memory(include_bytes!("../assets/ArduinoUno_LED_BUILTIN_ON.svg"))
});

pub static ARDUINO_UNO_LED_POWER_ON_SVG: LazyLock<svg::Handle> = LazyLock::new(|| {
	svg::Handle::from_memory(include_bytes!("../assets/ArduinoUno_LED_POWER_ON.svg"))
});

pub static APP_ICON_PNG_BYTES: &[u8] = include_bytes!("../assets/icon.png");

pub static JETBRAINS_MONO_FONT: Font = Font {
	family: Family::Name("JetBrains Mono"),
	..Font::DEFAULT
};
pub static JETBRAINS_MONO_FONT_BYTES: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");
