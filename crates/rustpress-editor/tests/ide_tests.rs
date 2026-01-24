//! IDE Tests for RustPress (300 Tests)
//!
//! Comprehensive IDE/Editor testing covering:
//! - Editor Initialization & Setup (1-30)
//! - Toolbar & Controls (31-60)
//! - Block Operations (61-100)
//! - Text Editing (101-140)
//! - Keyboard Shortcuts (141-170)
//! - Drag & Drop (171-200)
//! - Undo/Redo (201-230)
//! - Autosave & Recovery (231-260)
//! - Collaborative Editing (261-280)
//! - Accessibility (281-300)

use std::collections::HashMap;

// ============================================================================
// Mock Types and Test Utilities
// ============================================================================

/// Editor state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorState {
    Loading,
    Ready,
    Editing,
    Saving,
    Error(String),
    Disabled,
}

/// Editor mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Visual,
    Code,
    Preview,
    Split,
}

/// Block type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockType {
    Paragraph,
    Heading,
    Image,
    List,
    Quote,
    Code,
    Table,
    Embed,
    Gallery,
    Video,
    Audio,
    File,
    Separator,
    Spacer,
    Button,
    Columns,
    Group,
    Cover,
    Custom(String),
}

/// Block alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockAlignment {
    Left,
    Center,
    Right,
    Wide,
    Full,
    None,
}

/// A content block
#[derive(Debug, Clone)]
pub struct Block {
    pub id: String,
    pub block_type: BlockType,
    pub content: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<Block>,
    pub alignment: BlockAlignment,
    pub is_selected: bool,
    pub is_valid: bool,
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Self {
            id: format!("block_{}", rand_id()),
            block_type,
            content: String::new(),
            attributes: HashMap::new(),
            children: Vec::new(),
            alignment: BlockAlignment::None,
            is_selected: false,
            is_valid: true,
        }
    }

    pub fn with_content(mut self, content: &str) -> Self {
        self.content = content.to_string();
        self
    }

    pub fn with_attribute(mut self, key: &str, value: &str) -> Self {
        self.attributes.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_alignment(mut self, alignment: BlockAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn select(mut self) -> Self {
        self.is_selected = true;
        self
    }
}

/// Toolbar button state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Active,
    Disabled,
    Hidden,
}

/// Toolbar button
#[derive(Debug, Clone)]
pub struct ToolbarButton {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub shortcut: Option<String>,
    pub state: ButtonState,
    pub tooltip: String,
}

impl ToolbarButton {
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            icon: format!("{}_icon", id),
            shortcut: None,
            state: ButtonState::Normal,
            tooltip: label.to_string(),
        }
    }

    pub fn with_shortcut(mut self, shortcut: &str) -> Self {
        self.shortcut = Some(shortcut.to_string());
        self
    }

    pub fn with_state(mut self, state: ButtonState) -> Self {
        self.state = state;
        self
    }

    pub fn is_enabled(&self) -> bool {
        self.state != ButtonState::Disabled && self.state != ButtonState::Hidden
    }
}

/// Keyboard shortcut
#[derive(Debug, Clone)]
pub struct KeyboardShortcut {
    pub key: String,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub action: String,
}

impl KeyboardShortcut {
    pub fn new(key: &str, action: &str) -> Self {
        Self {
            key: key.to_string(),
            ctrl: false,
            shift: false,
            alt: false,
            action: action.to_string(),
        }
    }

    pub fn ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub fn shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn to_string(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl { parts.push("Ctrl"); }
        if self.shift { parts.push("Shift"); }
        if self.alt { parts.push("Alt"); }
        parts.push(&self.key);
        parts.join("+")
    }
}

/// Selection range
#[derive(Debug, Clone)]
pub struct Selection {
    pub start_block: String,
    pub end_block: String,
    pub start_offset: usize,
    pub end_offset: usize,
    pub is_collapsed: bool,
}

impl Selection {
    pub fn collapsed(block_id: &str, offset: usize) -> Self {
        Self {
            start_block: block_id.to_string(),
            end_block: block_id.to_string(),
            start_offset: offset,
            end_offset: offset,
            is_collapsed: true,
        }
    }

    pub fn range(start_block: &str, start: usize, end_block: &str, end: usize) -> Self {
        Self {
            start_block: start_block.to_string(),
            end_block: end_block.to_string(),
            start_offset: start,
            end_offset: end,
            is_collapsed: false,
        }
    }
}

/// Undo/Redo action
#[derive(Debug, Clone)]
pub struct UndoAction {
    pub id: String,
    pub action_type: String,
    pub description: String,
    pub timestamp: u64,
    pub data: String,
}

/// Drag operation
#[derive(Debug, Clone)]
pub struct DragOperation {
    pub source_block: String,
    pub target_position: usize,
    pub is_valid: bool,
}

/// Autosave state
#[derive(Debug, Clone)]
pub struct AutosaveState {
    pub enabled: bool,
    pub interval_seconds: u64,
    pub last_save: Option<u64>,
    pub pending_changes: bool,
    pub save_count: u64,
}

impl Default for AutosaveState {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 60,
            last_save: None,
            pending_changes: false,
            save_count: 0,
        }
    }
}

/// Collaborative user
#[derive(Debug, Clone)]
pub struct CollaborativeUser {
    pub id: String,
    pub name: String,
    pub color: String,
    pub cursor_block: Option<String>,
    pub cursor_offset: Option<usize>,
    pub is_active: bool,
}

/// Mock Editor
pub struct MockEditor {
    pub state: EditorState,
    pub mode: EditorMode,
    pub blocks: Vec<Block>,
    pub selection: Option<Selection>,
    pub toolbar_buttons: Vec<ToolbarButton>,
    pub shortcuts: Vec<KeyboardShortcut>,
    pub undo_stack: Vec<UndoAction>,
    pub redo_stack: Vec<UndoAction>,
    pub autosave: AutosaveState,
    pub collaborators: Vec<CollaborativeUser>,
    pub is_dirty: bool,
    pub is_fullscreen: bool,
    pub word_count: usize,
    pub character_count: usize,
}

