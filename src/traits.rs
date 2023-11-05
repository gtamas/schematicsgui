use relm4::gtk::glib::object::Object;
use relm4::gtk::prelude::{Cast, IsA};
use relm4::gtk::Widget;

pub trait Validator {
    fn print_error(&mut self, message: &str);
    fn print_success(&mut self, message: &str);
    fn clear_error(&mut self);
    fn clear_success(&mut self);
}

pub trait WidgetUtils {
    fn is_a<W: IsA<Object> + IsA<Widget> + Clone, T: IsA<Object> + IsA<Widget>>(
        &self,
        widget: &W,
    ) -> bool {
        widget.clone().upcast::<Widget>().downcast::<T>().is_ok()
    }
}

#[macro_export]
macro_rules! impl_validation {
    ($($t:ty),+ $(,)?) => ($(
        impl Validator for $t {
            fn print_error(&mut self, message: &str) {
                self.error = true;
                self.message =  format!("Error: {}",String::from(message));
            }

            fn print_success(&mut self, message: &str) {
                self.success = true;
                self.message = String::from(message);
            }

            fn clear_error(&mut self) {
                self.error = false;
            }

            fn clear_success(&mut self) {
                self.success = false;
            }
        }
    )+)
}
