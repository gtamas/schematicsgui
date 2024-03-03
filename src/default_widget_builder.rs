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
    cwd: Option<String>,
}

impl DefaultWidgetBuilder {
    pub fn new(prop: &SchemaProp, field: String, cwd: Option<String>) -> Self {
        DefaultWidgetBuilder {
            prop: prop.clone(),
            utils: FormUtils::new(),
            cwd,
            field,
        }
    }

    pub fn get_widget(&self) -> Widget {
        let prompt = self.prop.x_prompt.as_ref();

        if self.prop.r#type == "string" || self.prop.r#type == "array" {
            if self.prop.r#enum.is_some()
                || (self.prop.x_prompt.is_some()
                    && self.prop.x_prompt.as_ref().unwrap().has_items())
            {
                if prompt.is_some() && prompt.unwrap().has_multiselect() {
                    return self.get_multiselect(MenuEntry::default()).upcast();
                }
                return self.get_menu(MenuEntry::default()).upcast();
            } else if self.prop.format.is_some() {
                let format = self.prop.format.as_deref().unwrap();
                if format == "path" {
                    return self
                        .get_file_input(FsEntry {
                            is_dir: true,
                            ..Default::default()
                        })
                        .upcast();
                } else if format == "date" {
                    return self.get_date_input(DateEntry::default()).upcast();
                }
            }
            self.get_text_input(TextEntry::default()).upcast()
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

    fn get_multiselect(&self, options: MenuEntry) -> Widget {
        self.utils
            .multiselect_input(
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
        }
        if (self.prop.r#type == "array" || self.prop.r#type == "string")
            && self.prop.x_prompt.is_some()
            && self.prop.x_prompt.as_ref().unwrap().has_items()
        {
            let prompt = self.prop.x_prompt.as_ref().unwrap();
            if self.cwd.is_some() {
                if prompt.has_modules() {
                    return prompt.get_modules(self.cwd.as_ref().unwrap());
                } else if prompt.has_models() {
                    return prompt.get_models(self.cwd.as_ref().unwrap());
                } else if prompt.has_dirs() {
                    return prompt.get_dirs_or_files(true, self.cwd.as_ref().unwrap());
                } else if prompt.has_files() {
                    return prompt.get_dirs_or_files(false, self.cwd.as_ref().unwrap());
                }
            }
            if prompt.has_modules() || prompt.has_models() || prompt.has_dirs() {
                return prompt.get_items_placeholder();
            }
            return prompt.get_items();
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
            .switch(&self.field, Some(options), self.prop.default.clone())
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
