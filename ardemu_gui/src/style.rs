use iced::{
	border::rounded,
	widget::{button, container, text, text_editor},
	Border, Color, Theme,
};

pub fn text_editor_style(theme: &Theme, status: text_editor::Status) -> text_editor::Style {
	let palette = theme.extended_palette();

	let active = text_editor::Style {
		background: Color::from_rgb8(1, 4, 9).into(),
		border: Border {
			radius: 8.0.into(),
			width: 0.0,
			color: palette.background.strong.color,
		},
		icon: palette.background.weak.text,
		placeholder: palette.background.strong.color,
		value: palette.background.base.text,
		selection: palette.primary.weak.color,
	};

	match status {
		text_editor::Status::Active => active,
		text_editor::Status::Hovered => text_editor::Style {
			border: Border {
				color: palette.background.base.text,
				..active.border
			},
			..active
		},
		text_editor::Status::Focused => text_editor::Style {
			border: Border {
				color: palette.primary.strong.color,
				..active.border
			},
			..active
		},
		text_editor::Status::Disabled => text_editor::Style {
			background: palette.background.weak.color.into(),
			value: active.placeholder,
			..active
		},
	}
}

pub fn button_style(theme: &Theme, status: button::Status) -> button::Style {
	let color_pair = match status {
		button::Status::Active => theme.extended_palette().primary.strong,
		button::Status::Hovered | button::Status::Pressed | button::Status::Disabled => {
			theme.extended_palette().primary.weak
		}
	};

	button::Style {
		background: Some(color_pair.color.into()),
		text_color: color_pair.text,
		border: rounded(8),
		..Default::default()
	}
}

pub fn hidden_secondary_button_style(theme: &Theme, status: button::Status) -> button::Style {
	match status {
		button::Status::Active | button::Status::Disabled => button::Style {
			background: None,
			text_color: Color::WHITE,
			..button::secondary(theme, status)
		},
		button::Status::Hovered | button::Status::Pressed => button::secondary(theme, status),
	}
}

pub fn panel_style(_theme: &Theme) -> container::Style {
	container::Style {
		background: Some(Color::from_rgb8(1, 4, 9).into()),
		text_color: Some(Color::WHITE),
		border: rounded(8),
		..Default::default()
	}
}

pub fn background_style(_theme: &Theme) -> container::Style {
	container::Style {
		background: Some(Color::from_rgb8(5, 9, 21).into()),
		text_color: Some(Color::WHITE),
		..Default::default()
	}
}

/// formats 1000000 into "1.000.000"
#[allow(clippy::unwrap_used)] // string -> int parsing with string already generated from valid number
pub fn format_big_number(n: usize) -> String {
	n.to_string()
		.as_bytes()
		.rchunks(3)
		.rev()
		.map(std::str::from_utf8)
		.collect::<Result<Vec<&str>, _>>()
		.unwrap()
		.join(",")
}

pub fn secondary_text_style(_theme: &Theme) -> text::Style {
	text::Style {
		color: Some(Color::from_rgb(0.85, 0.85, 0.85)),
	}
}
