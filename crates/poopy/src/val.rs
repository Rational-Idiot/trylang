use core::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Val {
    Number(i32),
    Unit,
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(num) => write!(f, "{}", num),
            Self::Unit => write!(f, "Unit"),
        }
    }
}
