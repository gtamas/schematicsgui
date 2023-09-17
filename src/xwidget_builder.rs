use std::marker::PhantomData;

use relm4::gtk::glib::object::Cast;
use relm4::gtk::{ApplicationWindow, Box, CheckButton, EntryBuffer, TextBuffer, Window};
use relm4::gtk::{Label, Widget};

use crate::form_utils::FormUtils;
use crate::schematic_ui::{
    ColorEntry, ColorEntryType, DateEntry, DirEntry, FileEntry, MenuEntry, MenuType, NumericEntry,
    Primitive, SchemaProp, TextEntry, XWidget, XWidgetType,
};

pub struct XWidgetBuilder<'r> {
    prop: SchemaProp,
    xwidget: XWidget,
    utils: FormUtils,
    field: String,
    phantom: PhantomData<&'r Widget>,
}

impl<'r> XWidgetBuilder<'r> {
    pub fn new(prop: &SchemaProp, field: String) -> Self {
        println!("{:?}", prop);
        XWidgetBuilder {
            prop: prop.clone(),
            xwidget: prop.x_widget.clone().unwrap(),
            utils: FormUtils::new(),
            phantom: PhantomData,
            field,
        }
    }

    pub fn get_widget(&self) -> Widget {
        if let XWidgetType::Color(c) = &self.xwidget.options {
            if c.r#type == ColorEntryType::Button {
                return self.get_color_button(c.clone()).upcast();
            }
            return self.get_color_input(&self.field, c.clone()).upcast();
        } else if let XWidgetType::Date(c) = &self.xwidget.options {
            return self.get_date_input(c.clone()).upcast();
        } else if let XWidgetType::File(c) = &self.xwidget.options {
            return self.get_file_input(c.clone()).upcast();
        } else if let XWidgetType::Dir(c) = &self.xwidget.options {
            return self.get_dir_input(c.clone()).upcast();
        } else if let XWidgetType::Slider(c) = &self.xwidget.options {
            return self.get_slider_input(c.clone()).upcast();
        } else if let XWidgetType::Text(c) = &self.xwidget.options {
            return self.get_text_input(c.clone()).upcast();
        } else if let XWidgetType::Menu(c) = &self.xwidget.options {
            if c.r#type == MenuType::DropDown {
                return self.get_menu(c.clone()).upcast();
            } else if c.r#type == MenuType::Radio {
                return self.get_radio_group(c.clone()).upcast();
            } else if c.r#type == MenuType::Toggle {
                return self.get_toggle_group(c.clone()).upcast();
            }
        }

        relm4::gtk::ColorButton::new().upcast()
    }

    fn get_file_input(&self, options: FileEntry) -> Widget {
        self.utils
            .file_input(&self.field, Some(options), None)
            .upcast()
    }

    fn get_dir_input(&self, options: DirEntry) -> Widget {
        self.utils
            .file_input(&self.field, None, Some(options))
            .upcast()
    }

    fn get_slider_input(&self, options: NumericEntry) -> Widget {
        self.utils
            .slider(&self.field, Some(options), self.prop.default.clone())
            .upcast()
    }

    fn get_date_input(&self, options: DateEntry) -> Widget {
        self.utils
            .date_input(&self.field, Some(options), self.prop.default.clone())
            .upcast()
    }

    fn get_color_button(&self, options: ColorEntry) -> Widget {
        self.utils
            .color_button(&self.field, Some(options), self.prop.default.clone())
            .upcast()
    }

    fn get_menu(&self, options: MenuEntry) -> Widget {
        self.utils
            .dropdown("dsd", &self.field, self.prop.r#enum.as_ref().unwrap(), None)
            .upcast()
    }

    fn get_radio_group(&self, options: MenuEntry) -> Widget {
        self.utils
            .radio_group(&self.field, &self.prop.r#enum.clone().unwrap())
            .upcast()
    }

    fn get_toggle_group(&self, options: MenuEntry) -> Widget {
        self.utils
            .toggle_group(&self.field, &self.prop.r#enum.clone().unwrap())
            .upcast()
    }

    fn get_text_input(&self, options: TextEntry) -> Widget {
        if options.multiline {
            self.utils
                .textarea_input(&self.field, Some(options), None)
                .upcast()
        } else {
            self.utils
                .text_input(&self.field, Some(options), None)
                .upcast()
        }
    }

    fn get_color_input(&self, field: &str, options: ColorEntry) -> Widget {
        self.utils
            .color_input(field, Some(options), self.prop.default.clone())
            .upcast()
    }
}
