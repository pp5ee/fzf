use crate::options::Options;
use crate::terminal::Terminal;
use std::collections::HashMap;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    // Navigation
    CursorDown,
    CursorUp,
    CursorLeft,
    CursorRight,
    CursorHome,
    CursorEnd,
    CursorPageUp,
    CursorPageDown,

    // Selection
    Accept,
    AcceptNonEmpty,
    Abort,
    Cancel,
    Toggle,
    ToggleAll,
    ToggleIn,
    ToggleOut,
    SelectAll,
    DeselectAll,

    // Input editing
    DeleteChar,
    DeleteCharEOF,
    DeleteWord,
    DeleteWordBack,
    DeleteLine,
    DeleteLineEnd,
    ClearScreen,
    ClearQuery,
    ClearSelection,
    BackwardWord,
    ForwardWord,
    BeginningOfLine,
    EndOfLine,
    KillLine,
    KillWord,
    UnixWordRubout,
    UnixLineDiscard,
    Yank,

    // Preview
    TogglePreview,
    TogglePreviewWrap,
    PreviewUp,
    PreviewDown,
    PreviewPageUp,
    PreviewPageDown,
    PreviewHalfPageUp,
    PreviewHalfPageDown,

    // Search
    ToggleSearch,
    ToggleMatch,
    ToggleSort,
    ShowInput,
    HideInput,
    ToggleInput,

    // Modes
    EnableMulti,
    DisableMulti,
    ToggleMulti,
    ToggleSync,
    ToggleTrack,

    // History
    PreviousHistory,
    NextHistory,

    // Other
    Ignore,
    Execute(String),
    ExecuteSilent(String),
    ReplaceQuery,
    ChangeQuery(String),
    TransformQuery(String),
    ChangePrompt(String),
    ChangeNth(String),
    Reload(String),
    Unbind(String),
    Rebind(String),
    Become(String),

    // Mouse actions
    Mouse,
    ToggleMouse,

    // Page navigation
    PageUp,
    PageDown,
    HalfPageUp,
    HalfPageDown,

    // Jump
    Jump,
    JumpAccept,
}

impl Action {
    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        let action_name = parts[0];
        let action_arg = parts.get(1).map(|s| s.to_string());

