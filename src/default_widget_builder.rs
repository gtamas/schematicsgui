use relm4::gtk::glib::object::Cast;
use relm4::gtk::{Box, Orientation, Widget};

use crate::form_utils::FormUtils;
use crate::schema_parsing::{
    ChoiceEntry, DateEntry, FsEntry, MenuEntry, NumericEntry, SchemaProp, TextEntry,
};

pub struct DefaultWidgetBuilder {
    prop: SchemaProp,
    utils: FormUtils,
    field: String,
}

impl DefaultWidgetBuilder {
    pub fn new(prop: &SchemaProp, field: String) -> Self {
        DefaultWidgetBuilder {
            prop: prop.clone(),
            utils: FormUtils::new(),
            field,
        }
    }

    pub fn get_widget(&self) -> Widget {
        if self.prop.r#type == "string" {
            if self.prop.r#enum.is_some() {
                return self.get_menu(MenuEntry::default()).upcast();
            } else if self.prop.format.is_some() {
                let format = self.prop.format.as_deref().unwrap();
                if format == "path" {
                    return self.get_file_input(FsEntry::default()).upcast();
                } else if format == "date" {
                    return self.get_date_input(DateEntry::default()).upcast();
                }
            }
            return self.get_text_input(TextEntry::default()).upcast();
        } else if self.prop.r#type == "boolean" {
            return self.get_switch_input(ChoiceEntry::default()).upcast();
        } else if self.prop.r#type == "number" {
            return self.get_numeric_input(NumericEntry::default()).upcast();
        } else {
            return Box::new(Orientation::Horizontal, 0).upcast();
        }
    }

    fn get_menu(&self, options: MenuEntry) -> Widget {
        self.utils
            .dropdown(
                &self.field,
                &self.get_items(),
                Some(options),
                self.prop.default.clone(),
            )
            .upcast()
    }

    fn get_items(&self) -> Vec<String> {
        let empty: Vec<String> = vec![];
        if self.prop.r#type == "string" && self.prop.r#enum.is_some() {
            return self.prop.r#enum.as_ref().unwrap().clone();
        } else if self.prop.r#type == "array" && self.prop.items.is_some() {
            return self.prop.items.as_ref().unwrap().r#enum.clone();
        }

        empty
    }

    fn get_file_input(&self, options: FsEntry) -> Widget {
        self.utils
            .file_input(&self.field, Some(options), None)
            .upcast()
    }

    fn get_numeric_input(&self, options: NumericEntry) -> Widget {
        self.utils
            .numeric_input(&self.field, Some(options), self.prop.default.clone())
            .upcast()
    }

    fn get_switch_input(&self, options: ChoiceEntry) -> Widget {
        self.utils
            .switch(&self.field, self.prop.default.clone())
            .upcast()
    }

    fn get_date_input(&self, options: DateEntry) -> Widget {
        self.utils
            .date_input(&self.field, Some(options), self.prop.default.clone())
            .upcast()
    }

    fn get_text_input(&self, options: TextEntry) -> Widget {
        self.utils
            .text_input(&self.field, Some(options), self.prop.default.clone())
            .upcast()
    }
}
