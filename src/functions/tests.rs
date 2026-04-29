#[cfg(test)]
mod tests {
    use crate::functions::{Action, ActionHandler, Action::*};

    #[test]
    fn test_action_from_string_accept() {
        assert_eq!(Action::from_string("accept"), Some(Accept));
    }

    #[test]
    fn test_action_from_string_abort() {
        assert_eq!(Action::from_string("abort"), Some(Abort));
    }

    #[test]
    fn test_action_from_string_execute() {
        let action = Action::from_string("execute:echo hello");
        assert!(matches!(action, Some(Execute(cmd)) if cmd == "echo hello"));
    }

    #[test]
    fn test_action_from_string_execute_silent() {
        let action = Action::from_string("execute-silent:notify-send test");
        assert!(matches!(action, Some(ExecuteSilent(cmd)) if cmd == "notify-send test"));
    }

    #[test]
    fn test_action_from_string_change_query() {
        let action = Action::from_string("change-query:test");
        assert!(matches!(action, Some(ChangeQuery(q)) if q == "test"));
    }

    #[test]
    fn test_action_from_string_reload() {
        let action = Action::from_string("reload:find .");
        assert!(matches!(action, Some(Reload(cmd)) if cmd == "find ."));
    }

    #[test]
    fn test_action_from_string_invalid() {
        assert_eq!(Action::from_string("invalid_action"), None);
    }

    #[test]
    fn test_action_to_string() {
        assert_eq!(Accept.to_string(), "accept");
        assert_eq!(Abort.to_string(), "abort");
        assert_eq!(CursorDown.to_string(), "down");
        assert_eq!(Execute("test".to_string()).to_string(), "execute:test");
    }

    #[test]
    fn test_action_handler_default_bindings() {
        let handler = ActionHandler::new();

        // Check some default bindings exist
        assert!(handler.get_actions("enter").is_some());
        assert!(handler.get_actions("ctrl-c").is_some());
        assert!(handler.get_actions("esc").is_some());
        assert!(handler.get_actions("tab").is_some());
        assert!(handler.get_actions("up").is_some());
        assert!(handler.get_actions("down").is_some());
    }

    #[test]
    fn test_action_handler_bind() {
        let mut handler = ActionHandler::new();
        handler.bind("ctrl-x", Action::Abort);

        let actions = handler.get_actions("ctrl-x");
        assert!(actions.is_some());
        let actions_vec = actions.unwrap();
        assert!(actions_vec.iter().any(|a| matches!(a, Action::Abort)));
    }

    #[test]
    fn test_action_handler_unbind() {
        let mut handler = ActionHandler::new();
        handler.unbind("enter");

        assert!(handler.get_actions("enter").is_none());
    }

    #[test]
    fn test_parse_bindings() {
        let bindings = ActionHandler::parse_bindings("ctrl-a:select-all,ctrl-d:deselect-all");

        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].key, "ctrl-a");
        assert!(matches!(bindings[0].action, SelectAll));
        assert_eq!(bindings[1].key, "ctrl-d");
        assert!(matches!(bindings[1].action, DeselectAll));
    }

    #[test]
    fn test_parse_bindings_with_args() {
        let bindings = ActionHandler::parse_bindings("ctrl-r:reload:find .");

        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].key, "ctrl-r");
        assert!(matches!(&bindings[0].action, Reload(cmd) if cmd == "find ."));
    }

    #[test]
    fn test_parse_bindings_empty() {
        let bindings = ActionHandler::parse_bindings("");
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_action_navigation_variants() {
        // Test cursor synonyms
        assert_eq!(Action::from_string("cursor-down"), Some(CursorDown));
        assert_eq!(Action::from_string("cursor-up"), Some(CursorUp));
        assert_eq!(Action::from_string("beginning-of-line"), Some(CursorHome));
        assert_eq!(Action::from_string("end-of-line"), Some(CursorEnd));
    }

    #[test]
    fn test_action_preview_variants() {
        assert_eq!(Action::from_string("toggle-preview"), Some(TogglePreview));
        assert_eq!(Action::from_string("preview-up"), Some(PreviewUp));
        assert_eq!(Action::from_string("preview-down"), Some(PreviewDown));
    }

    #[test]
    fn test_action_modes() {
        assert_eq!(Action::from_string("toggle-sort"), Some(ToggleSort));
        assert_eq!(Action::from_string("toggle-search"), Some(ToggleSearch));
        assert_eq!(Action::from_string("toggle-input"), Some(ToggleInput));
        assert_eq!(Action::from_string("toggle-multi"), Some(ToggleMulti));
    }

    #[test]
    fn test_action_editing() {
        assert_eq!(Action::from_string("delete-char"), Some(DeleteChar));
        assert_eq!(Action::from_string("delete-word"), Some(DeleteWord));
        assert_eq!(Action::from_string("clear-query"), Some(ClearQuery));
        assert_eq!(Action::from_string("clear-screen"), Some(ClearScreen));
    }
}
