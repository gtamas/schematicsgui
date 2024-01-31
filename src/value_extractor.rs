use crate::command_builder::{InputType, Param};
use crate::form_utils::FormUtils;
use crate::schema_parsing::ColorEntryFormat;
use crate::string_list_item::StringListItem;
use crate::traits::WidgetUtils;
use relm4::gtk::gio::ListModel;
use relm4::gtk::glib::{BoxedAnyObject, GString};
use relm4::gtk::prelude::{
    ButtonExt, Cast, CheckButtonExt, ColorChooserExt, EntryBufferExtManual, EntryExt, ListModelExt,
    RangeExt, SelectionModelExt, TextBufferExt, TextViewExt, ToggleButtonExt, WidgetExt,
};
use relm4::gtk::{
    Box, Calendar, CheckButton, ColorButton, ComboBoxText, DropDown, Entry, EntryBuffer, ListView,
    MultiSelection, Range, SpinButton, StringObject, Switch, TextView, ToggleButton, Widget,
};
use std::cell::Ref;

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
            } else if kind.contains(&GString::from("time_input_container")) {
                Some(Param {
                    name: container.widget_name().to_string(),
                    value: self.get_time_input_value(&container),
                    kind: InputType::Time,
                })
            } else if kind.contains(&GString::from("date_time_input_container")) {
                Some(Param {
                    name: container.widget_name().to_string(),
                    value: self.get_date_time_input_value(&container),
                    kind: InputType::DateTime,
                })
            } else if kind.contains(&GString::from("date_input_container")) {
                Some(Param {
                    name: container.widget_name().to_string(),
                    value: self.get_date_input_value(&container),
                    kind: InputType::Date,
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
            } else if kind.contains(&GString::from("file_input_container"))
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
        } else if self.is_a::<_, ListView>(self.widget) {
            Some(Param {
                name,
                value: self.get_multiselect_value(),
                kind: InputType::Multiselect,
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
        let bf: EntryBuffer = if let Some(entry_value) = entry {
            entry_value.clone().downcast::<Entry>().unwrap().buffer()
        } else {
            self.widget.clone().downcast::<Entry>().unwrap().buffer()
        };

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

    fn get_calendar(&self, container: &Box) -> Calendar {
        container
            .first_child()
            .unwrap()
            .downcast::<Calendar>()
            .unwrap()
    }

    fn get_date_input_value(&self, container: &Box) -> String {
        let calendar = self.get_calendar(container);

        let date = FormUtils::format_date(String::from("%Y-%m-%d"), &calendar.date());
        format!("{}", date)
    }

    fn get_date_time_input_value(&self, container: &Box) -> String {
        let calendar = self.get_calendar(container);
        let date = self.get_date_input_value(container);

        let time_widget = calendar.next_sibling().unwrap().downcast::<Box>().unwrap();
        let time = self.get_time_input_value(&time_widget);
        format!("{} {}", date, time)
    }

    fn get_time_input_value(&self, container: &Box) -> String {
        let hour = container
            .first_child()
            .unwrap()
            .downcast::<SpinButton>()
            .unwrap();
        let minute = hour
            .next_sibling()
            .unwrap()
            .downcast::<SpinButton>()
            .unwrap();
        let second = minute
            .next_sibling()
            .unwrap()
            .downcast::<SpinButton>()
            .unwrap();
        format!(
            "{:02}:{:02}:{:02}",
            hour.value(),
            minute.value(),
            second.value()
        )
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

            if w.is_none() {
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

    fn get_multiselect_value(&self) -> String {
        let mut result: Vec<String> = vec![];
        let list_view = self.widget.clone().downcast::<ListView>().unwrap();
        let selection = list_view
            .model()
            .unwrap()
            .downcast::<MultiSelection>()
            .unwrap();
        let list_model = selection.model().unwrap().downcast::<ListModel>().unwrap();
        let items_no = list_model.n_items();

        for i in 0..items_no {
            if selection.is_selected(i) {
                let item = list_model.item(i).unwrap();
                let wrapper = item.downcast::<BoxedAnyObject>().unwrap();
                let value: Ref<StringListItem> = wrapper.borrow();
                result.push(value.value.to_string());
            }
        }

        result.join(",")
    }
}
