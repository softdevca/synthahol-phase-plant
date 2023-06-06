pub trait HashTag {}

impl dyn HashTag {
    /// Convert the text to a hashtag with a leading hash mark if the text is not empty.
    /// Whitespace, control characters and extraneous hash marks are removed.
    pub fn from_lossy(text: &str) -> String {
        let cleaned: String = text
            .chars()
            .filter(|c| !(*c == '#' || c.is_whitespace() || c.is_control()))
            .collect();
        if cleaned.is_empty() {
            String::new()
        } else {
            format!("#{cleaned}")
        }
    }
}

pub trait TextOptionExt<T> {
    /// Trim the string inside the `Option`. Returns `Some` if there is text, otherwise `None`.
    fn trim(&self) -> Option<T>;

    fn empty_to_none(&self) -> Option<T>;
}

impl TextOptionExt<String> for Option<String> {
    fn trim(&self) -> Option<String> {
        self.as_ref().map(|s| s.trim().to_owned())
    }

    fn empty_to_none(&self) -> Option<String> {
        match self {
            Some(s) => {
                if s.is_empty() {
                    None
                } else {
                    Some(s.clone())
                }
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::TextOptionExt;
    use super::*;

    #[test]
    fn empty_to_none() {
        assert_eq!(None, None.empty_to_none());
        assert_eq!(
            Some("abc".to_owned()).empty_to_none(),
            Some("abc".to_owned())
        );
        assert_eq!(Some("".to_owned()).empty_to_none(), None);
    }

    #[test]
    fn hashtag_from_lossy() {
        assert_eq!(<dyn HashTag>::from_lossy(""), "");
        assert_eq!(<dyn HashTag>::from_lossy("#"), "");
        assert_eq!(<dyn HashTag>::from_lossy("###"), "");
        assert_eq!(<dyn HashTag>::from_lossy("tag"), "#tag");
        assert_eq!(<dyn HashTag>::from_lossy("#tag"), "#tag");
        assert_eq!(<dyn HashTag>::from_lossy(" # t a g  "), "#tag");
    }

    #[test]
    fn trim_string() {
        assert_eq!(None, None.trim());
        assert_eq!(Some("abc".to_owned()).trim(), Some("abc".to_owned()));
        assert_eq!(Some(" abc ".to_owned()).trim(), Some("abc".to_owned()));
        assert_eq!(Some("".to_owned()).trim(), Some("".to_owned()));
        assert_eq!(Some(" ".to_owned()).trim(), Some("".to_owned()));
        assert_eq!(Some(" \t ".to_owned()).trim(), Some("".to_owned()));
    }
}