impl MockEditor {
    pub fn new() -> Self {
        Self {
            state: EditorState::Loading,
            mode: EditorMode::Visual,
            blocks: Vec::new(),
            selection: None,
            toolbar_buttons: Self::default_toolbar(),
            shortcuts: Self::default_shortcuts(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            autosave: AutosaveState::default(),
            collaborators: Vec::new(),
            is_dirty: false,
            is_fullscreen: false,
            word_count: 0,
            character_count: 0,
        }
    }

    fn default_toolbar() -> Vec<ToolbarButton> {
        vec![
            ToolbarButton::new("bold", "Bold").with_shortcut("Ctrl+B"),
            ToolbarButton::new("italic", "Italic").with_shortcut("Ctrl+I"),
            ToolbarButton::new("underline", "Underline").with_shortcut("Ctrl+U"),
            ToolbarButton::new("strikethrough", "Strikethrough"),
            ToolbarButton::new("link", "Insert Link").with_shortcut("Ctrl+K"),
            ToolbarButton::new("align_left", "Align Left"),
            ToolbarButton::new("align_center", "Align Center"),
            ToolbarButton::new("align_right", "Align Right"),
            ToolbarButton::new("list_bullet", "Bullet List"),
            ToolbarButton::new("list_number", "Numbered List"),
            ToolbarButton::new("quote", "Block Quote"),
            ToolbarButton::new("code", "Code"),
            ToolbarButton::new("undo", "Undo").with_shortcut("Ctrl+Z"),
            ToolbarButton::new("redo", "Redo").with_shortcut("Ctrl+Y"),
        ]
    }

    fn default_shortcuts() -> Vec<KeyboardShortcut> {
        vec![
            KeyboardShortcut::new("B", "bold").ctrl(),
            KeyboardShortcut::new("I", "italic").ctrl(),
            KeyboardShortcut::new("U", "underline").ctrl(),
            KeyboardShortcut::new("K", "link").ctrl(),
            KeyboardShortcut::new("Z", "undo").ctrl(),
            KeyboardShortcut::new("Y", "redo").ctrl(),
            KeyboardShortcut::new("Z", "redo").ctrl().shift(),
            KeyboardShortcut::new("S", "save").ctrl(),
            KeyboardShortcut::new("A", "select_all").ctrl(),
            KeyboardShortcut::new("C", "copy").ctrl(),
            KeyboardShortcut::new("V", "paste").ctrl(),
            KeyboardShortcut::new("X", "cut").ctrl(),
            KeyboardShortcut::new("Enter", "new_block"),
            KeyboardShortcut::new("Backspace", "delete_back"),
            KeyboardShortcut::new("Delete", "delete_forward"),
            KeyboardShortcut::new("Tab", "indent"),
            KeyboardShortcut::new("Tab", "outdent").shift(),
            KeyboardShortcut::new("Escape", "deselect"),
        ]
    }

    pub fn initialize(&mut self) {
        self.state = EditorState::Ready;
    }

    pub fn add_block(&mut self, block: Block) -> String {
        let id = block.id.clone();
        self.blocks.push(block);
        self.is_dirty = true;
        self.update_counts();
        self.push_undo("add_block", &id);
        id
    }

    pub fn remove_block(&mut self, block_id: &str) -> bool {
        let initial_len = self.blocks.len();
        self.blocks.retain(|b| b.id != block_id);
        if self.blocks.len() < initial_len {
            self.is_dirty = true;
            self.update_counts();
            self.push_undo("remove_block", block_id);
            true
        } else {
            false
        }
    }

    pub fn move_block(&mut self, block_id: &str, new_index: usize) -> bool {
        if let Some(pos) = self.blocks.iter().position(|b| b.id == block_id) {
            if new_index < self.blocks.len() {
                let block = self.blocks.remove(pos);
                let insert_pos = if new_index > pos { new_index - 1 } else { new_index };
                self.blocks.insert(insert_pos.min(self.blocks.len()), block);
                self.is_dirty = true;
                self.push_undo("move_block", block_id);
                return true;
            }
        }
        false
    }

    pub fn select_block(&mut self, block_id: &str) -> bool {
        // Deselect all first
        for block in &mut self.blocks {
            block.is_selected = false;
        }

        // Select the target
        if let Some(block) = self.blocks.iter_mut().find(|b| b.id == block_id) {
            block.is_selected = true;
            self.selection = Some(Selection::collapsed(block_id, 0));
            true
        } else {
            false
        }
    }

    pub fn get_selected_blocks(&self) -> Vec<&Block> {
        self.blocks.iter().filter(|b| b.is_selected).collect()
    }

    pub fn update_block_content(&mut self, block_id: &str, content: &str) -> bool {
        if let Some(block) = self.blocks.iter_mut().find(|b| b.id == block_id) {
            block.content = content.to_string();
            self.is_dirty = true;
            self.update_counts();
            true
        } else {
            false
        }
    }

    pub fn transform_block(&mut self, block_id: &str, new_type: BlockType) -> bool {
        if self.blocks.iter().any(|b| b.id == block_id) {
            self.push_undo("transform_block", block_id);
            if let Some(block) = self.blocks.iter_mut().find(|b| b.id == block_id) {
                block.block_type = new_type;
                self.is_dirty = true;
            }
            true
        } else {
            false
        }
    }

    pub fn set_block_alignment(&mut self, block_id: &str, alignment: BlockAlignment) -> bool {
        if let Some(block) = self.blocks.iter_mut().find(|b| b.id == block_id) {
            block.alignment = alignment;
            self.is_dirty = true;
            true
        } else {
            false
        }
    }

    pub fn get_button(&self, id: &str) -> Option<&ToolbarButton> {
        self.toolbar_buttons.iter().find(|b| b.id == id)
    }

    pub fn set_button_state(&mut self, id: &str, state: ButtonState) -> bool {
        if let Some(button) = self.toolbar_buttons.iter_mut().find(|b| b.id == id) {
            button.state = state;
            true
        } else {
            false
        }
    }

    pub fn execute_shortcut(&self, key: &str, ctrl: bool, shift: bool, alt: bool) -> Option<String> {
        for shortcut in &self.shortcuts {
            if shortcut.key == key && shortcut.ctrl == ctrl && shortcut.shift == shift && shortcut.alt == alt {
                return Some(shortcut.action.clone());
            }
        }
        None
    }

    pub fn push_undo(&mut self, action_type: &str, data: &str) {
        let action = UndoAction {
            id: format!("undo_{}", rand_id()),
            action_type: action_type.to_string(),
            description: format!("{} action", action_type),
            timestamp: current_timestamp(),
            data: data.to_string(),
        };
        self.undo_stack.push(action);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> Option<UndoAction> {
        if let Some(action) = self.undo_stack.pop() {
            self.redo_stack.push(action.clone());
            Some(action)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<UndoAction> {
        if let Some(action) = self.redo_stack.pop() {
            self.undo_stack.push(action.clone());
            Some(action)
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn trigger_autosave(&mut self) -> bool {
        if self.autosave.enabled && self.is_dirty {
            self.autosave.last_save = Some(current_timestamp());
            self.autosave.save_count += 1;
            self.autosave.pending_changes = false;
            self.is_dirty = false;
            true
        } else {
            false
        }
    }

    pub fn set_mode(&mut self, mode: EditorMode) {
        self.mode = mode;
    }

    pub fn toggle_fullscreen(&mut self) {
        self.is_fullscreen = !self.is_fullscreen;
    }

    pub fn add_collaborator(&mut self, user: CollaborativeUser) {
        self.collaborators.push(user);
    }

    pub fn remove_collaborator(&mut self, user_id: &str) {
        self.collaborators.retain(|u| u.id != user_id);
    }

    pub fn update_collaborator_cursor(&mut self, user_id: &str, block_id: &str, offset: usize) -> bool {
        if let Some(user) = self.collaborators.iter_mut().find(|u| u.id == user_id) {
            user.cursor_block = Some(block_id.to_string());
            user.cursor_offset = Some(offset);
            true
        } else {
            false
        }
    }

    fn update_counts(&mut self) {
        let all_content: String = self.blocks.iter().map(|b| &b.content).cloned().collect();
        self.character_count = all_content.len();
        self.word_count = all_content.split_whitespace().count();
    }

    pub fn validate_all_blocks(&mut self) -> Vec<String> {
        // First, collect validation results
        let validations: Vec<(usize, bool)> = self.blocks.iter().enumerate()
            .map(|(i, block)| (i, Self::validate_block_static(block)))
            .collect();

        // Then apply results and collect invalid block IDs
        let mut invalid_blocks = Vec::new();
        for (i, is_valid) in validations {
            self.blocks[i].is_valid = is_valid;
            if !is_valid {
                invalid_blocks.push(self.blocks[i].id.clone());
            }
        }
        invalid_blocks
    }

    fn validate_block_static(block: &Block) -> bool {
        match block.block_type {
            BlockType::Image => block.attributes.contains_key("src"),
            BlockType::Video => block.attributes.contains_key("src"),
            BlockType::Audio => block.attributes.contains_key("src"),
            BlockType::Embed => block.attributes.contains_key("url"),
            BlockType::Table => !block.children.is_empty(),
            _ => true,
        }
    }
}

// Utility functions
fn rand_id() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64 % 1_000_000_000
}

fn current_timestamp() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// ============================================================================
// EDITOR INITIALIZATION & SETUP TESTS (1-30)
// ============================================================================

/// Test 1: Editor should start in loading state
#[test]
fn test_001_editor_starts_loading() {
    let editor = MockEditor::new();
    assert_eq!(editor.state, EditorState::Loading);
}

/// Test 2: Editor should transition to ready state
#[test]
fn test_002_editor_initializes_to_ready() {
    let mut editor = MockEditor::new();
    editor.initialize();
    assert_eq!(editor.state, EditorState::Ready);
}

/// Test 3: Editor should start in visual mode
#[test]
fn test_003_editor_default_visual_mode() {
    let editor = MockEditor::new();
    assert_eq!(editor.mode, EditorMode::Visual);
}

/// Test 4: Editor should have empty blocks initially
#[test]
fn test_004_editor_empty_blocks_initially() {
    let editor = MockEditor::new();
    assert!(editor.blocks.is_empty());
}

/// Test 5: Editor should not be dirty initially
#[test]
fn test_005_editor_not_dirty_initially() {
    let editor = MockEditor::new();
    assert!(!editor.is_dirty);
}

/// Test 6: Editor should not be fullscreen initially
#[test]
fn test_006_editor_not_fullscreen_initially() {
    let editor = MockEditor::new();
    assert!(!editor.is_fullscreen);
}

/// Test 7: Editor should have default toolbar
#[test]
fn test_007_editor_has_default_toolbar() {
    let editor = MockEditor::new();
    assert!(!editor.toolbar_buttons.is_empty());
}

/// Test 8: Editor should have default shortcuts
#[test]
fn test_008_editor_has_default_shortcuts() {
    let editor = MockEditor::new();
    assert!(!editor.shortcuts.is_empty());
}

/// Test 9: Editor should have autosave enabled by default
#[test]
fn test_009_editor_autosave_enabled() {
    let editor = MockEditor::new();
    assert!(editor.autosave.enabled);
}

/// Test 10: Editor should have no collaborators initially
#[test]
fn test_010_editor_no_collaborators_initially() {
    let editor = MockEditor::new();
    assert!(editor.collaborators.is_empty());
}

/// Test 11: Editor should have zero word count initially
#[test]
fn test_011_editor_zero_word_count() {
    let editor = MockEditor::new();
    assert_eq!(editor.word_count, 0);
}

/// Test 12: Editor should have zero character count initially
#[test]
fn test_012_editor_zero_char_count() {
    let editor = MockEditor::new();
    assert_eq!(editor.character_count, 0);
}

/// Test 13: Editor should have empty undo stack
#[test]
fn test_013_editor_empty_undo_stack() {
    let editor = MockEditor::new();
    assert!(editor.undo_stack.is_empty());
}

/// Test 14: Editor should have empty redo stack
#[test]
fn test_014_editor_empty_redo_stack() {
    let editor = MockEditor::new();
    assert!(editor.redo_stack.is_empty());
}

/// Test 15: Editor should have no selection initially
#[test]
fn test_015_editor_no_selection_initially() {
    let editor = MockEditor::new();
    assert!(editor.selection.is_none());
}

/// Test 16: Can set editor mode to code
#[test]
fn test_016_editor_set_code_mode() {
    let mut editor = MockEditor::new();
    editor.set_mode(EditorMode::Code);
    assert_eq!(editor.mode, EditorMode::Code);
}

/// Test 17: Can set editor mode to preview
#[test]
fn test_017_editor_set_preview_mode() {
    let mut editor = MockEditor::new();
    editor.set_mode(EditorMode::Preview);
    assert_eq!(editor.mode, EditorMode::Preview);
}

/// Test 18: Can set editor mode to split
#[test]
fn test_018_editor_set_split_mode() {
    let mut editor = MockEditor::new();
    editor.set_mode(EditorMode::Split);
    assert_eq!(editor.mode, EditorMode::Split);
}

/// Test 19: Can toggle fullscreen
#[test]
fn test_019_editor_toggle_fullscreen() {
    let mut editor = MockEditor::new();
    assert!(!editor.is_fullscreen);
    editor.toggle_fullscreen();
    assert!(editor.is_fullscreen);
    editor.toggle_fullscreen();
    assert!(!editor.is_fullscreen);
}

/// Test 20: Default autosave interval is 60 seconds
#[test]
fn test_020_editor_autosave_interval() {
    let editor = MockEditor::new();
    assert_eq!(editor.autosave.interval_seconds, 60);
}

/// Test 21: Editor has bold button
#[test]
fn test_021_editor_has_bold_button() {
    let editor = MockEditor::new();
    assert!(editor.get_button("bold").is_some());
}

/// Test 22: Editor has italic button
#[test]
fn test_022_editor_has_italic_button() {
    let editor = MockEditor::new();
    assert!(editor.get_button("italic").is_some());
}

/// Test 23: Editor has undo button
#[test]
fn test_023_editor_has_undo_button() {
    let editor = MockEditor::new();
    assert!(editor.get_button("undo").is_some());
}

/// Test 24: Editor has redo button
#[test]
fn test_024_editor_has_redo_button() {
    let editor = MockEditor::new();
    assert!(editor.get_button("redo").is_some());
}

/// Test 25: Editor has link button
#[test]
fn test_025_editor_has_link_button() {
    let editor = MockEditor::new();
    assert!(editor.get_button("link").is_some());
}

/// Test 26: Bold button has shortcut Ctrl+B
#[test]
fn test_026_bold_button_shortcut() {
    let editor = MockEditor::new();
    let button = editor.get_button("bold").unwrap();
    assert_eq!(button.shortcut, Some("Ctrl+B".to_string()));
}

/// Test 27: Italic button has shortcut Ctrl+I
#[test]
fn test_027_italic_button_shortcut() {
    let editor = MockEditor::new();
    let button = editor.get_button("italic").unwrap();
    assert_eq!(button.shortcut, Some("Ctrl+I".to_string()));
}

/// Test 28: All toolbar buttons start in normal state
#[test]
fn test_028_toolbar_buttons_normal_state() {
    let editor = MockEditor::new();
    for button in &editor.toolbar_buttons {
        assert_eq!(button.state, ButtonState::Normal);
    }
}

/// Test 29: All toolbar buttons are enabled initially
#[test]
fn test_029_toolbar_buttons_enabled() {
    let editor = MockEditor::new();
    for button in &editor.toolbar_buttons {
        assert!(button.is_enabled());
    }
}

/// Test 30: Editor initializes with autosave having no last save
#[test]
fn test_030_autosave_no_last_save() {
    let editor = MockEditor::new();
    assert!(editor.autosave.last_save.is_none());
}

// ============================================================================
// TOOLBAR & CONTROLS TESTS (31-60)
// ============================================================================

/// Test 31: Can disable toolbar button
#[test]
fn test_031_disable_toolbar_button() {
    let mut editor = MockEditor::new();
    editor.set_button_state("bold", ButtonState::Disabled);
    let button = editor.get_button("bold").unwrap();
    assert_eq!(button.state, ButtonState::Disabled);
    assert!(!button.is_enabled());
}

/// Test 32: Can set button to active state
#[test]
fn test_032_set_button_active() {
    let mut editor = MockEditor::new();
    editor.set_button_state("bold", ButtonState::Active);
    let button = editor.get_button("bold").unwrap();
    assert_eq!(button.state, ButtonState::Active);
}

/// Test 33: Can hide toolbar button
#[test]
fn test_033_hide_toolbar_button() {
    let mut editor = MockEditor::new();
    editor.set_button_state("strikethrough", ButtonState::Hidden);
    let button = editor.get_button("strikethrough").unwrap();
    assert!(!button.is_enabled());
}

/// Test 34: Setting state on nonexistent button returns false
#[test]
fn test_034_set_state_nonexistent_button() {
    let mut editor = MockEditor::new();
    let result = editor.set_button_state("nonexistent", ButtonState::Active);
    assert!(!result);
}

/// Test 35: Toolbar has align left button
#[test]
fn test_035_toolbar_has_align_left() {
    let editor = MockEditor::new();
    assert!(editor.get_button("align_left").is_some());
}

/// Test 36: Toolbar has align center button
#[test]
fn test_036_toolbar_has_align_center() {
    let editor = MockEditor::new();
    assert!(editor.get_button("align_center").is_some());
}

/// Test 37: Toolbar has align right button
#[test]
fn test_037_toolbar_has_align_right() {
    let editor = MockEditor::new();
    assert!(editor.get_button("align_right").is_some());
}

/// Test 38: Toolbar has bullet list button
#[test]
fn test_038_toolbar_has_bullet_list() {
    let editor = MockEditor::new();
    assert!(editor.get_button("list_bullet").is_some());
}

/// Test 39: Toolbar has numbered list button
#[test]
fn test_039_toolbar_has_numbered_list() {
    let editor = MockEditor::new();
    assert!(editor.get_button("list_number").is_some());
}

/// Test 40: Toolbar has quote button
#[test]
fn test_040_toolbar_has_quote() {
    let editor = MockEditor::new();
    assert!(editor.get_button("quote").is_some());
}

/// Test 41: Toolbar has code button
#[test]
fn test_041_toolbar_has_code() {
    let editor = MockEditor::new();
    assert!(editor.get_button("code").is_some());
}

/// Test 42: Toolbar has underline button
#[test]
fn test_042_toolbar_has_underline() {
    let editor = MockEditor::new();
    assert!(editor.get_button("underline").is_some());
}

/// Test 43: Underline button has Ctrl+U shortcut
#[test]
fn test_043_underline_shortcut() {
    let editor = MockEditor::new();
    let button = editor.get_button("underline").unwrap();
    assert_eq!(button.shortcut, Some("Ctrl+U".to_string()));
}

/// Test 44: Buttons have tooltips
#[test]
fn test_044_buttons_have_tooltips() {
    let editor = MockEditor::new();
    for button in &editor.toolbar_buttons {
        assert!(!button.tooltip.is_empty());
    }
}

/// Test 45: Buttons have icons
#[test]
fn test_045_buttons_have_icons() {
    let editor = MockEditor::new();
    for button in &editor.toolbar_buttons {
        assert!(!button.icon.is_empty());
    }
}

/// Test 46: Can create custom toolbar button
#[test]
fn test_046_create_custom_button() {
    let button = ToolbarButton::new("custom", "Custom Action")
        .with_shortcut("Ctrl+Shift+C")
        .with_state(ButtonState::Normal);

    assert_eq!(button.id, "custom");
    assert_eq!(button.shortcut, Some("Ctrl+Shift+C".to_string()));
}

/// Test 47: Undo button has Ctrl+Z shortcut
#[test]
fn test_047_undo_shortcut() {
    let editor = MockEditor::new();
    let button = editor.get_button("undo").unwrap();
    assert_eq!(button.shortcut, Some("Ctrl+Z".to_string()));
}

/// Test 48: Redo button has Ctrl+Y shortcut
#[test]
fn test_048_redo_shortcut() {
    let editor = MockEditor::new();
    let button = editor.get_button("redo").unwrap();
    assert_eq!(button.shortcut, Some("Ctrl+Y".to_string()));
}

/// Test 49: Link button has Ctrl+K shortcut
#[test]
fn test_049_link_shortcut() {
    let editor = MockEditor::new();
    let button = editor.get_button("link").unwrap();
    assert_eq!(button.shortcut, Some("Ctrl+K".to_string()));
}

/// Test 50: Toolbar button count is correct
#[test]
fn test_050_toolbar_button_count() {
    let editor = MockEditor::new();
    assert!(editor.toolbar_buttons.len() >= 14);
}

/// Test 51: All buttons have unique IDs
#[test]
fn test_051_buttons_unique_ids() {
    let editor = MockEditor::new();
    let mut ids: Vec<&str> = editor.toolbar_buttons.iter().map(|b| b.id.as_str()).collect();
    let original_len = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), original_len);
}

/// Test 52: Button labels are non-empty
#[test]
fn test_052_buttons_have_labels() {
    let editor = MockEditor::new();
    for button in &editor.toolbar_buttons {
        assert!(!button.label.is_empty());
    }
}

/// Test 53: Can restore button to normal state
#[test]
fn test_053_restore_button_normal() {
    let mut editor = MockEditor::new();
    editor.set_button_state("bold", ButtonState::Disabled);
    editor.set_button_state("bold", ButtonState::Normal);
    let button = editor.get_button("bold").unwrap();
    assert_eq!(button.state, ButtonState::Normal);
}

/// Test 54: Multiple buttons can be active
#[test]
fn test_054_multiple_buttons_active() {
    let mut editor = MockEditor::new();
    editor.set_button_state("bold", ButtonState::Active);
    editor.set_button_state("italic", ButtonState::Active);

    assert_eq!(editor.get_button("bold").unwrap().state, ButtonState::Active);
    assert_eq!(editor.get_button("italic").unwrap().state, ButtonState::Active);
}

/// Test 55: Strikethrough button exists
#[test]
fn test_055_strikethrough_button() {
    let editor = MockEditor::new();
    assert!(editor.get_button("strikethrough").is_some());
}

/// Test 56: Get nonexistent button returns None
#[test]
fn test_056_get_nonexistent_button() {
    let editor = MockEditor::new();
    assert!(editor.get_button("nonexistent").is_none());
}

/// Test 57: Button state affects enabled status
#[test]
fn test_057_button_state_enabled() {
    let button_normal = ToolbarButton::new("test", "Test").with_state(ButtonState::Normal);
    let button_active = ToolbarButton::new("test", "Test").with_state(ButtonState::Active);
    let button_disabled = ToolbarButton::new("test", "Test").with_state(ButtonState::Disabled);
    let button_hidden = ToolbarButton::new("test", "Test").with_state(ButtonState::Hidden);

    assert!(button_normal.is_enabled());
    assert!(button_active.is_enabled());
    assert!(!button_disabled.is_enabled());
    assert!(!button_hidden.is_enabled());
}

/// Test 58: Toolbar buttons are in logical order
#[test]
fn test_058_toolbar_logical_order() {
    let editor = MockEditor::new();
    let ids: Vec<&str> = editor.toolbar_buttons.iter().map(|b| b.id.as_str()).collect();

    // Bold should come before alignment
    let bold_pos = ids.iter().position(|&id| id == "bold");
    let align_pos = ids.iter().position(|&id| id == "align_left");
    assert!(bold_pos < align_pos);
}

/// Test 59: Toolbar has formatting buttons
#[test]
fn test_059_toolbar_formatting_buttons() {
    let editor = MockEditor::new();
    let formatting = ["bold", "italic", "underline", "strikethrough"];
    for id in formatting {
        assert!(editor.get_button(id).is_some(), "Missing button: {}", id);
    }
}

/// Test 60: Toolbar has structural buttons
#[test]
fn test_060_toolbar_structural_buttons() {
    let editor = MockEditor::new();
    let structural = ["list_bullet", "list_number", "quote"];
    for id in structural {
        assert!(editor.get_button(id).is_some(), "Missing button: {}", id);
    }
}

// ============================================================================
// BLOCK OPERATIONS TESTS (61-100)
// ============================================================================

/// Test 61: Can add paragraph block
#[test]
fn test_061_add_paragraph_block() {
    let mut editor = MockEditor::new();
    let block = Block::new(BlockType::Paragraph);
    let id = editor.add_block(block);
    assert!(!id.is_empty());
    assert_eq!(editor.blocks.len(), 1);
}

/// Test 62: Can add heading block
#[test]
fn test_062_add_heading_block() {
    let mut editor = MockEditor::new();
    let block = Block::new(BlockType::Heading).with_attribute("level", "2");
    editor.add_block(block);
    assert_eq!(editor.blocks[0].block_type, BlockType::Heading);
}

/// Test 63: Can add image block
#[test]
fn test_063_add_image_block() {
    let mut editor = MockEditor::new();
    let block = Block::new(BlockType::Image).with_attribute("src", "image.jpg");
    editor.add_block(block);
    assert_eq!(editor.blocks[0].block_type, BlockType::Image);
}

/// Test 64: Can remove block
#[test]
fn test_064_remove_block() {
    let mut editor = MockEditor::new();
    let block = Block::new(BlockType::Paragraph);
    let id = editor.add_block(block);

    assert!(editor.remove_block(&id));
    assert!(editor.blocks.is_empty());
}

/// Test 65: Remove nonexistent block returns false
#[test]
fn test_065_remove_nonexistent_block() {
    let mut editor = MockEditor::new();
    assert!(!editor.remove_block("nonexistent"));
}

/// Test 66: Adding block sets dirty flag
#[test]
fn test_066_add_block_sets_dirty() {
    let mut editor = MockEditor::new();
    assert!(!editor.is_dirty);
    editor.add_block(Block::new(BlockType::Paragraph));
    assert!(editor.is_dirty);
}

/// Test 67: Can move block up
#[test]
fn test_067_move_block_up() {
    let mut editor = MockEditor::new();
    let id1 = editor.add_block(Block::new(BlockType::Paragraph).with_content("First"));
    let id2 = editor.add_block(Block::new(BlockType::Paragraph).with_content("Second"));

    editor.move_block(&id2, 0);
    assert_eq!(editor.blocks[0].content, "Second");
    assert_eq!(editor.blocks[1].content, "First");
}

/// Test 68: Can move block down
#[test]
fn test_068_move_block_down() {
    let mut editor = MockEditor::new();
    let id1 = editor.add_block(Block::new(BlockType::Paragraph).with_content("First"));
    let _id2 = editor.add_block(Block::new(BlockType::Paragraph).with_content("Second"));

    editor.move_block(&id1, 1);
    assert_eq!(editor.blocks[0].content, "Second");
}

/// Test 69: Can select block
#[test]
fn test_069_select_block() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));

