use iced::widget::svg;
use std::sync::LazyLock;

pub static ARDUINO_UNO_SVG: LazyLock<svg::Handle> =
	LazyLock::new(|| svg::Handle::from_memory(include_bytes!("../../assets/ArduinoUno.svg")));