        match action_name {
            // Navigation
            "accept" => Some(Self::Accept),
            "accept-non-empty" => Some(Self::AcceptNonEmpty),
            "abort" => Some(Self::Abort),
            "cancel" => Some(Self::Cancel),
            "down" | "cursor-down" => Some(Self::CursorDown),
            "up" | "cursor-up" => Some(Self::CursorUp),
            "left" | "cursor-left" => Some(Self::CursorLeft),
            "right" | "cursor-right" => Some(Self::CursorRight),
            "home" | "beginning-of-line" => Some(Self::CursorHome),
            "end" | "end-of-line" => Some(Self::CursorEnd),
            "page-up" => Some(Self::CursorPageUp),
            "page-down" => Some(Self::CursorPageDown),
            "half-page-up" => Some(Self::HalfPageUp),
            "half-page-down" => Some(Self::HalfPageDown),

            // Selection
            "toggle" => Some(Self::Toggle),
            "toggle-all" => Some(Self::ToggleAll),
            "toggle-in" => Some(Self::ToggleIn),
            "toggle-out" => Some(Self::ToggleOut),
            "select-all" => Some(Self::SelectAll),
            "deselect-all" => Some(Self::DeselectAll),

            // Input editing
            "delete-char" => Some(Self::DeleteChar),
            "delete-char-eof" => Some(Self::DeleteCharEOF),
            "delete-word" => Some(Self::DeleteWord),
            "delete-word-back" => Some(Self::DeleteWordBack),
            "delete-line" => Some(Self::DeleteLine),
            "delete-line-end" => Some(Self::DeleteLineEnd),
            "clear-screen" => Some(Self::ClearScreen),
            "clear-query" => Some(Self::ClearQuery),
            "clear-selection" => Some(Self::ClearSelection),
            "backward-word" => Some(Self::BackwardWord),
            "forward-word" => Some(Self::ForwardWord),
            "unix-word-rubout" => Some(Self::UnixWordRubout),
            "unix-line-discard" => Some(Self::UnixLineDiscard),
            "kill-line" => Some(Self::KillLine),
            "kill-word" => Some(Self::KillWord),
            "yank" => Some(Self::Yank),

            // Preview
            "toggle-preview" => Some(Self::TogglePreview),
            "toggle-preview-wrap" => Some(Self::TogglePreviewWrap),
            "preview-up" => Some(Self::PreviewUp),
            "preview-down" => Some(Self::PreviewDown),
            "preview-page-up" => Some(Self::PreviewPageUp),
            "preview-page-down" => Some(Self::PreviewPageDown),
            "preview-half-page-up" => Some(Self::PreviewHalfPageUp),
            "preview-half-page-down" => Some(Self::PreviewHalfPageDown),

            // Search
            "toggle-search" => Some(Self::ToggleSearch),
            "toggle-match" => Some(Self::ToggleMatch),
            "toggle-sort" => Some(Self::ToggleSort),
            "show-input" => Some(Self::ShowInput),
            "hide-input" => Some(Self::HideInput),
            "toggle-input" => Some(Self::ToggleInput),

            // Modes
            "enable-multi" => Some(Self::EnableMulti),
            "disable-multi" => Some(Self::DisableMulti),
            "toggle-multi" => Some(Self::ToggleMulti),
            "toggle-sync" => Some(Self::ToggleSync),
            "toggle-track" => Some(Self::ToggleTrack),

            // History
            "previous-history" => Some(Self::PreviousHistory),
            "next-history" => Some(Self::NextHistory),

            // Other
            "ignore" => Some(Self::Ignore),
            "mouse" => Some(Self::Mouse),
            "toggle-mouse" => Some(Self::ToggleMouse),

            // Actions with arguments
            "execute" => action_arg.map(Self::Execute),
            "execute-silent" => action_arg.map(Self::ExecuteSilent),
            "change-query" => action_arg.map(Self::ChangeQuery),
            "transform-query" => action_arg.map(Self::TransformQuery),
            "change-prompt" => action_arg.map(Self::ChangePrompt),
            "change-nth" => action_arg.map(Self::ChangeNth),
            "reload" => action_arg.map(Self::Reload),
            "unbind" => action_arg.map(Self::Unbind),
            "rebind" => action_arg.map(Self::Rebind),
            "become" => action_arg.map(Self::Become),
            "replace-query" => Some(Self::ReplaceQuery),

            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Accept => "accept".to_string(),
            Self::AcceptNonEmpty => "accept-non-empty".to_string(),
            Self::Abort => "abort".to_string(),
            Self::Cancel => "cancel".to_string(),
            Self::CursorDown => "down".to_string(),
            Self::CursorUp => "up".to_string(),
            Self::CursorLeft => "left".to_string(),
            Self::CursorRight => "right".to_string(),
            Self::CursorHome => "home".to_string(),
            Self::CursorEnd => "end".to_string(),
            Self::CursorPageUp => "page-up".to_string(),
            Self::CursorPageDown => "page-down".to_string(),
            Self::Toggle => "toggle".to_string(),
            Self::ToggleAll => "toggle-all".to_string(),
            Self::ToggleIn => "toggle-in".to_string(),
            Self::ToggleOut => "toggle-out".to_string(),
            Self::SelectAll => "select-all".to_string(),
            Self::DeselectAll => "deselect-all".to_string(),
            Self::DeleteChar => "delete-char".to_string(),
            Self::ClearScreen => "clear-screen".to_string(),
            Self::ClearQuery => "clear-query".to_string(),
            Self::ClearSelection => "clear-selection".to_string(),
            Self::TogglePreview => "toggle-preview".to_string(),
            Self::ToggleSort => "toggle-sort".to_string(),
            Self::PreviousHistory => "previous-history".to_string(),
            Self::NextHistory => "next-history".to_string(),
            Self::Ignore => "ignore".to_string(),
            Self::Execute(cmd) => format!("execute:{}", cmd),
            Self::ExecuteSilent(cmd) => format!("execute-silent:{}", cmd),
            Self::ReplaceQuery => "replace-query".to_string(),
            Self::ChangeQuery(q) => format!("change-query:{}", q),
            Self::Reload(cmd) => format!("reload:{}", cmd),
            _ => "unknown".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub key: String,
    pub action: Action,
}

pub struct ActionHandler {
    bindings: HashMap<String, Vec<Action>>,
}

impl ActionHandler {
    pub fn new() -> Self {
        let mut handler = Self {
            bindings: HashMap::new(),
        };
        handler.setup_default_bindings();
        handler
    }

