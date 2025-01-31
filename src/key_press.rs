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
        "return" | "enter" => Ok(Key::Return),
        "tab" => Ok(Key::Tab),
        "space" => Ok(Key::Space),
        "backspace" => Ok(Key::Backspace),
        "up" => Ok(Key::UpArrow),
        "down" => Ok(Key::DownArrow),
        "left" => Ok(Key::LeftArrow),
        "right" => Ok(Key::RightArrow),
        
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
        _ => Ok(Key::Unicode(key.chars().next().ok_or("Invalid key {")?)),
    }
}
