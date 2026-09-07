#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(unsafe_code)]

#[cfg(not(target_arch = "wasm32"))]
use ardemu_gui::assets::APP_ICON_PNG_BYTES;
use ardemu_gui::assets::{JETBRAINS_MONO_FONT, JETBRAINS_MONO_FONT_BYTES};
use ardemu_gui::App;

#[cfg(all(not(target_arch = "wasm32"), target_os = "linux"))]
use iced::window::settings::PlatformSpecific;
#[cfg(not(target_arch = "wasm32"))]
use iced::window::{self, icon};

pub fn main() -> iced::Result {
	#[cfg(target_arch = "wasm32")]
	hide_loading_spinner();

	let app = iced::application(App::new, App::update, App::view)
		.title(App::title)
		.theme(App::theme)
		.subscription(App::subscription)
		.font(JETBRAINS_MONO_FONT_BYTES)
		.default_font(JETBRAINS_MONO_FONT);

	#[cfg(not(target_arch = "wasm32"))]
	let app = app.window(window::Settings {
		icon: icon::from_file_data(APP_ICON_PNG_BYTES, Some(image::ImageFormat::Png)).ok(),
		#[cfg(target_os = "linux")]
		platform_specific: PlatformSpecific {
			application_id: "ardemu".to_string(),
			..Default::default()
		},
		..Default::default()
	});

	app.run()
}

/// Removes the loading spinner overlay from the page once the wasm app is
/// about to start, so it doesn't sit on top of the rendered canvas.
#[cfg(target_arch = "wasm32")]
fn hide_loading_spinner() {
	use web_sys::window;

	let Some(window) = window() else {
		return;
	};
	let Some(document) = window.document() else {
		return;
	};
	let Some(loading) = document.get_element_by_id("loading") else {
		return;
	};
	loading.remove();
}
