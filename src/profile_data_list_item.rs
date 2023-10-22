use relm4::gtk::prelude::WidgetExt;
use relm4::{binding::StringBinding, gtk, typed_list_view::RelmListItem};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProfileDataMenuItem {
    pub file: String,
    pub label: String,
}

impl ProfileDataMenuItem {
    pub fn new(file: String, label: String) -> Self {
        ProfileDataMenuItem { file, label }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProfileDataListItem {
    pub value: ProfileDataMenuItem,
    binding: StringBinding,
}

impl ProfileDataListItem {
    pub fn new(value: ProfileDataMenuItem) -> Self {
        Self {
            value,
            binding: StringBinding::new(""),
        }
    }
}

pub struct Widgets {
    label: gtk::Label,
}

impl RelmListItem for ProfileDataListItem {
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

        label.set_label(&self.value.label);
        label.set_css_classes(&["selector_item"]);
    }
}
