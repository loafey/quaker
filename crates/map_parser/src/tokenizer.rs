#[derive(Clone)]
pub enum Token {
    LParan,
    RParan,
    LBrack,
    RBrack,
    LSquare,
    RSquare,
    Number(String),
    String(String),
    Texture(String),
}
impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Texture(arg0) => write!(f, "{arg0}"),
            Token::LParan => write!(f, "("),
            Token::RParan => write!(f, ")"),
            Token::LBrack => write!(f, "{{"),
            Token::RBrack => write!(f, "}}"),
            Token::LSquare => write!(f, "]"),
            Token::RSquare => write!(f, "["),
            Token::Number(str) => write!(f, "{str}"),
            Token::String(str) => write!(f, "{str}"),
        }
    }
}
impl From<String> for Token {
    fn from(value: String) -> Self {
        match &value[..] {
            "(" => Self::LParan,
            ")" => Self::RParan,
            "{" => Self::LBrack,
            "}" => Self::RBrack,
            "]" => Self::RSquare,
            "[" => Self::LSquare,
            x if x.starts_with('"') && x.ends_with('"') => Self::String(value),
            x if x.chars().all(|c| c.is_ascii_digit())
                || (x.starts_with('-') && x[1..].chars().all(|c| c.is_ascii_digit())) =>
            {
                Self::Number(value)
            }
            _ => Self::Texture(value),
        }
    }
}

enum LexState {
    Normal,
    InString,
    AlmostInComment,
    InComment,
}

pub fn tokenizer(str: &str) -> Vec<Token> {
    use LexState::*;
    let mut toks = Vec::new();

    let mut state = LexState::Normal;
    let mut curr = String::new();

    for c in str.chars() {
        match c {
            // Comments
            '\n' if matches!(state, InComment) => state = Normal,
            _ if matches!(state, InComment) => {}
            '/' if !matches!(state, InComment | AlmostInComment) => {
                state = AlmostInComment;
                if !curr.is_empty() {
                    toks.push(curr);
                }
                curr = '/'.to_string();
            }
            '/' if matches!(state, AlmostInComment) => {
                state = InComment;
                curr = String::new();
            }

            // Spaces
            ' ' | '\t' if !matches!(state, InString) => {
                if !curr.is_empty() {
                    toks.push(curr);
                }
                curr = String::new();
            }
            '\n' if matches!(state, InString) => curr.push(c),
            '\n' => {}

            // Strings
            '"' if matches!(state, Normal) => {
                state = InString;
                curr.push(c);
            }
            '"' if matches!(state, InString) => {
                state = Normal;
                curr.push(c);
                toks.push(curr);
                curr = String::new();
            }

            // Blocks
            '{' | '}' | '(' | ')' if matches!(state, Normal | AlmostInComment) => {
                state = Normal;
                if !curr.is_empty() {
                    toks.push(curr);
                }
                toks.push(c.to_string());
                curr = String::new();
            }

            // Rest
            c => curr.push(c),
        }
    }
    if !curr.is_empty() {
        toks.push(curr);
    }

    toks.into_iter().map(|s| s.into()).collect()
}
