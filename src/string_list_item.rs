use relm4::gtk::prelude::WidgetExt;
use relm4::{binding::StringBinding, gtk, typed_list_view::RelmListItem};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringListItem {
    pub value: String,
    binding: StringBinding,
}

impl StringListItem {
    pub fn new(value: String) -> Self {
        Self {
            value,
            binding: StringBinding::new(""),
        }
    }
}

impl From<StringListItem> for f32 {
    fn from(val: StringListItem) -> Self {
        val.value.parse().unwrap_or(0.0)
    }
}

pub struct Widgets {
    label: gtk::Label,
}

impl RelmListItem for StringListItem {
    type Root = gtk::Box;
    type Widgets = Widgets;

    fn setup(_item: &gtk::ListItem) -> (gtk::Box, Widgets) {
        relm4::view! {
            my_box = gtk::Box {
                #[name = "label"]
                gtk::Label,
            }
        }

        let widgets = Widgets { label };

        (my_box, widgets)
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        let Widgets { label } = widgets;

        label.set_label(&self.value);
        label.set_css_classes(&["selector_item"]);
    }
}
