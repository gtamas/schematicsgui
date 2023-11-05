use relm4::gtk::glib::GString;
use relm4::gtk::prelude::{
    ButtonExt, Cast, CheckButtonExt, ColorChooserExt, EntryBufferExtManual, EntryExt, RangeExt,
    TextBufferExt, TextViewExt, ToggleButtonExt, WidgetExt,
};
use relm4::gtk::{
    Box, CheckButton, ColorButton, ComboBoxText, DropDown, Entry, EntryBuffer, Range, SpinButton,
    StringObject, Switch, TextView, ToggleButton, Widget,
};

use crate::command_builder::{InputType, Param};
use crate::form_utils::FormUtils;
use crate::schema_parsing::ColorEntryFormat;
use crate::traits::WidgetUtils;

pub struct ValueExtractor<'l> {
    widget: &'l Widget,
}

impl<'l> WidgetUtils for ValueExtractor<'l> {}

impl<'l> ValueExtractor<'l> {
    fn get_optional_param_value(&self, value: String, result: Param) -> Option<Param> {
        match value.is_empty() {
            true => None,
            false => Some(result),
        }
    }

    pub fn new(widget: &'l Widget) -> Self {
        ValueExtractor { widget }
    }

    pub fn set_widget(&mut self, widget: &'l Widget) {
        self.widget = widget
    }

    pub fn get_name_value(&self) -> Option<Param> {
        let name = self.widget.widget_name().to_string();
        if self.is_a::<_, Entry>(self.widget) {
            let value = self.get_entry_value(None).to_owned();
            self.get_optional_param_value(value.clone(), Param::new(name, value, InputType::Text))
        } else if self.is_a::<_, TextView>(self.widget) {
            let value = self.get_text_view_value();
            self.get_optional_param_value(
                value.clone(),
                Param::new(name, value, InputType::TextArea),
            )
        } else if self.is_a::<_, Switch>(self.widget) {
            Some(Param {
                name,
                value: self.get_switch_value(),
                kind: InputType::Switch,
            })
        } else if self.is_a::<_, ColorButton>(self.widget) {
            let value = self.get_color_button_value();
            self.get_optional_param_value(
                value.clone(),
                Param::new(name, value, InputType::ColorButton),
            )
        } else if self.is_a::<_, Box>(self.widget) {
            let container = self.widget.clone().downcast::<Box>().unwrap();
            let kind = container.css_classes();

            if kind.contains(&GString::from("slider_input_container")) {
                Some(Param {
                    name: container
                        .first_child()
                        .unwrap()
                        .next_sibling()
                        .unwrap()
                        .widget_name()
                        .to_string(),
                    value: self.get_slider_value(&container),
                    kind: InputType::Slider,
                })
            } else if kind.contains(&GString::from("radio_group_container"))
                || kind.contains(&GString::from("toggle_group_container"))
            {
                Some(Param {
                    name: container.first_child().unwrap().widget_name().to_string(),
                    value: self.get_group_value(&container),
                    kind: match kind.contains(&GString::from("radio_group_container")) {
                        true => InputType::RadioGroup,
                        false => InputType::ToggleGroup,
                    },
                })
            } else if kind.contains(&GString::from("date_input_container"))
                || kind.contains(&GString::from("file_input_container"))
                || kind.contains(&GString::from("dir_input_container"))
                || kind.contains(&GString::from("color_input_container"))
            {
                let k: InputType;
                if kind.contains(&GString::from("date_input_container")) {
                    k = InputType::Date;
                } else if kind.contains(&GString::from("color_input_container")) {
                    k = InputType::ColorInput;
                } else if kind.contains(&GString::from("file_input_container")) {
                    k = InputType::File;
                } else {
                    k = InputType::Dir;
                }
                let value = self.get_entry_value(container.first_child());
                let name = container.first_child().unwrap().widget_name().to_string();
                self.get_optional_param_value(value.clone(), Param::new(name, value, k))
            } else {
                Some(Param::default())
            }
        } else if self.is_a::<_, CheckButton>(self.widget) {
            Some(Param {
                name,
                value: self.get_check_button_value(),
                kind: InputType::Checkbox,
            })
        } else if self.is_a::<_, ToggleButton>(self.widget) {
            Some(Param {
                name,
                value: self.get_toggle_button_value(),
                kind: InputType::Toggle,
            })
        } else if self.is_a::<_, SpinButton>(self.widget) {
            let value = self.get_numeric_input();
            self.get_optional_param_value(
                value.clone(),
                Param::new(name, value, InputType::Numeric),
            )
        } else if self.is_a::<_, DropDown>(self.widget) {
            Some(Param {
                name,
                value: self.get_dropdown_value(),
                kind: InputType::DropDown,
            })
        } else if self.is_a::<_, ComboBoxText>(self.widget) {
            Some(Param {
                name,
                value: self.get_combo_box_value(),
                kind: InputType::Combobox,
            })
        } else {
            Some(Param::default())
        }
    }

