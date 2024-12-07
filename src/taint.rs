#[derive(Debug, PartialEq)]
pub enum OpSuccessColor {
    OpSuccessNotInTheEnd,
    OpSuccessOnlyAtTheEnd,
    NoOpSuccess,
    Undetermined,
}

impl Default for OpSuccessColor {
    fn default() -> Self {
        OpSuccessColor::Undetermined
    }
}