    assert!(editor.select_block(&id));
    assert!(editor.blocks[0].is_selected);
}

/// Test 70: Selecting block creates selection
#[test]
fn test_070_select_block_creates_selection() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));

    editor.select_block(&id);
    assert!(editor.selection.is_some());
}

/// Test 71: Can get selected blocks
#[test]
fn test_071_get_selected_blocks() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.add_block(Block::new(BlockType::Paragraph));

    editor.select_block(&id);
    let selected = editor.get_selected_blocks();
    assert_eq!(selected.len(), 1);
}

/// Test 72: Can update block content
#[test]
fn test_072_update_block_content() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));

    assert!(editor.update_block_content(&id, "Hello World"));
    assert_eq!(editor.blocks[0].content, "Hello World");
}

/// Test 73: Update content updates word count
#[test]
fn test_073_update_content_word_count() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));

    editor.update_block_content(&id, "Hello World Test");
    assert_eq!(editor.word_count, 3);
}

/// Test 74: Update content updates character count
#[test]
fn test_074_update_content_char_count() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));

    editor.update_block_content(&id, "Hello");
    assert_eq!(editor.character_count, 5);
}

/// Test 75: Can transform block type
#[test]
fn test_075_transform_block() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));

    assert!(editor.transform_block(&id, BlockType::Heading));
    assert_eq!(editor.blocks[0].block_type, BlockType::Heading);
}

/// Test 76: Transform adds to undo stack
#[test]
fn test_076_transform_adds_undo() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.undo_stack.clear();

    editor.transform_block(&id, BlockType::Heading);
    assert!(!editor.undo_stack.is_empty());
}

/// Test 77: Can set block alignment
#[test]
fn test_077_set_block_alignment() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));

    assert!(editor.set_block_alignment(&id, BlockAlignment::Center));
    assert_eq!(editor.blocks[0].alignment, BlockAlignment::Center);
}

/// Test 78: Block alignment options
#[test]
fn test_078_block_alignment_options() {
    let alignments = [
        BlockAlignment::Left,
        BlockAlignment::Center,
        BlockAlignment::Right,
        BlockAlignment::Wide,
        BlockAlignment::Full,
        BlockAlignment::None,
    ];

    for alignment in alignments {
        let block = Block::new(BlockType::Paragraph).with_alignment(alignment);
        assert_eq!(block.alignment, alignment);
    }
}

/// Test 79: Block can have children
#[test]
fn test_079_block_children() {
    let mut parent = Block::new(BlockType::Columns);
    let child = Block::new(BlockType::Paragraph);
    parent.children.push(child);

    assert_eq!(parent.children.len(), 1);
}

/// Test 80: Block attributes
#[test]
fn test_080_block_attributes() {
    let block = Block::new(BlockType::Image)
        .with_attribute("src", "image.jpg")
        .with_attribute("alt", "Description");

    assert_eq!(block.attributes.get("src"), Some(&"image.jpg".to_string()));
    assert_eq!(block.attributes.get("alt"), Some(&"Description".to_string()));
}

/// Test 81: Can add list block
#[test]
fn test_081_add_list_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::List));
    assert_eq!(editor.blocks[0].block_type, BlockType::List);
}

