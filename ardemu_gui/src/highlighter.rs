//! COPIED FROM: https://github.com/pml68/iced_custom_highlighter/blob/fix/temp-update-on-theme-changed/src/lib.rs
//! MODIFIED TO FIT OUR NEED FOR CUSTOM AVR ASSEMBLY SYNTAX HIGHLIGHTING
//!
//! A custom syntax highlighter for iced.
//!
//! It uses the colors from your app's Theme, based on a styling method (like [`default_style`])
//!
//! # Example
//!
//! ```no_run
//! use iced::widget::{Column, pick_list, text_editor};
//! use iced::{Element, Theme};
//! use iced_custom_highlighter::{Highlight, Highlighter, Settings};
//!
//! #[derive(Default)]
//! struct State {
//!     content: text_editor::Content,
//!     theme: Theme,
//! }
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     Edit(text_editor::Action),
//!     ChangeTheme(Theme),
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//! Column::new()
//!     .push(
//!         text_editor(&state.content)
//!             .placeholder("Type something here...")
//!             .highlight_with::<Highlighter>(
//!                 Settings::new(vec![], Highlight::default_style, state.theme.clone(), "rs"),
//!                 Highlight::to_format,
//!             )
//!             .on_action(Message::Edit),
//!     )
//!     .push(pick_list(
//!         Theme::ALL,
//!         Some(state.theme.clone()),
//!         Message::ChangeTheme,
//!     ))
//!     .into()
//! }
//!
//! fn update(state: &mut State, message: Message) {
//!     match message {
//!         Message::Edit(action) => {
//!             state.content.perform(action);
//!         }
//!
//!         Message::ChangeTheme(theme) => {
//!             state.theme = theme;
//!         }
//!     }
//! }
//! ```
//!
//! [`default_style`]: crate::Highlight::default_style
use iced::advanced::text::highlighter::{self, Format};
use iced::font::Font;
use iced::{Color, Theme};

use std::ops::Range;
use std::str::FromStr;
use std::sync::LazyLock;
use syntect::highlighting;
use syntect::parsing::{self, SyntaxDefinition, SyntaxReference, SyntaxSet, SyntaxSetBuilder};

struct AsmSyntax {
	syntax_set: SyntaxSet,
	syntax_reference: SyntaxReference,
}

static ASM_SYNTAX: LazyLock<AsmSyntax> = LazyLock::new(|| {
	let mut builder = SyntaxSetBuilder::new();
	let definition = SyntaxDefinition::load_from_str(
		include_str!("assembly.sublime-syntax"),
		false,
		Some("asm"),
	);
	match definition {
		Ok(definition) => builder.add(definition),
		Err(_) => builder.add_plain_text_syntax(),
	}
	let syntax_set = builder.build();
	let syntax_reference = syntax_set.syntaxes()[0].clone();
	AsmSyntax {
		syntax_set,
		syntax_reference,
	}
});

const LINES_PER_SNAPSHOT: usize = 50;

type ScopeSelectorsResult =
	std::result::Result<highlighting::ScopeSelectors, parsing::ParseScopeError>;

/// A syntax highlighter.
#[derive(Debug)]
pub struct Highlighter {
	syntax: &'static parsing::SyntaxReference,
	caches: Vec<(parsing::ParseState, parsing::ScopeStack)>,
	current_line: usize,
}

impl highlighter::Highlighter for Highlighter {
	type Settings = Settings;
	type Highlight = Highlight;

	type Iterator<'a> = Box<dyn Iterator<Item = (Range<usize>, Self::Highlight)> + 'a>;

	fn new(_settings: &Self::Settings) -> Self {
		let parser = parsing::ParseState::new(&ASM_SYNTAX.syntax_reference);
		let stack = parsing::ScopeStack::new();

		Highlighter {
			syntax: &ASM_SYNTAX.syntax_reference,
			caches: vec![(parser, stack)],
			current_line: 0,
		}
	}

	fn update(&mut self, _new_settings: &Self::Settings) {
		// Restart the highlighter
		self.change_line(0);
	}

	fn change_line(&mut self, line: usize) {
		let snapshot = line / LINES_PER_SNAPSHOT;

		if snapshot <= self.caches.len() {
			self.caches.truncate(snapshot);
			self.current_line = snapshot * LINES_PER_SNAPSHOT;
		} else {
			self.caches.truncate(1);
			self.current_line = 0;
		}

		let (parser, stack) = self.caches.last().cloned().unwrap_or_else(|| {
			(
				parsing::ParseState::new(self.syntax),
				parsing::ScopeStack::new(),
			)
		});

		self.caches.push((parser, stack));
	}

	fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
		if self.current_line / LINES_PER_SNAPSHOT >= self.caches.len() {
			let (parser, stack) = self.caches.last().expect("Caches must not be empty");

			self.caches.push((parser.clone(), stack.clone()));
		}