    fn get_entry_value(&self, entry: Option<Widget>) -> String {
        let bf: EntryBuffer;

        if entry.is_some() {
            bf = entry.unwrap().clone().downcast::<Entry>().unwrap().buffer();
        } else {
            bf = self.widget.clone().downcast::<Entry>().unwrap().buffer();
        }

        bf.text().to_string()
    }

    fn get_text_view_value(&self) -> String {
        let bf = self.widget.clone().downcast::<TextView>().unwrap().buffer();
        bf.text(&bf.start_iter(), &bf.end_iter(), false).to_string()
    }

    fn get_slider_value(&self, container: &Box) -> String {
        let scale = container
            .first_child()
            .unwrap()
            .next_sibling()
            .unwrap()
            .downcast::<Range>()
            .unwrap();
        scale.value().to_string()
    }

    fn get_toggle_button_value(&self) -> String {
        let toggle = self.widget.clone().downcast::<ToggleButton>().unwrap();
        toggle.is_active().to_string()
    }

    fn get_check_button_value(&self) -> String {
        let checkbox = self.widget.clone().downcast::<CheckButton>().unwrap();
        checkbox.is_active().to_string()
    }

    fn get_numeric_input(&self) -> String {
        let entry = self.widget.clone().downcast::<SpinButton>().unwrap();
        // TODO format according to options
        entry.value().to_string()
    }

    fn get_switch_value(&self) -> String {
        let switch = self.widget.clone().downcast::<Switch>().unwrap();
        switch.is_active().to_string()
    }

    fn get_combo_box_value(&self) -> String {
        let combo: ComboBoxText = self.widget.clone().downcast::<ComboBoxText>().unwrap();
        combo.active_text().unwrap_or(GString::from("")).into()
    }

    fn get_color_button_value(&self) -> String {
        let button = self.widget.clone().downcast::<ColorButton>().unwrap();
        // TODO format according to options
        FormUtils::format_color_str(ColorEntryFormat::RGB, &button.rgba())
    }

    fn get_group_value(&self, container: &Box) -> String {
        let mut w = container.first_child();

        loop {
            let widget = w.as_ref().unwrap();
            if self.is_a::<_, CheckButton>(widget) {
                let button = widget.clone().downcast::<CheckButton>().unwrap();
                if button.is_active() {
                    return button.label().unwrap().to_string();
                }
            } else if self.is_a::<_, ToggleButton>(widget) {
                let button = widget.clone().downcast::<ToggleButton>().unwrap();
                if button.is_active() {
                    return button.label().unwrap().to_string();
                }
            }
            w = w.as_ref().unwrap().next_sibling();

            if (w.is_none()) {
                break String::from("");
            }
        }
    }

    fn get_dropdown_value(&self) -> String {
        let dropdown = self.widget.clone().downcast::<DropDown>().unwrap();
        let selected = dropdown.selected_item();
        selected
            .unwrap()
            .downcast::<StringObject>()
            .unwrap()
            .string()
            .to_string()
    }
}