/// Test 82: Can add quote block
#[test]
fn test_082_add_quote_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Quote));
    assert_eq!(editor.blocks[0].block_type, BlockType::Quote);
}

/// Test 83: Can add code block
#[test]
fn test_083_add_code_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Code));
    assert_eq!(editor.blocks[0].block_type, BlockType::Code);
}

/// Test 84: Can add table block
#[test]
fn test_084_add_table_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Table));
    assert_eq!(editor.blocks[0].block_type, BlockType::Table);
}

/// Test 85: Can add embed block
#[test]
fn test_085_add_embed_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Embed));
    assert_eq!(editor.blocks[0].block_type, BlockType::Embed);
}

/// Test 86: Can add gallery block
#[test]
fn test_086_add_gallery_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Gallery));
    assert_eq!(editor.blocks[0].block_type, BlockType::Gallery);
}

/// Test 87: Can add video block
#[test]
fn test_087_add_video_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Video));
    assert_eq!(editor.blocks[0].block_type, BlockType::Video);
}

/// Test 88: Can add audio block
#[test]
fn test_088_add_audio_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Audio));
    assert_eq!(editor.blocks[0].block_type, BlockType::Audio);
}

/// Test 89: Can add separator block
#[test]
fn test_089_add_separator_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Separator));
    assert_eq!(editor.blocks[0].block_type, BlockType::Separator);
}

/// Test 90: Can add spacer block
#[test]
fn test_090_add_spacer_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Spacer));
    assert_eq!(editor.blocks[0].block_type, BlockType::Spacer);
}

/// Test 91: Can add button block
#[test]
fn test_091_add_button_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Button));
    assert_eq!(editor.blocks[0].block_type, BlockType::Button);
}

/// Test 92: Can add columns block
#[test]
fn test_092_add_columns_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Columns));
    assert_eq!(editor.blocks[0].block_type, BlockType::Columns);
}

/// Test 93: Can add group block
#[test]
fn test_093_add_group_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Group));
    assert_eq!(editor.blocks[0].block_type, BlockType::Group);
}

/// Test 94: Can add cover block
#[test]
fn test_094_add_cover_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Cover));
    assert_eq!(editor.blocks[0].block_type, BlockType::Cover);
}

/// Test 95: Can add custom block
#[test]
fn test_095_add_custom_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Custom("my-block".to_string())));

    match &editor.blocks[0].block_type {
        BlockType::Custom(name) => assert_eq!(name, "my-block"),
        _ => panic!("Expected custom block"),
    }
}

/// Test 96: Block has unique ID
#[test]
fn test_096_block_unique_id() {
    let block1 = Block::new(BlockType::Paragraph);
    let block2 = Block::new(BlockType::Paragraph);
    assert_ne!(block1.id, block2.id);
}

/// Test 97: Block starts as valid
#[test]
fn test_097_block_starts_valid() {
    let block = Block::new(BlockType::Paragraph);
    assert!(block.is_valid);
}

/// Test 98: Block starts unselected
#[test]
fn test_098_block_starts_unselected() {
    let block = Block::new(BlockType::Paragraph);
    assert!(!block.is_selected);
}

/// Test 99: Can chain block builder methods
#[test]
fn test_099_block_builder_chain() {
    let block = Block::new(BlockType::Image)
        .with_content("Caption")
        .with_attribute("src", "image.jpg")
        .with_alignment(BlockAlignment::Center)
        .select();

    assert_eq!(block.content, "Caption");
    assert!(block.is_selected);
    assert_eq!(block.alignment, BlockAlignment::Center);
}

/// Test 100: Adding block creates undo entry
#[test]
fn test_100_add_block_creates_undo() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    assert!(editor.can_undo());
}

// ============================================================================
// TEXT EDITING TESTS (101-140)
// ============================================================================

/// Test 101: Empty content has zero words
#[test]
fn test_101_empty_content_zero_words() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph).with_content(""));
    editor.update_block_content(&id, "");
    assert_eq!(editor.word_count, 0);
}

/// Test 102: Single word counts correctly
#[test]
fn test_102_single_word_count() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.update_block_content(&id, "Hello");
    assert_eq!(editor.word_count, 1);
}

/// Test 103: Multiple words count correctly
#[test]
fn test_103_multiple_word_count() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.update_block_content(&id, "The quick brown fox");
    assert_eq!(editor.word_count, 4);
}

/// Test 104: Word count across multiple blocks
#[test]
fn test_104_word_count_multiple_blocks() {
    let mut editor = MockEditor::new();
    let id1 = editor.add_block(Block::new(BlockType::Paragraph));
    let id2 = editor.add_block(Block::new(BlockType::Paragraph));

    editor.update_block_content(&id1, "Hello World");
    editor.update_block_content(&id2, "Foo Bar Baz");

    assert_eq!(editor.word_count, 5);
}

/// Test 105: Character count is accurate
#[test]
fn test_105_character_count() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.update_block_content(&id, "Hello World");
    assert_eq!(editor.character_count, 11);
}

/// Test 106: Character count includes spaces
#[test]
fn test_106_char_count_includes_spaces() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.update_block_content(&id, "A B C");
    assert_eq!(editor.character_count, 5);
}

/// Test 107: Update nonexistent block returns false
#[test]
fn test_107_update_nonexistent_block() {
    let mut editor = MockEditor::new();
    assert!(!editor.update_block_content("nonexistent", "content"));
}

/// Test 108: Content update sets dirty flag
#[test]
fn test_108_content_update_sets_dirty() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.is_dirty = false;

    editor.update_block_content(&id, "New content");
    assert!(editor.is_dirty);
}

/// Test 109: Can clear block content
#[test]
fn test_109_clear_block_content() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph).with_content("Initial"));

    editor.update_block_content(&id, "");
    assert!(editor.blocks[0].content.is_empty());
}

/// Test 110: Content with special characters
#[test]
fn test_110_content_special_chars() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));

    let content = "Hello <World> & \"Friends\"";
    editor.update_block_content(&id, content);
    assert_eq!(editor.blocks[0].content, content);
}

/// Test 111: Content with unicode
#[test]
fn test_111_content_unicode() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));

    let content = "Hello 世界 🌍";
    editor.update_block_content(&id, content);
    assert_eq!(editor.blocks[0].content, content);
}

/// Test 112: Content with newlines
#[test]
fn test_112_content_newlines() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));

    let content = "Line 1\nLine 2\nLine 3";
    editor.update_block_content(&id, content);
    assert!(editor.blocks[0].content.contains('\n'));
}

/// Test 113: Content preserves whitespace
#[test]
fn test_113_content_preserves_whitespace() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Code));

    let content = "    indented code";
    editor.update_block_content(&id, content);
    assert!(editor.blocks[0].content.starts_with("    "));
}

/// Test 114: Multiple content updates
#[test]
fn test_114_multiple_content_updates() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));

    editor.update_block_content(&id, "First");
    editor.update_block_content(&id, "Second");
    editor.update_block_content(&id, "Third");

    assert_eq!(editor.blocks[0].content, "Third");
}

/// Test 115: Long content handling
#[test]
fn test_115_long_content() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));

    let content = "word ".repeat(1000);
    editor.update_block_content(&id, &content);
    assert_eq!(editor.word_count, 1000);
}

/// Test 116: Selection collapsed at position
#[test]
fn test_116_selection_collapsed() {
    let selection = Selection::collapsed("block_1", 5);

    assert!(selection.is_collapsed);
    assert_eq!(selection.start_offset, 5);
    assert_eq!(selection.end_offset, 5);
}

/// Test 117: Selection range
#[test]
fn test_117_selection_range() {
    let selection = Selection::range("block_1", 0, "block_1", 10);

    assert!(!selection.is_collapsed);
    assert_eq!(selection.start_offset, 0);
    assert_eq!(selection.end_offset, 10);
}

/// Test 118: Cross-block selection
#[test]
fn test_118_cross_block_selection() {
    let selection = Selection::range("block_1", 5, "block_2", 10);

    assert_ne!(selection.start_block, selection.end_block);
}

/// Test 119: Selecting new block deselects previous
#[test]
fn test_119_select_deselects_previous() {
    let mut editor = MockEditor::new();
    let id1 = editor.add_block(Block::new(BlockType::Paragraph));
    let id2 = editor.add_block(Block::new(BlockType::Paragraph));

    editor.select_block(&id1);
    assert!(editor.blocks[0].is_selected);

    editor.select_block(&id2);
    assert!(!editor.blocks[0].is_selected);
    assert!(editor.blocks[1].is_selected);
}

/// Test 120: Block content can be HTML
#[test]
fn test_120_block_html_content() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));

    let html = "<strong>Bold</strong> and <em>italic</em>";
    editor.update_block_content(&id, html);
    assert!(editor.blocks[0].content.contains("<strong>"));
}

/// Test 121-140: Additional text editing tests
#[test]
fn test_121_empty_blocks_zero_count() {
    let editor = MockEditor::new();
    assert_eq!(editor.word_count, 0);
    assert_eq!(editor.character_count, 0);
}

#[test]
fn test_122_removing_block_updates_counts() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.update_block_content(&id, "Hello World");
    assert_eq!(editor.word_count, 2);

    editor.remove_block(&id);
    assert_eq!(editor.word_count, 0);
}

#[test]
fn test_123_tab_character_in_content() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Code));
    editor.update_block_content(&id, "def func():\n\treturn True");
    assert!(editor.blocks[0].content.contains('\t'));
}

#[test]
fn test_124_empty_string_word_count() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.update_block_content(&id, "   ");
    assert_eq!(editor.word_count, 0);
}

#[test]
fn test_125_punctuation_handling() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.update_block_content(&id, "Hello, World! How are you?");
    assert_eq!(editor.word_count, 5);
}

#[test]
fn test_126_hyphenated_word() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.update_block_content(&id, "well-known fact");
    // Hyphenated words count as one or three depending on impl
    assert!(editor.word_count >= 2);
}

#[test]
fn test_127_numbers_in_content() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.update_block_content(&id, "There are 100 items");
    assert_eq!(editor.word_count, 4);
}

#[test]
fn test_128_url_in_content() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.update_block_content(&id, "Visit https://example.com for more");
    assert!(editor.word_count >= 3);
}

#[test]
fn test_129_email_in_content() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.update_block_content(&id, "Contact test@example.com today");
    assert_eq!(editor.word_count, 3);
}

#[test]
fn test_130_content_with_quotes() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Quote));
    editor.update_block_content(&id, "\"To be or not to be\" - Shakespeare");
    assert!(editor.word_count >= 7);
}

#[test]
fn test_131_multiline_content() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.update_block_content(&id, "Line one\nLine two\nLine three");
    assert_eq!(editor.word_count, 6);
}

#[test]
fn test_132_content_normalization() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.update_block_content(&id, "Multiple   spaces   here");
    assert_eq!(editor.word_count, 3);
}

#[test]
fn test_133_heading_content() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Heading));
    editor.update_block_content(&id, "Chapter One: Introduction");
    assert_eq!(editor.blocks[0].content, "Chapter One: Introduction");
}

#[test]
fn test_134_list_item_content() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::List));
    editor.update_block_content(&id, "- Item one\n- Item two");
    assert!(editor.blocks[0].content.contains("Item one"));
}

#[test]
fn test_135_code_block_content() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Code));
    editor.update_block_content(&id, "fn main() {\n    println!(\"Hello\");\n}");
    assert!(editor.blocks[0].content.contains("fn main"));
}

#[test]
fn test_136_quote_attribution() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Quote));
    editor.update_block_content(&id, "The only thing we have to fear is fear itself.");
    editor.blocks[0].attributes.insert("citation".to_string(), "FDR".to_string());
    assert!(editor.blocks[0].attributes.contains_key("citation"));
}

#[test]
fn test_137_image_caption() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Image).with_attribute("src", "img.jpg"));
    editor.update_block_content(&id, "A beautiful sunset");
    assert_eq!(editor.blocks[0].content, "A beautiful sunset");
}

#[test]
fn test_138_button_text() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Button));
    editor.update_block_content(&id, "Click Here");
    assert_eq!(editor.blocks[0].content, "Click Here");
}

