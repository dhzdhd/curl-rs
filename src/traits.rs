pub trait Tab
where
    Self: Clone + Copy + PartialEq + Eq + Sized,
{
    fn as_int(&self) -> u8;
    fn to_enum(&self, num: u8) -> Self;
    fn next(&self) -> Self;
    fn previous(&self) -> Self;
}
