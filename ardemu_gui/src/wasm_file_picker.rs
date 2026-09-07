//! Browser (wasm) file picker for loading `.elf`/`.ihex` files.
//!
//! Browsers don't expose a filesystem path, so this opens a hidden
//! `<input type="file">` element and reads the chosen file into memory. It
//! then hands back an in-memory [`ProgramSource`] through the usual message
//! channel, mirroring how the native `rfd` dialog returns a path. If the user
//! cancels, the picker task simply stays pending, leaving the current program
//! source untouched (the same "no change" outcome the native dialog produces).

use crate::{Message, ProgramSource};
use js_sys::{ArrayBuffer, Uint8Array};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Blob, Event, File, FileList, HtmlInputElement};

/// The kind of file the picker accepts, which drives the `accept` filter and
/// how the picked contents are read back into memory.
#[derive(Debug, Clone, Copy)]
pub enum FileKind {
	Elf,
	IHex,
}

impl FileKind {
	fn accept(self) -> &'static str {
		match self {
			Self::Elf => ".elf,application/octet-stream",
			Self::IHex => ".hex,.ihex,text/plain",
		}
	}
}

/// Opens the browser picker for `kind` and returns a `Task` that resolves to a
/// [`Message`]. On success it compiles the picked in-memory source; on cancel
/// the task never fires, leaving the current source as-is.
pub fn pick_and_load_file(kind: FileKind, previous_source: ProgramSource) -> iced::Task<Message> {
	let future = async move {
		match picked_program_source(kind).await {
			Some(program_source) => Message::ChangeAndCompileProgramSource(program_source),
			None => Message::ChangeProgramSource(previous_source),
		}
	};

	iced::Task::perform(future, |message| message)
}

/// Runs the whole picker flow and returns the in-memory [`ProgramSource`], or
/// `None` if the user cancelled or the file could not be picked/read.
async fn picked_program_source(kind: FileKind) -> Option<ProgramSource> {
	let input = create_file_input(kind)?;

	let file = picked_file(&input).await?;

	let result = match kind {
		FileKind::Elf => read_as_bytes(&file).await.map(ProgramSource::ElfFile),
		FileKind::IHex => read_as_text(&file).await.map(ProgramSource::IHexFile),
	};

	input.remove();

	match result {
		Ok(program_source) => Some(program_source),
		Err(e) => {
			eprintln!("ardemu: {e}");
			None
		}
	}
}

/// Creates a hidden `<input type="file">`, appends it to the document so a
/// programmatic click opens the native picker, and returns it.
fn create_file_input(kind: FileKind) -> Option<HtmlInputElement> {
	let document = window()?.document()?;

	let input = document
		.create_element("input")
		.ok()?
		.dyn_into::<HtmlInputElement>()
		.ok()?;

	input.set_type("file");
	input.set_attribute("accept", kind.accept()).ok()?;
	input.set_attribute("style", "display:none").ok()?;

	document.body()?.append_child(input.as_ref()).ok()?;

	Some(input)
}

/// Opens the dialog, waits for the change event, and returns the chosen
/// [`File`], or `None` if the user cancelled.
async fn picked_file(input: &HtmlInputElement) -> Option<File> {
	let change = wait_for_change(input);

	input.click();

	change.await.ok()?;

	let files: Option<FileList> = input.files();
	files.and_then(|list| list.get(0))
}

/// Waits until the picker fires a `change` event (i.e. the user picked a file).
///
/// The browser dialog stays open until the user acts, so on cancel no `change`
/// event fires and this future stays pending forever (an inert no-op).
async fn wait_for_change(input: &HtmlInputElement) -> Result<(), JsValue> {
	let target = input.clone();
	JsFuture::from(js_sys::Promise::new(&mut |resolve, _reject| {
		let on_change = Closure::<dyn FnMut(Event)>::new(move |_: Event| {
			let _ = resolve.call0(&JsValue::UNDEFINED);
		});
		let _ =
			target.add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref());
		on_change.forget();
	}))
	.await
	.map(|_| ())
}

/// Reads a file's raw bytes (for `.elf`).
async fn read_as_bytes(file: &File) -> Result<Vec<u8>, String> {
	let buffer = read_array_buffer(file).await?;
	Ok(Uint8Array::new(&buffer).to_vec())
}

/// Reads a file's contents as UTF-8 text (for `.ihex`).
async fn read_as_text(file: &File) -> Result<String, String> {
	let blob = file.clone().unchecked_into::<Blob>();
	let value = JsFuture::from(Blob::text(&blob))
		.await
		.map_err(|e| format!("failed to read ihex file: {e:?}"))?;
	value
		.as_string()
		.ok_or_else(|| "ihex file is not valid UTF-8".to_string())
}

/// Reads a file into an in-memory [`ArrayBuffer`] (for `.elf`).
async fn read_array_buffer(file: &File) -> Result<ArrayBuffer, String> {
	let blob = file.clone().unchecked_into::<Blob>();
	let value = JsFuture::from(Blob::array_buffer(&blob))
		.await
		.map_err(|e| format!("failed to read elf file: {e:?}"))?;
	Ok(value.into())
}
