pub trait Validator {
    fn set_error(&mut self, message: &str);
    fn clear_error(&mut self);
}

#[macro_export]
macro_rules! impl_validation {
    ($($t:ty),+ $(,)?) => ($(
        impl Validator for $t {
            fn set_error(&mut self, message: &str) {
                self.error = true;
                self.error_message = String::from(message);
            }

            fn clear_error(&mut self) {
                self.error = false;
            }
        }
    )+)
}
