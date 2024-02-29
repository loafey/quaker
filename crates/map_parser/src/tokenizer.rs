#[derive(Clone)]
pub enum Symbol {
    /// (
    LParan,
    /// )
    RParan,
    /// {
    LBrack,
    /// }
    RBrack,
    /// [
    LSquare,
    /// ]
    RSquare,
    /// Negative or positive number
    Number(String),
    /// Just a string
    String(String),
    /// A texture
    Texture(String),
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::Texture(arg0) => write!(f, "{arg0}"),
            Symbol::LParan => write!(f, "("),
            Symbol::RParan => write!(f, ")"),
            Symbol::LBrack => write!(f, "{{"),
            Symbol::RBrack => write!(f, "}}"),
            Symbol::LSquare => write!(f, "]"),
            Symbol::RSquare => write!(f, "["),
            Symbol::Number(str) => write!(f, "{str}"),
            Symbol::String(str) => write!(f, "{str}"),
        }
    }
}

#[derive(Debug)]
pub struct Token(pub Symbol, pub usize, pub usize);
impl From<String> for Symbol {
    fn from(value: String) -> Self {
        match &value[..] {
            "(" => Self::LParan,
            ")" => Self::RParan,
            "{" => Self::LBrack,
            "}" => Self::RBrack,
            "]" => Self::RSquare,
            "[" => Self::LSquare,
            x if x.starts_with('"') && x.ends_with('"') => Self::String(value),
            x if x.chars().all(|c| c.is_ascii_digit() || c == '.')
                || (x.starts_with('-')
                    && x[1..].chars().all(|c| c.is_ascii_digit() || c == '.')) =>
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

    let mut col = 1;
    let mut row = 1;

    for c in str.chars() {
        match c {
            // Comments
            '\n' if matches!(state, InComment) => {
                state = Normal;
                col = 1;
                row += 1;
            }
            _ if matches!(state, InComment) => {}
            '/' if !matches!(state, InComment | AlmostInComment) => {
                state = AlmostInComment;
                if !curr.is_empty() {
                    toks.push((col - curr.len(), row, curr));
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
                    toks.push((col - curr.len(), row, curr));
                }
                curr = String::new();
            }
            '\n' if matches!(state, InString) => {
                curr.push(c);
                col = 1;
                row += 1;
            }
            '\n' => {
                col = 1;
                row += 1;
            }

            // Strings
            '"' if matches!(state, Normal) => {
                state = InString;
                curr.push(c);
            }
            '"' if matches!(state, InString) => {
                state = Normal;
                curr.push(c);
                toks.push((col - curr.len(), row, curr));
                curr = String::new();
            }

            // Blocks
            '{' | '}' | '(' | ')' if matches!(state, Normal | AlmostInComment) => {
                state = Normal;
                if !curr.is_empty() {
                    toks.push(((col.max(curr.len())) - curr.len(), row, curr));
                }
                toks.push((col, row, c.to_string()));
                curr = String::new();
            }

            // Rest
            c => {
                curr.push(c);
            }
        }
        col += 1;
    }
    if !curr.is_empty() {
        toks.push((col - curr.len(), row, curr));
    }

    toks.into_iter()
        .map(|(col, row, s)| Token(s.into(), col, row))
        .collect()
}