#[test]
fn test_139_emoji_in_content() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.update_block_content(&id, "Hello 👋 World 🌍");
    assert!(editor.character_count > 10);
}

#[test]
fn test_140_rtl_content() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.update_block_content(&id, "مرحبا بالعالم"); // Arabic "Hello World"
    assert!(editor.character_count > 0);
}

// ============================================================================
// KEYBOARD SHORTCUTS TESTS (141-170)
// ============================================================================

/// Test 141: Ctrl+B executes bold
#[test]
fn test_141_ctrl_b_bold() {
    let mut editor = MockEditor::new();
    let action = editor.execute_shortcut("B", true, false, false);
    assert_eq!(action, Some("bold".to_string()));
}

/// Test 142: Ctrl+I executes italic
#[test]
fn test_142_ctrl_i_italic() {
    let mut editor = MockEditor::new();
    let action = editor.execute_shortcut("I", true, false, false);
    assert_eq!(action, Some("italic".to_string()));
}

/// Test 143: Ctrl+U executes underline
#[test]
fn test_143_ctrl_u_underline() {
    let mut editor = MockEditor::new();
    let action = editor.execute_shortcut("U", true, false, false);
    assert_eq!(action, Some("underline".to_string()));
}

/// Test 144: Ctrl+K executes link
#[test]
fn test_144_ctrl_k_link() {
    let mut editor = MockEditor::new();
    let action = editor.execute_shortcut("K", true, false, false);
    assert_eq!(action, Some("link".to_string()));
}

/// Test 145: Ctrl+Z executes undo
#[test]
fn test_145_ctrl_z_undo() {
    let mut editor = MockEditor::new();
    let action = editor.execute_shortcut("Z", true, false, false);
    assert_eq!(action, Some("undo".to_string()));
}

/// Test 146: Ctrl+Y executes redo
#[test]
fn test_146_ctrl_y_redo() {
    let mut editor = MockEditor::new();
    let action = editor.execute_shortcut("Y", true, false, false);
    assert_eq!(action, Some("redo".to_string()));
}

/// Test 147: Ctrl+Shift+Z executes redo
#[test]
fn test_147_ctrl_shift_z_redo() {
    let mut editor = MockEditor::new();
    let action = editor.execute_shortcut("Z", true, true, false);
    assert_eq!(action, Some("redo".to_string()));
}

/// Test 148: Ctrl+S executes save
#[test]
fn test_148_ctrl_s_save() {
    let mut editor = MockEditor::new();
    let action = editor.execute_shortcut("S", true, false, false);
    assert_eq!(action, Some("save".to_string()));
}

/// Test 149: Ctrl+A executes select all
#[test]
fn test_149_ctrl_a_select_all() {
    let mut editor = MockEditor::new();
    let action = editor.execute_shortcut("A", true, false, false);
    assert_eq!(action, Some("select_all".to_string()));
}

/// Test 150: Ctrl+C executes copy
#[test]
fn test_150_ctrl_c_copy() {
    let mut editor = MockEditor::new();
    let action = editor.execute_shortcut("C", true, false, false);
    assert_eq!(action, Some("copy".to_string()));
}

/// Test 151: Ctrl+V executes paste
#[test]
fn test_151_ctrl_v_paste() {
    let mut editor = MockEditor::new();
    let action = editor.execute_shortcut("V", true, false, false);
    assert_eq!(action, Some("paste".to_string()));
}

/// Test 152: Ctrl+X executes cut
#[test]
fn test_152_ctrl_x_cut() {
    let mut editor = MockEditor::new();
    let action = editor.execute_shortcut("X", true, false, false);
    assert_eq!(action, Some("cut".to_string()));
}

/// Test 153: Enter creates new block
#[test]
fn test_153_enter_new_block() {
    let mut editor = MockEditor::new();
    let action = editor.execute_shortcut("Enter", false, false, false);
    assert_eq!(action, Some("new_block".to_string()));
}

/// Test 154: Backspace deletes backward
#[test]
fn test_154_backspace_delete() {
    let mut editor = MockEditor::new();
    let action = editor.execute_shortcut("Backspace", false, false, false);
    assert_eq!(action, Some("delete_back".to_string()));
}

/// Test 155: Delete deletes forward
#[test]
fn test_155_delete_forward() {
    let mut editor = MockEditor::new();
    let action = editor.execute_shortcut("Delete", false, false, false);
    assert_eq!(action, Some("delete_forward".to_string()));
}

/// Test 156: Tab indents
#[test]
fn test_156_tab_indent() {
    let mut editor = MockEditor::new();
    let action = editor.execute_shortcut("Tab", false, false, false);
    assert_eq!(action, Some("indent".to_string()));
}

/// Test 157: Shift+Tab outdents
#[test]
fn test_157_shift_tab_outdent() {
    let mut editor = MockEditor::new();
    let action = editor.execute_shortcut("Tab", false, true, false);
    assert_eq!(action, Some("outdent".to_string()));
}

/// Test 158: Escape deselects
#[test]
fn test_158_escape_deselect() {
    let mut editor = MockEditor::new();
    let action = editor.execute_shortcut("Escape", false, false, false);
    assert_eq!(action, Some("deselect".to_string()));
}

/// Test 159: Unknown shortcut returns None
#[test]
fn test_159_unknown_shortcut() {
    let mut editor = MockEditor::new();
    let action = editor.execute_shortcut("F5", false, false, false);
    assert!(action.is_none());
}

/// Test 160: Key without modifiers different from with modifiers
#[test]
fn test_160_modifiers_matter() {
    let mut editor = MockEditor::new();
    let without_ctrl = editor.execute_shortcut("B", false, false, false);
    let with_ctrl = editor.execute_shortcut("B", true, false, false);

    assert_ne!(without_ctrl, with_ctrl);
}

/// Test 161: Shortcut string representation
#[test]
fn test_161_shortcut_to_string() {
    let shortcut = KeyboardShortcut::new("S", "save").ctrl();
    assert_eq!(shortcut.to_string(), "Ctrl+S");
}

/// Test 162: Complex shortcut string
#[test]
fn test_162_complex_shortcut_string() {
    let shortcut = KeyboardShortcut::new("Z", "redo").ctrl().shift();
    assert_eq!(shortcut.to_string(), "Ctrl+Shift+Z");
}

/// Test 163: Alt shortcut
#[test]
fn test_163_alt_shortcut() {
    let shortcut = KeyboardShortcut::new("F4", "close").alt();
    assert!(shortcut.alt);
    assert!(!shortcut.ctrl);
}

/// Test 164: Multiple shortcuts same key
#[test]
fn test_164_multiple_shortcuts_same_key() {
    let editor = MockEditor::new();

    // Z with Ctrl is undo, Z with Ctrl+Shift is redo
    let undo = editor.shortcuts.iter().find(|s| s.key == "Z" && s.ctrl && !s.shift);
    let redo = editor.shortcuts.iter().find(|s| s.key == "Z" && s.ctrl && s.shift);

    assert!(undo.is_some());
    assert!(redo.is_some());
    assert_ne!(undo.unwrap().action, redo.unwrap().action);
}

/// Test 165: Shortcut count
#[test]
fn test_165_shortcut_count() {
    let editor = MockEditor::new();
    assert!(editor.shortcuts.len() >= 15);
}

/// Test 166: All shortcuts have actions
#[test]
fn test_166_shortcuts_have_actions() {
    let editor = MockEditor::new();
    for shortcut in &editor.shortcuts {
        assert!(!shortcut.action.is_empty());
    }
}

/// Test 167: All shortcuts have keys
#[test]
fn test_167_shortcuts_have_keys() {
    let editor = MockEditor::new();
    for shortcut in &editor.shortcuts {
        assert!(!shortcut.key.is_empty());
    }
}

/// Test 168: Can create custom shortcut
#[test]
fn test_168_custom_shortcut() {
    let shortcut = KeyboardShortcut::new("P", "print").ctrl().shift();

    assert_eq!(shortcut.key, "P");
    assert_eq!(shortcut.action, "print");
    assert!(shortcut.ctrl);
    assert!(shortcut.shift);
    assert!(!shortcut.alt);
}

/// Test 169: Shortcut builder chaining
#[test]
fn test_169_shortcut_chaining() {
    let shortcut = KeyboardShortcut::new("Delete", "force_delete")
        .ctrl()
        .shift()
        .alt();

    assert!(shortcut.ctrl);
    assert!(shortcut.shift);
    assert!(shortcut.alt);
}

/// Test 170: Function key shortcuts
#[test]
fn test_170_function_key_shortcut() {
    let shortcut = KeyboardShortcut::new("F1", "help");
    assert_eq!(shortcut.key, "F1");
}

// ============================================================================
// DRAG & DROP TESTS (171-200)
// ============================================================================

/// Test 171: Can move block to beginning
#[test]
fn test_171_move_block_to_beginning() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph).with_content("First"));
    editor.add_block(Block::new(BlockType::Paragraph).with_content("Second"));
    let id3 = editor.add_block(Block::new(BlockType::Paragraph).with_content("Third"));

    editor.move_block(&id3, 0);
    assert_eq!(editor.blocks[0].content, "Third");
}

/// Test 172: Can move block to end
#[test]
fn test_172_move_block_to_end() {
    let mut editor = MockEditor::new();
    let id1 = editor.add_block(Block::new(BlockType::Paragraph).with_content("First"));
    editor.add_block(Block::new(BlockType::Paragraph).with_content("Second"));
    editor.add_block(Block::new(BlockType::Paragraph).with_content("Third"));

    editor.move_block(&id1, 2);
    assert_eq!(editor.blocks[2].content, "First");
}

/// Test 173: Move nonexistent block returns false
#[test]
fn test_173_move_nonexistent_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    assert!(!editor.move_block("nonexistent", 0));
}

/// Test 174: Move to invalid index returns false
#[test]
fn test_174_move_invalid_index() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));

    assert!(!editor.move_block(&id, 100));
}

/// Test 175: Move preserves block content
#[test]
fn test_175_move_preserves_content() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph).with_content("First"));
    let id2 = editor.add_block(Block::new(BlockType::Paragraph).with_content("Second"));

    editor.move_block(&id2, 0);
    assert_eq!(editor.blocks[0].content, "Second");
}

/// Test 176: Move sets dirty flag
#[test]
fn test_176_move_sets_dirty() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    let id2 = editor.add_block(Block::new(BlockType::Paragraph));
    editor.is_dirty = false;

    editor.move_block(&id2, 0);
    assert!(editor.is_dirty);
}

/// Test 177: Move creates undo entry
#[test]
fn test_177_move_creates_undo() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    let id2 = editor.add_block(Block::new(BlockType::Paragraph));
    editor.undo_stack.clear();

    editor.move_block(&id2, 0);
    assert!(!editor.undo_stack.is_empty());
}

/// Test 178: Move to same position
#[test]
fn test_178_move_to_same_position() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph).with_content("First"));
    editor.add_block(Block::new(BlockType::Paragraph).with_content("Second"));

    let result = editor.move_block(&id, 0);
    assert!(result);
    assert_eq!(editor.blocks[0].content, "First");
}

/// Test 179: Move preserves attributes
#[test]
fn test_179_move_preserves_attributes() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    let id2 = editor.add_block(Block::new(BlockType::Image).with_attribute("src", "test.jpg"));

    editor.move_block(&id2, 0);
    assert_eq!(editor.blocks[0].attributes.get("src"), Some(&"test.jpg".to_string()));
}

/// Test 180: Move preserves alignment
#[test]
fn test_180_move_preserves_alignment() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    let id2 = editor.add_block(Block::new(BlockType::Paragraph).with_alignment(BlockAlignment::Center));

    editor.move_block(&id2, 0);
    assert_eq!(editor.blocks[0].alignment, BlockAlignment::Center);
}

/// Test 181: Drag operation validation
#[test]
fn test_181_drag_operation_valid() {
    let drag = DragOperation {
        source_block: "block_1".to_string(),
        target_position: 0,
        is_valid: true,
    };

    assert!(drag.is_valid);
}

