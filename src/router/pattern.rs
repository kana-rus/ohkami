#[derive(PartialEq, Debug)]
pub(super) enum Pattern<'p> {
    Any,
    Str(&'p str),
    Param(&'p str /* param name */),
} impl<'p> Pattern<'p> {
    pub fn from(section: &'p str) -> Self {
        match section {
            "*" => Self::Any,
            p if p.starts_with(':') => Self::Param(&p[1..]),
            p => Self::Str(p),
        }
    }
    pub fn matches(&self, section: &str) -> (bool, Option<(&'p str, &'p str)>) {
        match self {
            Pattern::Any => (true, None),
            Pattern::Str(p) => (p == &section, None),
            Pattern::Param(name) => (true, Some((name, section))),
        }
    }
    fn is_param(&self) -> bool {
        match self {
            Pattern::Param(_) => true,
            _ => false,
        }
    }
    fn is_str(&self) -> bool {
        match self {
            Pattern::Str(_) => true,
            _ => false,
        }
    }
}