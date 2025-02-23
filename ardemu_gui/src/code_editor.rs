use iced::{
	keyboard::{self, key},
	widget::text_editor::{Action, Binding, Content, Edit, KeyPress, Motion, Status},
};

pub fn code_editor_keybindings<Message>(
	key_press: KeyPress,
	unindent_messge: Message,
) -> Option<Binding<Message>> {
	let KeyPress {
		key,
		modifiers,
		status,
		..
	} = &key_press;

	if *status != Status::Focused {
		return None;
	}

	match key {
		keyboard::Key::Named(key::Named::Delete) => Some(if modifiers.command() {
			Binding::Sequence(vec![Binding::Select(Motion::WordRight), Binding::Delete])
		} else {
			Binding::Delete
		}),
		keyboard::Key::Named(key::Named::Tab) if !modifiers.command() => {
			Some(if modifiers.shift() {
				Binding::Custom(unindent_messge)
			} else {
				Binding::Sequence(vec![
					Binding::Insert(' '),
					Binding::Insert(' '),
					Binding::Insert(' '),
					Binding::Insert(' '),
				])
			})
		}
		keyboard::Key::Named(key::Named::Backspace) if modifiers.command() => {
			Some(Binding::Sequence(vec![
				Binding::<Message>::Select(Motion::WordLeft),
				Binding::Backspace,
			]))
		}
		_ => Binding::<Message>::from_key_press(key_press),
	}
}

pub fn unindent_text(text: &mut Content) {
	let (line, column) = text.cursor_position();
	let tab_column = text.line(line).and_then(|line| {
		line.split_at_checked(column)
			.and_then(|(line, _right_line)| {
				line.rfind("    ").and_then(|tab_column| {
					if tab_column <= column {
						Some(tab_column)
					} else {
						None
					}
				})
			})
	});

	if let Some(tab_column) = tab_column {
		let tab_end_column = tab_column as i32 + 4;
		let steps = column as i32 - tab_end_column;

		let move_steps = |steps: i32, text: &mut Content| {
			if steps > 0 {
				for _ in 0..steps {
					text.perform(Action::Move(Motion::Left));
				}
			} else {
				for _ in 0..(-steps) {
					text.perform(Action::Move(Motion::Right));
				}
			}
		};

		move_steps(steps, text);
		text.perform(Action::Edit(Edit::Backspace));
		text.perform(Action::Edit(Edit::Backspace));
		text.perform(Action::Edit(Edit::Backspace));
		text.perform(Action::Edit(Edit::Backspace));
		move_steps(-steps, text);
	}
}