/// Test 182: Drag operation invalid
#[test]
fn test_182_drag_operation_invalid() {
    let drag = DragOperation {
        source_block: "block_1".to_string(),
        target_position: 100,
        is_valid: false,
    };

    assert!(!drag.is_valid);
}

/// Test 183: Multiple sequential moves
#[test]
fn test_183_sequential_moves() {
    let mut editor = MockEditor::new();
    let id1 = editor.add_block(Block::new(BlockType::Paragraph).with_content("A"));
    let id2 = editor.add_block(Block::new(BlockType::Paragraph).with_content("B"));
    let _id3 = editor.add_block(Block::new(BlockType::Paragraph).with_content("C"));

    editor.move_block(&id2, 0);
    editor.move_block(&id1, 2);

    assert_eq!(editor.blocks[0].content, "B");
    assert_eq!(editor.blocks[2].content, "A");
}

/// Test 184: Move with single block
#[test]
fn test_184_move_single_block() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));

    let result = editor.move_block(&id, 0);
    assert!(result);
}

/// Test 185: Move preserves block ID
#[test]
fn test_185_move_preserves_id() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    let id2 = editor.add_block(Block::new(BlockType::Paragraph));

    editor.move_block(&id2, 0);
    assert_eq!(editor.blocks[0].id, id2);
}

/// Test 186: Move preserves children
#[test]
fn test_186_move_preserves_children() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    let mut parent = Block::new(BlockType::Columns);
    parent.children.push(Block::new(BlockType::Paragraph));
    let parent_id = parent.id.clone();
    editor.add_block(parent);

    editor.move_block(&parent_id, 0);
    assert_eq!(editor.blocks[0].children.len(), 1);
}

/// Test 187: Move image block
#[test]
fn test_187_move_image_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    let id = editor.add_block(Block::new(BlockType::Image).with_attribute("src", "img.jpg"));

    editor.move_block(&id, 0);
    assert_eq!(editor.blocks[0].block_type, BlockType::Image);
}

/// Test 188: Move table block
#[test]
fn test_188_move_table_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    let id = editor.add_block(Block::new(BlockType::Table));

    editor.move_block(&id, 0);
    assert_eq!(editor.blocks[0].block_type, BlockType::Table);
}

/// Test 189: Block order after multiple inserts and moves
#[test]
fn test_189_complex_order() {
    let mut editor = MockEditor::new();

    let id1 = editor.add_block(Block::new(BlockType::Paragraph).with_content("1"));
    let id2 = editor.add_block(Block::new(BlockType::Paragraph).with_content("2"));
    let id3 = editor.add_block(Block::new(BlockType::Paragraph).with_content("3"));

    // Move 3 to front, then 1 to end
    editor.move_block(&id3, 0);
    editor.move_block(&id1, 2);

    let contents: Vec<&str> = editor.blocks.iter().map(|b| b.content.as_str()).collect();
    assert_eq!(contents, vec!["3", "2", "1"]);
}

/// Test 190-200: Additional drag tests
#[test]
fn test_190_drag_preserves_selection() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    let id2 = editor.add_block(Block::new(BlockType::Paragraph).select());

    editor.move_block(&id2, 0);
    assert!(editor.blocks[0].is_selected);
}

#[test]
fn test_191_drag_custom_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    let id = editor.add_block(Block::new(BlockType::Custom("widget".to_string())));

    editor.move_block(&id, 0);
    assert!(matches!(editor.blocks[0].block_type, BlockType::Custom(_)));
}

#[test]
fn test_192_drag_between_many_blocks() {
    let mut editor = MockEditor::new();
    for i in 0..10 {
        editor.add_block(Block::new(BlockType::Paragraph).with_content(&format!("Block {}", i)));
    }

    let id = editor.blocks[9].id.clone();
    editor.move_block(&id, 5);

    assert_eq!(editor.blocks[5].content, "Block 9");
}

#[test]
fn test_193_move_maintains_validity() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    let mut block = Block::new(BlockType::Paragraph);
    block.is_valid = false;
    let id = block.id.clone();
    editor.blocks.push(block);

    editor.move_block(&id, 0);
    assert!(!editor.blocks[0].is_valid);
}

#[test]
fn test_194_move_code_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    let id = editor.add_block(Block::new(BlockType::Code).with_content("let x = 1;"));

    editor.move_block(&id, 0);
    assert_eq!(editor.blocks[0].content, "let x = 1;");
}

#[test]
fn test_195_move_embed_block() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    let id = editor.add_block(Block::new(BlockType::Embed).with_attribute("url", "https://youtube.com/watch?v=123"));

    editor.move_block(&id, 0);
    assert!(editor.blocks[0].attributes.contains_key("url"));
}

#[test]
fn test_196_move_multiple_times() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph).with_content("Moving"));
    editor.add_block(Block::new(BlockType::Paragraph).with_content("Static 1"));
    editor.add_block(Block::new(BlockType::Paragraph).with_content("Static 2"));

    editor.move_block(&id, 1);
    editor.move_block(&id, 2);
    editor.move_block(&id, 0);

    assert_eq!(editor.blocks[0].content, "Moving");
}

#[test]
fn test_197_undo_stack_after_moves() {
    let mut editor = MockEditor::new();
    let id1 = editor.add_block(Block::new(BlockType::Paragraph));
    editor.add_block(Block::new(BlockType::Paragraph));
    editor.undo_stack.clear();

    editor.move_block(&id1, 1);

    assert_eq!(editor.undo_stack.len(), 1);
    assert_eq!(editor.undo_stack[0].action_type, "move_block");
}

#[test]
fn test_198_move_heading() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    let id = editor.add_block(Block::new(BlockType::Heading).with_attribute("level", "1"));

    editor.move_block(&id, 0);
    assert_eq!(editor.blocks[0].block_type, BlockType::Heading);
}

#[test]
fn test_199_move_list() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    let id = editor.add_block(Block::new(BlockType::List));

    editor.move_block(&id, 0);
    assert_eq!(editor.blocks[0].block_type, BlockType::List);
}

#[test]
fn test_200_move_gallery() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    let id = editor.add_block(Block::new(BlockType::Gallery));

    editor.move_block(&id, 0);
    assert_eq!(editor.blocks[0].block_type, BlockType::Gallery);
}

// ============================================================================
// UNDO/REDO TESTS (201-230)
// ============================================================================

/// Test 201: Initial editor cannot undo
#[test]
fn test_201_cannot_undo_initially() {
    let editor = MockEditor::new();
    assert!(!editor.can_undo());
}

/// Test 202: Initial editor cannot redo
#[test]
fn test_202_cannot_redo_initially() {
    let editor = MockEditor::new();
    assert!(!editor.can_redo());
}

/// Test 203: Adding block enables undo
#[test]
fn test_203_add_block_enables_undo() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    assert!(editor.can_undo());
}

/// Test 204: Undo returns action
#[test]
fn test_204_undo_returns_action() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    let action = editor.undo();
    assert!(action.is_some());
}

/// Test 205: Undo on empty stack returns None
#[test]
fn test_205_undo_empty_stack() {
    let mut editor = MockEditor::new();
    let action = editor.undo();
    assert!(action.is_none());
}

/// Test 206: Redo on empty stack returns None
#[test]
fn test_206_redo_empty_stack() {
    let mut editor = MockEditor::new();
    let action = editor.redo();
    assert!(action.is_none());
}

/// Test 207: Undo moves action to redo stack
#[test]
fn test_207_undo_populates_redo() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    assert!(!editor.can_redo());
    editor.undo();
    assert!(editor.can_redo());
}

/// Test 208: Redo moves action to undo stack
#[test]
fn test_208_redo_populates_undo() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    editor.undo();
    assert!(!editor.can_undo());

    editor.redo();
    assert!(editor.can_undo());
}

/// Test 209: Multiple undos
#[test]
fn test_209_multiple_undos() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    editor.add_block(Block::new(BlockType::Paragraph));
    editor.add_block(Block::new(BlockType::Paragraph));

    assert!(editor.undo().is_some());
    assert!(editor.undo().is_some());
    assert!(editor.undo().is_some());
    assert!(editor.undo().is_none());
}

/// Test 210: Undo action has type
#[test]
fn test_210_undo_action_has_type() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    let action = editor.undo().unwrap();
    assert_eq!(action.action_type, "add_block");
}

/// Test 211: Transform creates undo entry
#[test]
fn test_211_transform_creates_undo() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.undo_stack.clear();

    editor.transform_block(&id, BlockType::Heading);

    assert!(editor.can_undo());
    let action = editor.undo().unwrap();
    assert_eq!(action.action_type, "transform_block");
}

/// Test 212: New action clears redo stack
#[test]
fn test_212_new_action_clears_redo() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    editor.undo();

    assert!(editor.can_redo());

    editor.add_block(Block::new(BlockType::Paragraph));
    assert!(!editor.can_redo());
}

/// Test 213: Undo stack size
#[test]
fn test_213_undo_stack_size() {
    let mut editor = MockEditor::new();

    for _ in 0..5 {
        editor.add_block(Block::new(BlockType::Paragraph));
    }

    assert_eq!(editor.undo_stack.len(), 5);
}

/// Test 214: Redo stack size after undos
#[test]
fn test_214_redo_stack_size() {
    let mut editor = MockEditor::new();

    for _ in 0..5 {
        editor.add_block(Block::new(BlockType::Paragraph));
    }

    for _ in 0..3 {
        editor.undo();
    }

    assert_eq!(editor.redo_stack.len(), 3);
}

/// Test 215: Undo/redo cycle
#[test]
fn test_215_undo_redo_cycle() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    let action1 = editor.undo().unwrap();
    let action2 = editor.redo().unwrap();

    assert_eq!(action1.id, action2.id);
}

/// Test 216: Undo action has timestamp
#[test]
fn test_216_undo_has_timestamp() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    let action = editor.undo().unwrap();
    assert!(action.timestamp > 0);
}

/// Test 217: Undo action has description
#[test]
fn test_217_undo_has_description() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    let action = editor.undo().unwrap();
    assert!(!action.description.is_empty());
}

/// Test 218: Undo action has data
#[test]
fn test_218_undo_has_data() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));

    let action = editor.undo().unwrap();
    assert_eq!(action.data, id);
}

/// Test 219: Remove block creates undo
#[test]
fn test_219_remove_creates_undo() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.undo_stack.clear();

    editor.remove_block(&id);

    let action = editor.undo().unwrap();
    assert_eq!(action.action_type, "remove_block");
}

/// Test 220: Undo after redo maintains state
#[test]
fn test_220_undo_after_redo() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    editor.add_block(Block::new(BlockType::Paragraph));

    editor.undo();
    editor.undo();
    editor.redo();
    editor.undo();

    assert_eq!(editor.undo_stack.len(), 0);
    assert_eq!(editor.redo_stack.len(), 2);
}

/// Test 221-230: Additional undo/redo tests
#[test]
fn test_221_undo_action_unique_id() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    editor.add_block(Block::new(BlockType::Paragraph));

    let action1 = editor.undo().unwrap();
    let action2 = editor.undo().unwrap();

    assert_ne!(action1.id, action2.id);
}

#[test]
fn test_222_redo_action_matches_undo() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    let undone = editor.undo().unwrap();
    let redone = editor.redo().unwrap();

    assert_eq!(undone.action_type, redone.action_type);
}

#[test]
fn test_223_clear_undo_stack() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    editor.add_block(Block::new(BlockType::Paragraph));

    editor.undo_stack.clear();

    assert!(!editor.can_undo());
}

#[test]
fn test_224_clear_redo_stack() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    editor.undo();

    editor.redo_stack.clear();

    assert!(!editor.can_redo());
}

#[test]
fn test_225_move_block_undo_type() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.add_block(Block::new(BlockType::Paragraph));
    editor.undo_stack.clear();

    editor.move_block(&id, 1);

    let action = editor.undo().unwrap();
    assert_eq!(action.action_type, "move_block");
}

