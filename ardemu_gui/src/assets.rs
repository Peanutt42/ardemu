use iced::widget::svg;
use std::sync::LazyLock;

pub static ARDUINO_UNO_SVG: LazyLock<svg::Handle> =
	LazyLock::new(|| svg::Handle::from_memory(include_bytes!("../assets/ArduinoUno.svg")));

pub static ARDUINO_UNO_LED_BUILTIN_ON_SVG: LazyLock<svg::Handle> = LazyLock::new(|| {
	svg::Handle::from_memory(include_bytes!("../assets/ArduinoUno_LED_BUILTIN_ON.svg"))
});

pub static ARDUINO_UNO_LED_POWER_ON_SVG: LazyLock<svg::Handle> = LazyLock::new(|| {
	svg::Handle::from_memory(include_bytes!("../assets/ArduinoUno_LED_POWER_ON.svg"))
});
