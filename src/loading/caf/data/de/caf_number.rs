
//-------------------------------------------------------------------------------------------------------------------

impl CafNumber {
    #[cold]
    pub(crate) fn unexpected(&self) -> Unexpected {
        if let Some(float) = self.number.as_f64() {
            Unexpected::Float(float)
        } else if let Some(uint) = self.number.as_u64() {
            Unexpected::Unsigned(uint)
        } else if let Some(int) = self.number.as_i64() {
            Unexpected::Signed(int)
        } else {
            unreachable!();
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