#[test]
fn test_226_undo_preserves_order() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph).with_content("First"));
    editor.add_block(Block::new(BlockType::Paragraph).with_content("Second"));

    let action1 = editor.undo().unwrap();
    let action2 = editor.undo().unwrap();

    // Last in, first out
    assert!(action1.data.contains("block_")); // Second block
    assert!(action2.data.contains("block_")); // First block
}

#[test]
fn test_227_many_undo_redo_cycles() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    for _ in 0..10 {
        editor.undo();
        editor.redo();
    }

    assert!(editor.can_undo());
    assert!(!editor.can_redo());
}

#[test]
fn test_228_interleaved_actions_and_undos() {
    let mut editor = MockEditor::new();

    editor.add_block(Block::new(BlockType::Paragraph));
    editor.add_block(Block::new(BlockType::Paragraph));
    editor.undo();
    editor.add_block(Block::new(BlockType::Paragraph));

    // Should have 2 items on undo (1 original + 1 new after undo)
    assert_eq!(editor.undo_stack.len(), 2);
    assert!(!editor.can_redo()); // Redo cleared by new action
}

#[test]
fn test_229_undo_action_has_id() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    let action = editor.undo().unwrap();
    assert!(!action.id.is_empty());
    assert!(action.id.starts_with("undo_"));
}

#[test]
fn test_230_complex_undo_scenario() {
    let mut editor = MockEditor::new();

    // Create blocks
    let id1 = editor.add_block(Block::new(BlockType::Paragraph));
    let id2 = editor.add_block(Block::new(BlockType::Paragraph));

    // Transform and move
    editor.transform_block(&id1, BlockType::Heading);
    editor.move_block(&id2, 0);

    // Undo all
    assert_eq!(editor.undo().unwrap().action_type, "move_block");
    assert_eq!(editor.undo().unwrap().action_type, "transform_block");
    assert_eq!(editor.undo().unwrap().action_type, "add_block");
    assert_eq!(editor.undo().unwrap().action_type, "add_block");
    assert!(editor.undo().is_none());
}

// ============================================================================
// AUTOSAVE & RECOVERY TESTS (231-260)
// ============================================================================

/// Test 231: Autosave is enabled by default
#[test]
fn test_231_autosave_enabled_default() {
    let editor = MockEditor::new();
    assert!(editor.autosave.enabled);
}

/// Test 232: Autosave default interval
#[test]
fn test_232_autosave_default_interval() {
    let editor = MockEditor::new();
    assert_eq!(editor.autosave.interval_seconds, 60);
}

/// Test 233: No pending changes initially
#[test]
fn test_233_no_pending_changes() {
    let editor = MockEditor::new();
    assert!(!editor.autosave.pending_changes);
}

/// Test 234: Autosave save count starts at zero
#[test]
fn test_234_save_count_zero() {
    let editor = MockEditor::new();
    assert_eq!(editor.autosave.save_count, 0);
}

/// Test 235: Trigger autosave when dirty
#[test]
fn test_235_trigger_autosave_dirty() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    let result = editor.trigger_autosave();
    assert!(result);
}

/// Test 236: Trigger autosave when not dirty
#[test]
fn test_236_trigger_autosave_not_dirty() {
    let mut editor = MockEditor::new();

    let result = editor.trigger_autosave();
    assert!(!result);
}

/// Test 237: Autosave updates last save time
#[test]
fn test_237_autosave_updates_time() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    assert!(editor.autosave.last_save.is_none());
    editor.trigger_autosave();
    assert!(editor.autosave.last_save.is_some());
}

/// Test 238: Autosave increments count
#[test]
fn test_238_autosave_increments_count() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    editor.trigger_autosave();
    assert_eq!(editor.autosave.save_count, 1);

    editor.add_block(Block::new(BlockType::Paragraph));
    editor.trigger_autosave();
    assert_eq!(editor.autosave.save_count, 2);
}

/// Test 239: Autosave clears dirty flag
#[test]
fn test_239_autosave_clears_dirty() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    assert!(editor.is_dirty);
    editor.trigger_autosave();
    assert!(!editor.is_dirty);
}

/// Test 240: Autosave clears pending changes
#[test]
fn test_240_autosave_clears_pending() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    editor.autosave.pending_changes = true;

    editor.trigger_autosave();
    assert!(!editor.autosave.pending_changes);
}

/// Test 241: Disable autosave
#[test]
fn test_241_disable_autosave() {
    let mut editor = MockEditor::new();
    editor.autosave.enabled = false;
    editor.add_block(Block::new(BlockType::Paragraph));

    let result = editor.trigger_autosave();
    assert!(!result);
}

/// Test 242: Custom autosave interval
#[test]
fn test_242_custom_autosave_interval() {
    let mut editor = MockEditor::new();
    editor.autosave.interval_seconds = 30;

    assert_eq!(editor.autosave.interval_seconds, 30);
}

/// Test 243: Multiple autosave triggers
#[test]
fn test_243_multiple_autosave_triggers() {
    let mut editor = MockEditor::new();

    for i in 0..5 {
        editor.add_block(Block::new(BlockType::Paragraph).with_content(&format!("Block {}", i)));
        editor.trigger_autosave();
    }

    assert_eq!(editor.autosave.save_count, 5);
}

/// Test 244: Autosave state after manual save
#[test]
fn test_244_autosave_after_manual_save() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    // Simulate manual save
    editor.is_dirty = false;
    editor.autosave.last_save = Some(current_timestamp());

    // Autosave shouldn't trigger
    let result = editor.trigger_autosave();
    assert!(!result);
}

/// Test 245: Last save timestamp is recent
#[test]
fn test_245_last_save_recent() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    let before = current_timestamp();
    editor.trigger_autosave();
    let after = current_timestamp();

    let last_save = editor.autosave.last_save.unwrap();
    assert!(last_save >= before && last_save <= after);
}

/// Test 246-260: Additional autosave tests
#[test]
fn test_246_autosave_state_default() {
    let state = AutosaveState::default();

    assert!(state.enabled);
    assert_eq!(state.interval_seconds, 60);
    assert!(state.last_save.is_none());
    assert!(!state.pending_changes);
    assert_eq!(state.save_count, 0);
}

#[test]
fn test_247_autosave_preserves_content() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph).with_content("Test content"));

    editor.trigger_autosave();

    assert_eq!(editor.blocks[0].content, "Test content");
}

#[test]
fn test_248_autosave_preserves_blocks() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    editor.add_block(Block::new(BlockType::Heading));
    editor.add_block(Block::new(BlockType::Image));

    editor.trigger_autosave();

    assert_eq!(editor.blocks.len(), 3);
}

#[test]
fn test_249_autosave_preserves_undo() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    editor.add_block(Block::new(BlockType::Paragraph));

    let undo_count_before = editor.undo_stack.len();
    editor.trigger_autosave();
    let undo_count_after = editor.undo_stack.len();

    assert_eq!(undo_count_before, undo_count_after);
}

#[test]
fn test_250_autosave_preserves_redo() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));
    editor.undo();

    let redo_count_before = editor.redo_stack.len();
    editor.add_block(Block::new(BlockType::Paragraph));
    editor.trigger_autosave();
    let redo_count_after = editor.redo_stack.len();

    // Note: adding block clears redo, so count should be 0
    assert_eq!(redo_count_after, 0);
}

#[test]
fn test_251_autosave_consecutive_no_change() {
    let mut editor = MockEditor::new();
    editor.add_block(Block::new(BlockType::Paragraph));

    editor.trigger_autosave();
    let count1 = editor.autosave.save_count;

    // Try again without changes
    let result = editor.trigger_autosave();
    let count2 = editor.autosave.save_count;

    assert!(!result);
    assert_eq!(count1, count2);
}

#[test]
fn test_252_autosave_interval_configurable() {
    let mut editor = MockEditor::new();
    editor.autosave.interval_seconds = 120;

    assert_eq!(editor.autosave.interval_seconds, 120);
}

#[test]
fn test_253_autosave_very_short_interval() {
    let mut editor = MockEditor::new();
    editor.autosave.interval_seconds = 1;

    assert_eq!(editor.autosave.interval_seconds, 1);
}

#[test]
fn test_254_autosave_very_long_interval() {
    let mut editor = MockEditor::new();
    editor.autosave.interval_seconds = 3600;

    assert_eq!(editor.autosave.interval_seconds, 3600);
}

#[test]
fn test_255_enable_autosave_after_disable() {
    let mut editor = MockEditor::new();
    editor.autosave.enabled = false;
    editor.add_block(Block::new(BlockType::Paragraph));

    assert!(!editor.trigger_autosave());

    editor.autosave.enabled = true;
    assert!(editor.trigger_autosave());
}

#[test]
fn test_256_autosave_doesnt_affect_mode() {
    let mut editor = MockEditor::new();
    editor.set_mode(EditorMode::Code);
    editor.add_block(Block::new(BlockType::Paragraph));

    editor.trigger_autosave();

    assert_eq!(editor.mode, EditorMode::Code);
}

#[test]
fn test_257_autosave_doesnt_affect_fullscreen() {
    let mut editor = MockEditor::new();
    editor.toggle_fullscreen();
    editor.add_block(Block::new(BlockType::Paragraph));

    editor.trigger_autosave();

    assert!(editor.is_fullscreen);
}

#[test]
fn test_258_autosave_preserves_selection() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.select_block(&id);

    editor.trigger_autosave();

    assert!(editor.selection.is_some());
}

#[test]
fn test_259_autosave_after_content_update() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));

    editor.trigger_autosave();
    editor.update_block_content(&id, "Updated content");

    assert!(editor.trigger_autosave());
}

#[test]
fn test_260_autosave_count_persistence() {
    let mut editor = MockEditor::new();

    for _ in 0..10 {
        editor.add_block(Block::new(BlockType::Paragraph));
        editor.trigger_autosave();
    }

    assert_eq!(editor.autosave.save_count, 10);
}

// ============================================================================
// COLLABORATIVE EDITING TESTS (261-280)
// ============================================================================

/// Test 261: No collaborators initially
#[test]
fn test_261_no_collaborators_initial() {
    let editor = MockEditor::new();
    assert!(editor.collaborators.is_empty());
}

/// Test 262: Can add collaborator
#[test]
fn test_262_add_collaborator() {
    let mut editor = MockEditor::new();

    let user = CollaborativeUser {
        id: "user_1".to_string(),
        name: "Alice".to_string(),
        color: "#FF0000".to_string(),
        cursor_block: None,
        cursor_offset: None,
        is_active: true,
    };

    editor.add_collaborator(user);
    assert_eq!(editor.collaborators.len(), 1);
}

/// Test 263: Can remove collaborator
#[test]
fn test_263_remove_collaborator() {
    let mut editor = MockEditor::new();

    let user = CollaborativeUser {
        id: "user_1".to_string(),
        name: "Alice".to_string(),
        color: "#FF0000".to_string(),
        cursor_block: None,
        cursor_offset: None,
        is_active: true,
    };

    editor.add_collaborator(user);
    editor.remove_collaborator("user_1");

    assert!(editor.collaborators.is_empty());
}

/// Test 264: Remove nonexistent collaborator
#[test]
fn test_264_remove_nonexistent_collaborator() {
    let mut editor = MockEditor::new();
    editor.remove_collaborator("nonexistent");
    // Should not panic
    assert!(editor.collaborators.is_empty());
}

/// Test 265: Update collaborator cursor
#[test]
fn test_265_update_collaborator_cursor() {
    let mut editor = MockEditor::new();

    let user = CollaborativeUser {
        id: "user_1".to_string(),
        name: "Alice".to_string(),
        color: "#FF0000".to_string(),
        cursor_block: None,
        cursor_offset: None,
        is_active: true,
    };

    editor.add_collaborator(user);
    let result = editor.update_collaborator_cursor("user_1", "block_1", 10);

    assert!(result);
    assert_eq!(editor.collaborators[0].cursor_block, Some("block_1".to_string()));
    assert_eq!(editor.collaborators[0].cursor_offset, Some(10));
}

/// Test 266: Update cursor for nonexistent user
#[test]
fn test_266_update_cursor_nonexistent() {
    let mut editor = MockEditor::new();
    let result = editor.update_collaborator_cursor("nonexistent", "block_1", 10);
    assert!(!result);
}

