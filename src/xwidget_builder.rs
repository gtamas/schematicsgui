use relm4::gtk::glib::object::Cast;
use relm4::gtk::{Box, Orientation, Widget};

use crate::form_utils::FormUtils;
use crate::schema_parsing::{
    ChoiceEntry, ChoiceType, ColorEntry, ColorEntryType, DateEntry, DateEntryType, FsEntry,
    MenuEntry, MenuType, NumericEntry, NumericType, SchemaProp, TextEntry, XWidget, XWidgetType,
};

pub struct XWidgetBuilder {
    prop: SchemaProp,
    xwidget: XWidget,
    utils: FormUtils,
    field: String,
    cwd: Option<String>,
}

impl XWidgetBuilder {
    pub fn new(prop: &SchemaProp, field: String, cwd: Option<String>) -> Self {
        XWidgetBuilder {
            prop: prop.clone(),
            xwidget: prop.x_widget.clone().unwrap(),
            utils: FormUtils::new(),
            field,
            cwd,
        }
    }

    pub fn get_widget(&self) -> Widget {
        if let XWidgetType::Color(c) = &self.xwidget.options {
            if c.r#type == ColorEntryType::Button {
                return self.get_color_button(c.clone()).upcast();
            }
            self.get_color_input(&self.field, c.clone()).upcast()
        } else if let XWidgetType::Date(c) = &self.xwidget.options {
            if c.r#type == DateEntryType::Date {
                return self.get_date_input(c.clone()).upcast();
            } else if c.r#type == DateEntryType::Time {
                return self.get_time_input(c.clone()).upcast();
            }
            return self.get_date_time_input(c.clone()).upcast();
        } else if let XWidgetType::File(c) = &self.xwidget.options {
            return self.get_file_input(c.clone()).upcast();
        } else if let XWidgetType::Dir(c) = &self.xwidget.options {
            return self.get_dir_input(c.clone()).upcast();
        } else if let XWidgetType::Text(c) = &self.xwidget.options {
            if c.multiline {
                return self.get_textarea_input(c.clone()).upcast();
            }
            return self.get_text_input(c.clone()).upcast();
        } else if let XWidgetType::Numeric(c) = &self.xwidget.options {
            if c.r#type == NumericType::Input {
                return self.get_numeric_input(c.clone()).upcast();
            } else {
                return self.get_slider_input(c.clone()).upcast();
            }
        } else if let XWidgetType::Choice(c) = &self.xwidget.options {
            if c.r#type == ChoiceType::Switch {
                return self.get_switch_input(c.clone()).upcast();
            } else if c.r#type == ChoiceType::Toggle {
                return self.get_toggle_input(c.clone()).upcast();
            }
            return self.get_checkbox_input(c.clone()).upcast();
        } else if let XWidgetType::Menu(c) = &self.xwidget.options {
            if c.r#type == MenuType::Combobox {
                return self.get_combo(c.clone()).upcast();
            } else if c.r#type == MenuType::Radio {
                return self.get_radio_group(c.clone()).upcast();
            } else if c.r#type == MenuType::Toggle {
                return self.get_toggle_group(c.clone()).upcast();
            } else if c.r#type == MenuType::Multiselect {
                return self.get_multiselect(c.clone()).upcast();
            }
            return self.get_menu(c.clone()).upcast();
        } else {
            return Box::new(Orientation::Horizontal, 0).upcast();
        }
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

    fn get_dir_input(&self, mut options: FsEntry) -> Widget {
        options.is_dir = true;
        self.utils
            .file_input(&self.field, Some(options), None)
            .upcast()
    }

    fn get_slider_input(&self, options: NumericEntry) -> Widget {
        self.utils
            .slider(&self.field, Some(options), self.prop.default.clone())
            .upcast()
    }

    fn get_numeric_input(&self, options: NumericEntry) -> Widget {
        self.utils
            .numeric_input(&self.field, Some(options), self.prop.default.clone())
            .upcast()
    }

    fn get_checkbox_input(&self, options: ChoiceEntry) -> Widget {
        self.utils
            .checkbox_or_radio(&self.field, None, Some(options), self.prop.default.clone())
            .upcast()
    }

    fn get_switch_input(&self, options: ChoiceEntry) -> Widget {
        self.utils
            .switch(&self.field, Some(options), self.prop.default.clone())
            .upcast()
    }

    fn get_toggle_input(&self, options: ChoiceEntry) -> Widget {
        self.utils
            .toggle_button(&self.field, None, Some(options), self.prop.default.clone())
            .upcast()
    }

    fn get_date_input(&self, options: DateEntry) -> Widget {
        self.utils
            .date_input(&self.field, Some(options), self.prop.default.clone())
            .upcast()
    }

    fn get_time_input(&self, options: DateEntry) -> Widget {
        self.utils
            .time_input(&self.field, Some(options), self.prop.default.clone())
            .upcast()
    }

    fn get_date_time_input(&self, options: DateEntry) -> Widget {
        self.utils
            .date_time_input(&self.field, Some(options), self.prop.default.clone())
            .upcast()
    }

    fn get_color_button(&self, options: ColorEntry) -> Widget {
        self.utils
            .color_button(&self.field, Some(options), self.prop.default.clone())
            .upcast()
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

    fn get_combo(&self, options: MenuEntry) -> Widget {
        self.utils
            .combobox_text(
                &self.field,
                &self.get_items(),
                Some(options),
                self.prop.default.clone(),
            )
            .upcast()
    }

    fn get_radio_group(&self, options: MenuEntry) -> Widget {
        self.utils
            .radio_group(
                &self.field,
                &self.get_items(),
                Some(options),
                self.prop.default.clone(),
            )
            .upcast()
    }

    fn get_toggle_group(&self, options: MenuEntry) -> Widget {
        self.utils
            .toggle_group(
                &self.field,
                &self.get_items(),
                Some(options),
                self.prop.default.clone(),
            )
            .upcast()
    }

    fn get_text_input(&self, options: TextEntry) -> Widget {
        self.utils
            .text_input(&self.field, Some(options), self.prop.default.clone())
            .upcast()
    }

    fn get_textarea_input(&self, options: TextEntry) -> Widget {
        self.utils
            .textarea_input(&self.field, Some(options), self.prop.default.clone())
            .upcast()
    }

    fn get_color_input(&self, field: &str, options: ColorEntry) -> Widget {
        self.utils
            .color_input(field, Some(options), self.prop.default.clone())
            .upcast()
    }
}
