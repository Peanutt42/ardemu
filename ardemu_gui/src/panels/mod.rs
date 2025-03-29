mod instructions_panel;
pub use instructions_panel::{InstructionsPanel, InstructionsPanelMessage};

mod memory_panel;
pub use memory_panel::{MemoryPanel, MemoryPanelMessage};

mod arduino_board_panel;
pub use arduino_board_panel::ArduinoBoardPanel;

mod registers_panel;
pub use registers_panel::RegistersPanel;

mod flags_panel;
pub use flags_panel::FlagsPanel;
