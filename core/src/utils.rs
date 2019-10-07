pub trait FindKind<T, D> {
    fn find_kind(&self, discrimenant: D) -> Option<&T>;
}

impl<T, D: PartialEq + for<'a> From<&'a T>> FindKind<T, D> for Vec<T> {
    fn find_kind(&self, discrimenant: D) -> Option<&T> {
        self.iter().find(|value| {
            let kind: D = (*value).into();
            kind == discrimenant
        })
    }
}