    fn setup_default_bindings(&mut self) {
        // Navigation
        self.bind("ctrl-n", Action::CursorDown);
        self.bind("ctrl-p", Action::CursorUp);
        self.bind("ctrl-j", Action::CursorDown);
        self.bind("ctrl-k", Action::CursorUp);
        self.bind("down", Action::CursorDown);
        self.bind("up", Action::CursorUp);
        self.bind("left", Action::CursorLeft);
        self.bind("right", Action::CursorRight);
        self.bind("ctrl-a", Action::CursorHome);
        self.bind("ctrl-e", Action::CursorEnd);
        self.bind("home", Action::CursorHome);
        self.bind("end", Action::CursorEnd);
        self.bind("page-up", Action::CursorPageUp);
        self.bind("page-down", Action::CursorPageDown);

        // Selection
        self.bind("enter", Action::Accept);
        self.bind("ctrl-m", Action::Accept);
        self.bind("ctrl-c", Action::Abort);
        self.bind("ctrl-g", Action::Abort);
        self.bind("esc", Action::Abort);
        self.bind("tab", Action::Toggle);
        self.bind("btab", Action::Toggle);
        self.bind("ctrl-i", Action::Toggle);

        // Editing
        self.bind("ctrl-u", Action::ClearQuery);
        self.bind("ctrl-w", Action::DeleteWordBack);
        self.bind("ctrl-h", Action::DeleteChar);
        self.bind("backspace", Action::DeleteChar);
        self.bind("delete", Action::DeleteChar);
        self.bind("ctrl-d", Action::DeleteCharEOF);
        self.bind("ctrl-b", Action::CursorLeft);
        self.bind("ctrl-f", Action::CursorRight);
        self.bind("alt-b", Action::BackwardWord);
        self.bind("alt-f", Action::ForwardWord);
        self.bind("alt-d", Action::DeleteWord);

        // Preview
        self.bind("ctrl-+", Action::TogglePreview);
        self.bind("ctrl-/_", Action::TogglePreview);

        // Other
        self.bind("ctrl-l", Action::ClearScreen);
        self.bind("ctrl-r", Action::ToggleSort);
    }

    pub fn bind(&mut self, key: &str, action: Action) {
        let key = key.to_lowercase();
        self.bindings.entry(key).or_default().push(action);
    }

    pub fn unbind(&mut self, key: &str) {
        self.bindings.remove(&key.to_lowercase());
    }

    pub fn get_actions(&self, key: &str) -> Option<&Vec<Action>> {
        self.bindings.get(&key.to_lowercase())
    }

    pub fn parse_bindings(bind_str: &str) -> Vec<KeyBinding> {
        let mut bindings = Vec::new();

        for part in bind_str.split(',') {
            let binding_str = part.trim();
            if binding_str.is_empty() {
                continue;
            }

            // Format: key:action or key:action:arg
            let parts: Vec<&str> = binding_str.splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let action_str = parts[1].trim();

                if let Some(action) = Action::from_string(action_str) {
                    bindings.push(KeyBinding { key, action });
                }
            }
        }

        bindings
    }

    pub fn apply_bindings(&mut self, bindings: &[KeyBinding]) {
        for binding in bindings {
            self.bind(&binding.key, binding.action.clone());
        }
    }
}

impl Default for ActionHandler {
    fn default() -> Self {
        Self::new()
    }
}

pub fn execute_action(action: &Action, terminal: &mut Terminal, options: &mut Options) -> Result<(), String> {
    match action {
        Action::Execute(cmd) => {
            // Execute command synchronously
            let _ = execute_command(cmd);
            Ok(())
        }
        Action::ExecuteSilent(cmd) => {
            // Execute command silently in background
            let _ = execute_command_silent(cmd);
            Ok(())
        }
        Action::ChangeQuery(query) => {
            // Would update terminal query
            Ok(())
        }
        Action::ChangePrompt(prompt) => {
            options.prompt = prompt.clone();
            Ok(())
        }
        Action::TogglePreview => {
            // Would toggle preview visibility
            Ok(())
        }
        Action::ToggleSort => {
            options.no_sort = !options.no_sort;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn execute_command(cmd: &str) -> Result<(), String> {
    use std::process::Command;

    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", cmd])
            .status()
            .map_err(|e| e.to_string())?
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()
            .map_err(|e| e.to_string())?
    };

    if status.success() {
        Ok(())
    } else {
        Err(format!("Command exited with status: {}", status))
    }
}

fn execute_command_silent(cmd: &str) -> Result<(), String> {
    use std::process::{Command, Stdio};
    use std::thread;

    let cmd = cmd.to_string();
    thread::spawn(move || {
        let _ = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &cmd])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
        };
    });

    Ok(())
}
