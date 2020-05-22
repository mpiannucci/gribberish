
pub struct LocalUseSection<'a> {
    data: &'a[u8],
}

impl<'a> LocalUseSection<'a> {
    pub fn from_data(data: &[u8]) -> LocalUseSection {
        LocalUseSection {
            data: &data,
        }
    }
}