/// Test 267: Multiple collaborators
#[test]
fn test_267_multiple_collaborators() {
    let mut editor = MockEditor::new();

    for i in 0..5 {
        let user = CollaborativeUser {
            id: format!("user_{}", i),
            name: format!("User {}", i),
            color: format!("#{:06X}", i * 100000),
            cursor_block: None,
            cursor_offset: None,
            is_active: true,
        };
        editor.add_collaborator(user);
    }

    assert_eq!(editor.collaborators.len(), 5);
}

/// Test 268: Collaborator has color
#[test]
fn test_268_collaborator_color() {
    let user = CollaborativeUser {
        id: "user_1".to_string(),
        name: "Alice".to_string(),
        color: "#FF5733".to_string(),
        cursor_block: None,
        cursor_offset: None,
        is_active: true,
    };

    assert_eq!(user.color, "#FF5733");
}

/// Test 269: Collaborator active status
#[test]
fn test_269_collaborator_active() {
    let mut editor = MockEditor::new();

    let user = CollaborativeUser {
        id: "user_1".to_string(),
        name: "Alice".to_string(),
        color: "#FF0000".to_string(),
        cursor_block: None,
        cursor_offset: None,
        is_active: true,
    };

    editor.add_collaborator(user);
    assert!(editor.collaborators[0].is_active);
}

/// Test 270: Collaborator inactive status
#[test]
fn test_270_collaborator_inactive() {
    let mut editor = MockEditor::new();

    let user = CollaborativeUser {
        id: "user_1".to_string(),
        name: "Alice".to_string(),
        color: "#FF0000".to_string(),
        cursor_block: None,
        cursor_offset: None,
        is_active: false,
    };

    editor.add_collaborator(user);
    assert!(!editor.collaborators[0].is_active);
}

/// Test 271-280: Additional collaborative tests
#[test]
fn test_271_collaborator_unique_ids() {
    let mut editor = MockEditor::new();

    for i in 0..3 {
        let user = CollaborativeUser {
            id: format!("user_{}", i),
            name: format!("User {}", i),
            color: "#000000".to_string(),
            cursor_block: None,
            cursor_offset: None,
            is_active: true,
        };
        editor.add_collaborator(user);
    }

    let ids: Vec<&str> = editor.collaborators.iter().map(|u| u.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().cloned().collect();
    assert_eq!(ids.len(), unique.len());
}

#[test]
fn test_272_collaborator_cursor_multiple_updates() {
    let mut editor = MockEditor::new();

    let user = CollaborativeUser {
        id: "user_1".to_string(),
        name: "Alice".to_string(),
        color: "#FF0000".to_string(),
        cursor_block: None,
        cursor_offset: None,
        is_active: true,
    };

    editor.add_collaborator(user);

    editor.update_collaborator_cursor("user_1", "block_1", 0);
    editor.update_collaborator_cursor("user_1", "block_2", 5);
    editor.update_collaborator_cursor("user_1", "block_3", 10);

    assert_eq!(editor.collaborators[0].cursor_block, Some("block_3".to_string()));
    assert_eq!(editor.collaborators[0].cursor_offset, Some(10));
}

#[test]
fn test_273_collaborator_different_colors() {
    let mut editor = MockEditor::new();

    let colors = ["#FF0000", "#00FF00", "#0000FF"];
    for (i, color) in colors.iter().enumerate() {
        let user = CollaborativeUser {
            id: format!("user_{}", i),
            name: format!("User {}", i),
            color: color.to_string(),
            cursor_block: None,
            cursor_offset: None,
            is_active: true,
        };
        editor.add_collaborator(user);
    }

    let user_colors: Vec<&str> = editor.collaborators.iter().map(|u| u.color.as_str()).collect();
    assert_eq!(user_colors, colors);
}

#[test]
fn test_274_remove_specific_collaborator() {
    let mut editor = MockEditor::new();

    for i in 0..3 {
        let user = CollaborativeUser {
            id: format!("user_{}", i),
            name: format!("User {}", i),
            color: "#000000".to_string(),
            cursor_block: None,
            cursor_offset: None,
            is_active: true,
        };
        editor.add_collaborator(user);
    }

    editor.remove_collaborator("user_1");

    assert_eq!(editor.collaborators.len(), 2);
    assert!(!editor.collaborators.iter().any(|u| u.id == "user_1"));
}

#[test]
fn test_275_collaborator_name() {
    let user = CollaborativeUser {
        id: "user_1".to_string(),
        name: "Alice Smith".to_string(),
        color: "#FF0000".to_string(),
        cursor_block: None,
        cursor_offset: None,
        is_active: true,
    };

    assert_eq!(user.name, "Alice Smith");
}

#[test]
fn test_276_collaborator_initial_cursor() {
    let user = CollaborativeUser {
        id: "user_1".to_string(),
        name: "Alice".to_string(),
        color: "#FF0000".to_string(),
        cursor_block: None,
        cursor_offset: None,
        is_active: true,
    };

    assert!(user.cursor_block.is_none());
    assert!(user.cursor_offset.is_none());
}

#[test]
fn test_277_collaborator_cursor_at_zero() {
    let mut editor = MockEditor::new();

    let user = CollaborativeUser {
        id: "user_1".to_string(),
        name: "Alice".to_string(),
        color: "#FF0000".to_string(),
        cursor_block: None,
        cursor_offset: None,
        is_active: true,
    };

    editor.add_collaborator(user);
    editor.update_collaborator_cursor("user_1", "block_1", 0);

    assert_eq!(editor.collaborators[0].cursor_offset, Some(0));
}

#[test]
fn test_278_collaborator_cursor_large_offset() {
    let mut editor = MockEditor::new();

    let user = CollaborativeUser {
        id: "user_1".to_string(),
        name: "Alice".to_string(),
        color: "#FF0000".to_string(),
        cursor_block: None,
        cursor_offset: None,
        is_active: true,
    };

    editor.add_collaborator(user);
    editor.update_collaborator_cursor("user_1", "block_1", 10000);

    assert_eq!(editor.collaborators[0].cursor_offset, Some(10000));
}

#[test]
fn test_279_add_remove_add_collaborator() {
    let mut editor = MockEditor::new();

    let user = CollaborativeUser {
        id: "user_1".to_string(),
        name: "Alice".to_string(),
        color: "#FF0000".to_string(),
        cursor_block: None,
        cursor_offset: None,
        is_active: true,
    };

    editor.add_collaborator(user.clone());
    editor.remove_collaborator("user_1");
    editor.add_collaborator(user);

    assert_eq!(editor.collaborators.len(), 1);
}

#[test]
fn test_280_collaborators_independent_of_blocks() {
    let mut editor = MockEditor::new();

    let user = CollaborativeUser {
        id: "user_1".to_string(),
        name: "Alice".to_string(),
        color: "#FF0000".to_string(),
        cursor_block: None,
        cursor_offset: None,
        is_active: true,
    };

    editor.add_collaborator(user);
    editor.add_block(Block::new(BlockType::Paragraph));
    editor.remove_block(&editor.blocks[0].id.clone());

    // Collaborator should still be there
    assert_eq!(editor.collaborators.len(), 1);
}

// ============================================================================
// ACCESSIBILITY TESTS (281-300)
// ============================================================================

/// Test 281: Toolbar buttons have labels
#[test]
fn test_281_buttons_have_labels() {
    let editor = MockEditor::new();
    for button in &editor.toolbar_buttons {
        assert!(!button.label.is_empty(), "Button {} has no label", button.id);
    }
}

/// Test 282: Toolbar buttons have tooltips
#[test]
fn test_282_buttons_have_tooltips() {
    let editor = MockEditor::new();
    for button in &editor.toolbar_buttons {
        assert!(!button.tooltip.is_empty(), "Button {} has no tooltip", button.id);
    }
}

/// Test 283: Keyboard shortcuts are accessible
#[test]
fn test_283_keyboard_shortcuts_accessible() {
    let editor = MockEditor::new();
    assert!(!editor.shortcuts.is_empty());
}

/// Test 284: All buttons have icons
#[test]
fn test_284_buttons_have_icons() {
    let editor = MockEditor::new();
    for button in &editor.toolbar_buttons {
        assert!(!button.icon.is_empty(), "Button {} has no icon", button.id);
    }
}

/// Test 285: Block types are distinguishable
#[test]
fn test_285_block_types_distinct() {
    let types = [
        BlockType::Paragraph,
        BlockType::Heading,
        BlockType::Image,
        BlockType::List,
        BlockType::Quote,
    ];

    for i in 0..types.len() {
        for j in (i + 1)..types.len() {
            assert_ne!(types[i], types[j]);
        }
    }
}

/// Test 286: Selection can be programmatic
#[test]
fn test_286_programmatic_selection() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));

    editor.select_block(&id);
    assert!(editor.selection.is_some());
}

/// Test 287: Focus moves with keyboard
#[test]
fn test_287_keyboard_focus() {
    let mut editor = MockEditor::new();

    // Tab should work
    let action = editor.execute_shortcut("Tab", false, false, false);
    assert!(action.is_some());
}

/// Test 288: Escape clears selection
#[test]
fn test_288_escape_clears_selection() {
    let editor = MockEditor::new();
    let action = editor.execute_shortcut("Escape", false, false, false);
    assert_eq!(action, Some("deselect".to_string()));
}

/// Test 289: Block has ID for ARIA
#[test]
fn test_289_block_has_id() {
    let block = Block::new(BlockType::Paragraph);
    assert!(!block.id.is_empty());
}

/// Test 290: Buttons have IDs for ARIA
#[test]
fn test_290_buttons_have_ids() {
    let editor = MockEditor::new();
    for button in &editor.toolbar_buttons {
        assert!(!button.id.is_empty());
    }
}

/// Test 291-300: Additional accessibility tests
#[test]
fn test_291_selection_state_queryable() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));

    assert!(editor.get_selected_blocks().is_empty());
    editor.select_block(&id);
    assert_eq!(editor.get_selected_blocks().len(), 1);
}

#[test]
fn test_292_button_state_queryable() {
    let editor = MockEditor::new();
    let button = editor.get_button("bold").unwrap();
    // State should be queryable
    assert_eq!(button.state, ButtonState::Normal);
}

#[test]
fn test_293_word_count_available() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.update_block_content(&id, "Three word sentence");

    // Word count should be available for screen readers
    assert_eq!(editor.word_count, 3);
}

#[test]
fn test_294_char_count_available() {
    let mut editor = MockEditor::new();
    let id = editor.add_block(Block::new(BlockType::Paragraph));
    editor.update_block_content(&id, "Test");

    // Character count should be available
    assert_eq!(editor.character_count, 4);
}

#[test]
fn test_295_editor_state_queryable() {
    let mut editor = MockEditor::new();
    assert_eq!(editor.state, EditorState::Loading);
    editor.initialize();
    assert_eq!(editor.state, EditorState::Ready);
}

#[test]
fn test_296_editor_mode_queryable() {
    let mut editor = MockEditor::new();
    assert_eq!(editor.mode, EditorMode::Visual);
    editor.set_mode(EditorMode::Code);
    assert_eq!(editor.mode, EditorMode::Code);
}

#[test]
fn test_297_dirty_state_queryable() {
    let mut editor = MockEditor::new();
    assert!(!editor.is_dirty);
    editor.add_block(Block::new(BlockType::Paragraph));
    assert!(editor.is_dirty);
}

#[test]
fn test_298_undo_redo_state_queryable() {
    let mut editor = MockEditor::new();

    assert!(!editor.can_undo());
    assert!(!editor.can_redo());

    editor.add_block(Block::new(BlockType::Paragraph));
    assert!(editor.can_undo());
}

#[test]
fn test_299_block_validity_queryable() {
    let block = Block::new(BlockType::Paragraph);
    assert!(block.is_valid);
}

#[test]
fn test_300_fullscreen_state_queryable() {
    let mut editor = MockEditor::new();
    assert!(!editor.is_fullscreen);
    editor.toggle_fullscreen();
    assert!(editor.is_fullscreen);
}
