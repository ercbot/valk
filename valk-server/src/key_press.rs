use enigo::Key;
use std::str::FromStr;

#[derive(Debug)]
pub struct KeyPress {
    pub modifiers: Vec<Key>,
    pub key: Key,
}

impl FromStr for KeyPress {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('+').collect();
        let mut modifiers = Vec::new();

        // For single key press with no modifiers
        if parts.len() == 1 {
            return Ok(KeyPress {
                modifiers: vec![],
                key: parse_single_key(parts[0])?,
            });
        }

        // Handle modifier + key combination
        for &part in &parts[..parts.len() - 1] {
            let modifier = match part.to_lowercase().as_str() {
                "ctrl" | "control" => Key::Control,
                "alt" => Key::Alt,
                "shift" => Key::Shift,
                "super" | "win" | "windows" | "command" => Key::Meta,
                _ => return Err(format!("Unknown modifier: {}", part)),
            };
            modifiers.push(modifier);
        }

        Ok(KeyPress {
            modifiers,
            key: parse_single_key(parts[parts.len() - 1])?,
        })
    }
}

fn parse_single_key(key: &str) -> Result<Key, String> {
    match key.to_lowercase().as_str() {
        // Special keys
        "esc" | "escape" => Ok(Key::Escape),
        "return" | "enter" => Ok(Key::Return),
        "tab" => Ok(Key::Tab),
        "space" => Ok(Key::Space),
        "backspace" => Ok(Key::Backspace),
        "up" => Ok(Key::UpArrow),
        "down" => Ok(Key::DownArrow),
        "left" => Ok(Key::LeftArrow),
        "right" => Ok(Key::RightArrow),
        "delete" => Ok(Key::Delete),
        "insert" => Ok(Key::Insert),
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        "pageup" => Ok(Key::PageUp),
        "pagedown" => Ok(Key::PageDown),
        "printscreen" => Ok(Key::PrintScr),
        "pause" => Ok(Key::Pause),
        "numlock" => Ok(Key::Numlock),
        "capslock" => Ok(Key::CapsLock),

        // Modifiers
        "ctrl" | "control" => Ok(Key::Control),
        "alt" => Ok(Key::Alt),
        "shift" => Ok(Key::Shift),
        "super" | "win" | "windows" | "command" => Ok(Key::Meta),

        // Function keys
        "f1" => Ok(Key::F1),
        "f2" => Ok(Key::F2),
        "f3" => Ok(Key::F3),
        "f4" => Ok(Key::F4),
        "f5" => Ok(Key::F5),
        "f6" => Ok(Key::F6),
        "f7" => Ok(Key::F7),
        "f8" => Ok(Key::F8),
        "f9" => Ok(Key::F9),
        "f10" => Ok(Key::F10),
        "f11" => Ok(Key::F11),
        "f12" => Ok(Key::F12),

        // Numpad keys (doesn't appear enigo handles these so just mapping them to unicode numbers)
        "kp_0" => Ok(Key::Unicode('0')),
        "kp_1" => Ok(Key::Unicode('1')),
        "kp_2" => Ok(Key::Unicode('2')),
        "kp_3" => Ok(Key::Unicode('3')),
        "kp_4" => Ok(Key::Unicode('4')),
        "kp_5" => Ok(Key::Unicode('5')),
        "kp_6" => Ok(Key::Unicode('6')),
        "kp_7" => Ok(Key::Unicode('7')),
        "kp_8" => Ok(Key::Unicode('8')),
        "kp_9" => Ok(Key::Unicode('9')),

        // Default case for Unicode characters
        _ => {
            if key.len() == 1 {
                Ok(Key::Unicode(key.chars().next().ok_or("Invalid key {")?))
            } else {
                Err("Invalid key".to_string())
            }
        }
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_key() {
        let key = KeyPress::from_str("a").unwrap();
        assert_eq!(key.modifiers.len(), 0);
        assert!(matches!(key.key, Key::Unicode('a')));

        let key = KeyPress::from_str("return").unwrap();
        assert_eq!(key.modifiers.len(), 0);
        assert!(matches!(key.key, Key::Return));
    }

    #[test]
    fn test_with_single_modifier() {
        let key = KeyPress::from_str("ctrl+a").unwrap();
        assert_eq!(key.modifiers.len(), 1);
        assert!(matches!(key.modifiers[0], Key::Control));
        assert!(matches!(key.key, Key::Unicode('a')));

        let key = KeyPress::from_str("shift+return").unwrap();
        assert_eq!(key.modifiers.len(), 1);
        assert!(matches!(key.modifiers[0], Key::Shift));
        assert!(matches!(key.key, Key::Return));
    }

    #[test]
    fn test_with_multiple_modifiers() {
        let key = KeyPress::from_str("ctrl+alt+shift+a").unwrap();
        assert_eq!(key.modifiers.len(), 3);
        assert!(matches!(key.modifiers[0], Key::Control));
        assert!(matches!(key.modifiers[1], Key::Alt));
        assert!(matches!(key.modifiers[2], Key::Shift));
        assert!(matches!(key.key, Key::Unicode('a')));
    }

    #[test]
    fn test_function_keys() {
        let key = KeyPress::from_str("f1").unwrap();
        assert_eq!(key.modifiers.len(), 0);
        assert!(matches!(key.key, Key::F1));

        let key = KeyPress::from_str("ctrl+f12").unwrap();
        assert_eq!(key.modifiers.len(), 1);
        assert!(matches!(key.modifiers[0], Key::Control));
        assert!(matches!(key.key, Key::F12));
    }

    #[test]
    fn test_special_keys() {
        let keys = vec![
            ("tab", Key::Tab),
            ("space", Key::Space),
            ("backspace", Key::Backspace),
            ("up", Key::UpArrow),
            ("down", Key::DownArrow),
            ("left", Key::LeftArrow),
            ("right", Key::RightArrow),
        ];

        for (input, expected) in keys {
            let key = KeyPress::from_str(input).unwrap();
            assert_eq!(key.modifiers.len(), 0);
            assert!(
                matches!(key.key, ref e if std::mem::discriminant(e) == std::mem::discriminant(&expected))
            );
        }
    }

    #[test]
    fn test_numpad_keys() {
        for i in 0..10 {
            let key = KeyPress::from_str(&format!("kp_{}", i)).unwrap();
            assert_eq!(key.modifiers.len(), 0);
            assert!(matches!(key.key, Key::Unicode(c) if c == char::from_digit(i, 10).unwrap()));
        }
    }

    #[test]
    fn test_case_insensitivity() {
        let input = "CTRL+ALT+SHIFT+A";
        let key = KeyPress::from_str(input)
            .unwrap_or_else(|e| panic!("Failed to parse '{}': {}", input, e));

        assert_eq!(
            key.modifiers.len(),
            3,
            "Expected 3 modifiers for '{}', got {}",
            input,
            key.modifiers.len()
        );

        assert!(
            matches!(key.modifiers[0], Key::Control),
            "First modifier for '{}' should be Control, got {:?}",
            input,
            key.modifiers[0]
        );
        assert!(
            matches!(key.modifiers[1], Key::Alt),
            "Second modifier for '{}' should be Alt, got {:?}",
            input,
            key.modifiers[1]
        );
        assert!(
            matches!(key.modifiers[2], Key::Shift),
            "Third modifier for '{}' should be Shift, got {:?}",
            input,
            key.modifiers[2]
        );

        if let Key::Unicode(c) = key.key {
            assert_eq!(c, 'A', "Key should be 'A', got '{}'", c);
        } else {
            panic!("Expected Unicode key 'A', got {:?}", key.key);
        }
    }

    #[test]
    fn test_alternative_modifier_names() {
        let modifiers = vec![
            ("control+a", Key::Control),
            ("win+a", Key::Meta),
            ("windows+a", Key::Meta),
            ("command+a", Key::Meta),
            ("super+a", Key::Meta),
        ];

        for (input, expected) in modifiers {
            let key = KeyPress::from_str(input).unwrap();
            assert_eq!(key.modifiers.len(), 1);
            assert!(
                matches!(key.modifiers[0], ref e if std::mem::discriminant(e) == std::mem::discriminant(&expected))
            );
            assert!(matches!(key.key, Key::Unicode('a')));
        }
    }

    #[test]
    fn test_invalid_inputs() {
        let test_cases = vec![
            ("", "empty string"),
            ("+", "just a separator"),
            ("ctrl+", "missing key"),
            ("invalid+a", "invalid modifier"),
            ("ctrl+invalid", "invalid key"),
            ("ctrl++a", "double separator"),
        ];

        for (input, description) in test_cases {
            match KeyPress::from_str(input) {
                Ok(key) => panic!(
                    "Expected error for {} ('{}'), but got successful parse: {:?}",
                    description, input, key
                ),
                Err(e) => println!(
                    "Got expected error for {} ('{}'): {}",
                    description, input, e
                ),
            };
        }
    }
}