		self.current_line += 1;

		let (parser, stack) = self.caches.last_mut().expect("Caches must not be empty");

		let ops = parser
			.parse_line(line, &ASM_SYNTAX.syntax_set)
			.unwrap_or_default();

		Box::new(
			ScopeRangeIterator {
				ops,
				line_length: line.len(),
				index: 0,
				last_str_index: 0,
			}
			.filter_map(move |(range, scope)| {
				let _ = stack.apply(&scope);

				if range.is_empty() {
					None
				} else {
					Some((
						range,
						Highlight {
							scope: Scope::from_scopestack(stack, &[]),
						},
					))
				}
			}),
		)
	}

	fn current_line(&self) -> usize {
		self.current_line
	}
}

/// The settings of a [`Highlighter`].
#[derive(Debug, Clone, PartialEq)]
pub struct Settings;

/// A highlight produced by a [`Highlighter`].
#[derive(Debug)]
pub struct Highlight {
	scope: Scope,
}

impl Highlight {
	/// Returns the [`Format`] of the [`Highlight`].
	///
	/// [`Format`]: iced_widget::core::text::highlighter::Format
	pub fn to_format(&self, _theme: &Theme) -> Format<Font> {
		let color = match self.scope {
			Scope::Comment | Scope::TagStart => Some(Color::from_rgb8(139, 148, 158)),
			Scope::String | Scope::RegExp | Scope::QuotedString | Scope::QuotedSingle => {
				Some(Color::from_rgb8(165, 214, 255))
			}
			Scope::EscapeSequence
			| Scope::Exception
			| Scope::SupportConstruct
			| Scope::Continuation => Some(Color::from_rgb8(255, 123, 114)),
			Scope::Number => Some(Color::from_rgb8(121, 192, 255)),
			Scope::Variable
			| Scope::VariableStart
			| Scope::TagName
			| Scope::Import
			| Scope::Brackets => Some(Color::from_rgb8(255, 166, 87)),
			Scope::Keyword
			| Scope::KeywordOperator
			| Scope::Operator
			| Scope::Parantheses
			| Scope::Braces => Some(Color::from_rgb8(255, 123, 114)),
			Scope::Storage
			| Scope::StorageModifier
			| Scope::StorageType
			| Scope::Class
			| Scope::LibraryClass
			| Scope::VariableFunction
			| Scope::FunctionName
			| Scope::LibraryFunction => Some(Color::from_rgb8(210, 168, 255)),
			Scope::BuiltinConstant | Scope::UserDefinedConstant => {
				Some(Color::from_rgb8(121, 192, 255))
			}
			Scope::Invalid => Some(Color::from_rgb(1.0, 0.0, 0.0)),
			Scope::Special | Scope::KeywordOther => Some(Color::from_rgb8(165, 214, 255)),
			Scope::Other | Scope::Custom { .. } => None,
		};

		Format { color, font: None }
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub enum Scope {
	Comment,
	String,
	RegExp,
	EscapeSequence,
	Number,
	Variable,
	VariableFunction,
	Keyword,
	KeywordOperator,
	KeywordOther,
	Import,
	Operator,
	Storage,
	StorageModifier,
	Class,
	LibraryClass,
	FunctionName,
	VariableStart,
	BuiltinConstant,
	UserDefinedConstant,
	SupportConstruct,
	TagName,
	TagStart,
	LibraryFunction,
	Continuation,
	StorageType,
	Exception,
	Special,
	Invalid,
	QuotedString,
	QuotedSingle,
	Brackets,
	Parantheses,
	Braces,
	#[default]
	Other,
	/// A custom scope.
	Custom {
		/// The name of the scope, letting you identify it in match statements easily.
		name: String,
		/// A series of selectors separated by commas or pipes.
		scope_string: String,
	},
}

impl Scope {
	/// A list with all the defined scopes.
	pub const ALL: &'static [Self] = &[
		Self::Comment,
		Self::String,
		Self::RegExp,
		Self::EscapeSequence,
		Self::Number,
		Self::Variable,
		Self::VariableFunction,
		Self::Keyword,
		Self::KeywordOperator,
		Self::KeywordOther,
		Self::Import,
		Self::Operator,
		Self::Storage,
		Self::StorageModifier,
		Self::Class,
		Self::LibraryClass,
		Self::FunctionName,
		Self::VariableStart,
		Self::BuiltinConstant,
		Self::UserDefinedConstant,
		Self::SupportConstruct,
		Self::TagName,
		Self::TagStart,
		Self::LibraryFunction,
		Self::Continuation,
		Self::StorageType,
		Self::Exception,
		Self::Special,
		Self::Invalid,
		Self::QuotedString,
		Self::QuotedSingle,
		Self::Brackets,
		Self::Parantheses,
		Self::Braces,
	];

	/// Creates a new custom [`Scope`].
	pub fn custom(name: impl Into<String>, scope_string: impl Into<String>) -> Self {
		Self::Custom {
			name: name.into(),
			scope_string: scope_string.into(),
		}
	}

	/// Retuns the scope string of the [`Scope`].
	#[must_use]
	pub fn scope_str(&self) -> &str {
		match self {
            Self::Comment => "comment, meta.documentation",
            Self::String => "string",
            Self::RegExp => "string.regexp",
            Self::EscapeSequence => "constant.character.escape",
            Self::Number => "constant.numeric",
            Self::Variable => "variable",
            Self::VariableFunction => "variable.function",
            Self::Keyword => "keyword",
            Self::KeywordOperator => "keyword.operator",
            Self::KeywordOther => "keyword.other",
            Self::Import => "meta.import keyword, keyword.control.import, keyword.control.import.from, keyword.other.import, keyword.control.at-rule.include, keyword.control.at-rule.import",
            Self::Operator => "keyword.operator.comparison, keyword.operator.assignment, keyword.operator.arithmetic",
            Self::Storage => "storage",
            Self::StorageModifier => "storage.modifier",
            Self::Class => "keyword.control.class, meta.class, entity.name.class, entity.name.type.class",
            Self::LibraryClass => "support, support.type, support.class",
            Self::FunctionName => "entity.name.function",
            Self::VariableStart => "punctuation.definition.variable",
            Self::BuiltinConstant => "constant.language, meta.preprocessor",
            Self::UserDefinedConstant => "constant.character, constant.other",
            Self::SupportConstruct => "support.function.construct, keyword.other.new",
            Self::TagName => "entity.name.tag",
            Self::TagStart => "punctuation.definition.tag.html, punctuation.definition.tag.begin, punctuation.definition.tag.end",
            Self::LibraryFunction => "support.function",
            Self::Continuation => "punctuation.separator.continuation",
            Self::StorageType => "storage.type",
            Self::Exception => "support.type.exception",
            Self::Special => "keyword.other.special-method",
            Self::Invalid => "invalid",
            Self::QuotedString => "string.quoted.double, string.quoted.single",
            Self::QuotedSingle => "punctuation.definition.string.begin, punctuation.definition.string.end",
            Self::Brackets => "meta.brace.squares",
            Self::Parantheses => "meta.brace.round, punctuation.definition.parameters.begin, punctuation.definition.parameters.end",
            Self::Braces => "meta.brace.curly",
            Self::Other => "",
            Self::Custom {scope_string,..} => scope_string
        }
	}

	fn from_scopestack(stack: &parsing::ScopeStack, custom_scopes: &[Self]) -> Self {
		let scopes: Vec<Self> = if custom_scopes.is_empty() {
			Self::ALL.to_vec()
		} else {
			let mut all = Self::ALL.to_vec();
			all.extend_from_slice(custom_scopes);
			all
		};

		let selectors: Vec<(Self, highlighting::ScopeSelectors)> = scopes
			.iter()
			.filter_map(|scope| {
				let selector: ScopeSelectorsResult = scope.clone().into();
				match selector {
					Ok(selector) => Some((scope.clone(), selector)),
					Err(_) => None,
				}
			})
			.collect();

		let mut matching_scopes: Vec<(parsing::MatchPower, Self)> = selectors
			.iter()
			.filter_map(|(scope, selector)| {
				selector
					.does_match(&stack.scopes)
					.map(|score| (score, scope.clone()))
			})
			.collect();
		matching_scopes.sort_by_key(|&(score, _)| score);
		match matching_scopes.last() {
			Some(scope) => scope.1.clone(),
			None => Self::Other,
		}
	}
}

impl From<Scope> for ScopeSelectorsResult {
	fn from(value: Scope) -> Self {
		highlighting::ScopeSelectors::from_str(value.scope_str())
	}
}

struct ScopeRangeIterator {
	ops: Vec<(usize, parsing::ScopeStackOp)>,
	line_length: usize,
	index: usize,
	last_str_index: usize,
}

impl Iterator for ScopeRangeIterator {
	type Item = (std::ops::Range<usize>, parsing::ScopeStackOp);

	fn next(&mut self) -> Option<Self::Item> {
		if self.index > self.ops.len() {
			return None;
		}

		let next_str_i = if self.index == self.ops.len() {
			self.line_length
		} else {
			self.ops[self.index].0
		};

		let range = self.last_str_index..next_str_i;
		self.last_str_index = next_str_i;

		let op = if self.index == 0 {
			parsing::ScopeStackOp::Noop
		} else {
			self.ops[self.index - 1].1.clone()
		};

		self.index += 1;
		Some((range, op))
	}
}
