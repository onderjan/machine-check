use proc_macro2::Span;
use std::hash::Hash;

#[derive(Clone, Debug)]
pub struct IPath {
    pub leading_colon: Option<Span>,
    pub segments: Vec<IPathSegment>,
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct IPathSegment {
    pub ident: IIdent,
}

impl IPath {
    /// Returns true if the path is absolute and the segment idents start with the given strings.
    ///
    /// Does not take generics into account.
    pub fn starts_with_absolute(&self, segments: &[&str]) -> bool {
        if self.leading_colon.is_none() {
            return false;
        }
        if self.segments.len() < segments.len() {
            return false;
        }
        for (self_segment, other_segment) in self.segments.iter().zip(segments.iter()) {
            if self_segment.ident.name != *other_segment {
                return false;
            }
        }
        true
    }

    /// Returns true if the path is relative and the segment idents match the given strings.
    ///
    /// Does not take generics into account.
    pub fn matches_relative(&self, segments: &[&str]) -> bool {
        if self.leading_colon.is_some() {
            return false;
        }
        if self.segments.len() != segments.len() {
            return false;
        }
        for (self_segment, other_segment) in self.segments.iter().zip(segments.iter()) {
            if self_segment.ident.name != *other_segment {
                return false;
            }
        }
        true
    }

    pub fn from_ident(ident: IIdent) -> Self {
        IPath {
            leading_colon: None,
            segments: vec![IPathSegment { ident }],
        }
    }

    pub fn span(&self) -> Span {
        // TODO: correct span
        if let Some(last_segment) = self.segments.last() {
            last_segment.ident.span
        } else {
            Span::call_site()
        }
    }

    pub fn segments_strs(&self) -> impl Iterator<Item = &str> {
        self.segments
            .iter()
            .map(|segment| segment.ident.name.as_str())
    }

    pub fn get_ident(&self) -> Option<&IIdent> {
        if self.leading_colon.is_none() && self.segments.len() == 1 {
            Some(&self.segments[0].ident)
        } else {
            None
        }
    }
}

impl PartialEq for IPath {
    fn eq(&self, other: &Self) -> bool {
        self.leading_colon.is_some() == other.leading_colon.is_some()
            && self.segments == other.segments
    }
}

impl Eq for IPath {}

impl Hash for IPath {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let has_leading_colon = self.leading_colon.is_some();
        has_leading_colon.hash(state);
        self.segments.hash(state);
    }
}

#[derive(Clone, Debug)]
pub struct IIdent {
    name: String,
    span: Span,
}

impl IIdent {
    pub fn new(name: String, span: Span) -> Self {
        Self { name, span }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_span(&mut self, span: Span) {
        self.span = span;
    }

    pub fn into_path(self) -> IPath {
        IPath::from_ident(self)
    }

    pub fn mck_prefixed(&self, prefix: &str) -> IIdent {
        let orig_ident_str = self.name();
        // make sure everything is prefixed by __mck_ only once at the start
        let stripped_ident_str = orig_ident_str
            .strip_prefix("__mck_")
            .unwrap_or(orig_ident_str);

        IIdent::new(
            format!("__mck_{}_{}", prefix, stripped_ident_str),
            self.span(),
        )
    }
}

impl PartialEq for IIdent {
    fn eq(&self, other: &Self) -> bool {
        // do not consider span for equality
        self.name == other.name
    }
}

impl Eq for IIdent {}

impl Hash for IIdent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // do not consider span for the hash
        // this is fine as it just means two idents
        // with different spans will hash to the same value
        self.name.hash(state);
    }
}

impl PartialOrd for IIdent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IIdent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // do not consider span for comparison
        self.name.cmp(&other.name)
    }
}
