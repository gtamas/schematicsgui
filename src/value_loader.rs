use relm4::gtk::glib::object::Object;
use relm4::gtk::glib::{DateTime, GString, TimeZone};
use relm4::gtk::prelude::{
    ButtonExt, Cast, CheckButtonExt, ColorChooserExt, ComboBoxExt, EntryBufferExtManual, EntryExt,
    IsA, ListModelExtManual, RangeExt, TextBufferExt, TextViewExt, ToggleButtonExt, WidgetExt,
};
use relm4::gtk::{
    Box, Button, Calendar, CheckButton, ColorButton, ComboBoxText, DropDown, Entry, EntryBuffer,
    Range, SpinButton, StringList, StringObject, Switch, TextView, ToggleButton, Widget,
};

use crate::form_utils::FormUtils;
use crate::traits::WidgetUtils;
use toml::Value;

pub struct ValueLoader<'l> {
    widget: &'l Widget,
}

impl<'l> WidgetUtils for ValueLoader<'l> {}

impl<'l> ValueLoader<'l> {
    pub fn new(widget: &'l Widget) -> Self {
        ValueLoader { widget }
    }

    pub fn set_widget(&mut self, widget: &'l Widget) {
        self.widget = widget
    }

    pub fn set_value(&self, value: &Value, widget_name: &str) -> () {
        if self.is_a::<_, Entry>(self.widget) {
            self.set_entry_value(value, None);
        } else if self.is_a::<_, TextView>(self.widget) {
            self.set_text_view_value(value);
        } else if self.is_a::<_, Switch>(self.widget) {
            self.set_switch_value(value);
        } else if self.is_a::<_, ColorButton>(self.widget) {
            self.set_color_button_value(value);
        } else if self.is_a::<_, Box>(self.widget) {
            let container = self.widget.clone().downcast::<Box>().unwrap();
            let kind = container.css_classes();

            if kind.contains(&GString::from("slider_input_container")) {
                self.set_slider_value(value, &container);
            } else if kind.contains(&GString::from("time_input_container")) {
                self.set_time_value(value, &container);
            } else if kind.contains(&GString::from("date_time_input_container")) {
                self.set_date_time_value(value, &container);
            } else if kind.contains(&GString::from("date_input_container")) {
                self.set_date_value(value, &container);
            } else if kind.contains(&GString::from("radio_group_container"))
                || kind.contains(&GString::from("toggle_group_container"))
            {
                self.set_group_value(value, &container);
            } else if kind.contains(&GString::from("file_input_container"))
                || kind.contains(&GString::from("dir_input_container"))
                || kind.contains(&GString::from("color_input_container"))
            {
                self.set_entry_value(value, container.first_child());
            }
        } else if self.is_a::<_, CheckButton>(self.widget) {
            self.set_check_button_value(value);
        } else if self.is_a::<_, ToggleButton>(self.widget) {
            self.set_toggle_button_value(value);
        } else if self.is_a::<_, SpinButton>(self.widget) {
            self.set_numeric_input(value);
        } else if self.is_a::<_, DropDown>(self.widget) {
            self.set_dropdown_value(value);
        } else if self.is_a::<_, ComboBoxText>(self.widget) {
            self.set_combo_box_value(value);
        }
    }

    fn set_entry_value(&self, value: &Value, entry: Option<Widget>) -> () {
        let bf: EntryBuffer;

        if entry.is_some() {
            bf = entry.unwrap().clone().downcast::<Entry>().unwrap().buffer();
        } else {
            bf = self.widget.clone().downcast::<Entry>().unwrap().buffer();
        }

        bf.set_text(value.as_str().unwrap_or_default());
    }

    fn set_text_view_value(&self, value: &Value) -> () {
        let bf = self.widget.clone().downcast::<TextView>().unwrap().buffer();
        bf.set_text(value.as_str().unwrap_or_default());
    }

    fn set_slider_value(&self, value: &Value, container: &Box) -> () {
        let scale = container
            .first_child()
            .unwrap()
            .next_sibling()
            .unwrap()
            .downcast::<Range>()
            .unwrap();
        scale.set_value(
            value
                .as_str()
                .unwrap_or("0.0")
                .parse::<f64>()
                .unwrap_or_default(),
        );
    }

    fn set_date_time_value(&self, value: &Value, container: &Box) -> () {
        let default_date = FormUtils::format_date(String::from(""), &DateTime::now_utc().unwrap());
        let vale_str = value.as_str().unwrap_or(default_date.as_str());
        let v = vale_str.split(" ").collect::<Vec<&str>>();

        self.set_date_value(value, container);

        self.set_time_value(
            &Value::String(String::from(v[1])),
            &container
                .first_child()
                .unwrap()
                .next_sibling()
                .unwrap()
                .downcast::<Box>()
                .unwrap(),
        );
    }

    fn set_date_value(&self, value: &Value, container: &Box) -> () {
        let default_date =
            FormUtils::format_date(String::from("%Y-%m-%d"), &DateTime::now_utc().unwrap());
        let vale_str = format!(
            "{} 00:00:00",
            value.as_str().unwrap_or(default_date.as_str())
        );

        let calendar = container
            .first_child()
            .unwrap()
            .downcast::<Calendar>()
            .unwrap();

        let d = DateTime::from_iso8601(&vale_str, Some(&TimeZone::utc()));
        calendar.select_day(&d.unwrap());
    }

    fn set_time_value(&self, value: &Value, container: &Box) -> () {
        let mut v = value
            .as_str()
            .unwrap_or("0:0:0")
            .split(":")
            .collect::<Vec<&str>>();

        if v.len() != 3 {
            v = vec!["0", "0", "0"];
        }

        let buttons = [
            container
                .first_child()
                .unwrap()
                .downcast::<SpinButton>()
                .unwrap(),
            container
                .first_child()
                .unwrap()
                .next_sibling()
                .unwrap()
                .downcast::<SpinButton>()
                .unwrap(),
            container
                .first_child()
                .unwrap()
                .next_sibling()
                .unwrap()
                .next_sibling()
                .unwrap()
                .downcast::<SpinButton>()
                .unwrap(),
        ];

        for (index, button) in buttons.iter().enumerate() {
            button.set_value(v[index].parse::<f64>().unwrap_or_default());
        }
    }

    fn set_toggle_button_value(&self, value: &Value) -> () {
        let toggle = self.widget.clone().downcast::<ToggleButton>().unwrap();
        toggle.set_active(
            value
                .as_str()
                .unwrap_or("false")
                .parse::<bool>()
                .unwrap_or_default(),
        );
    }

    fn set_check_button_value(&self, value: &Value) -> () {
        let checkbox = self.widget.clone().downcast::<CheckButton>().unwrap();
        checkbox.set_active(
            value
                .as_str()
                .unwrap_or("false")
                .parse::<bool>()
                .unwrap_or_default(),
        );
    }

    fn set_numeric_input(&self, value: &Value) -> () {
        let entry = self.widget.clone().downcast::<SpinButton>().unwrap();
        entry.set_value(
            value
                .as_str()
                .unwrap_or("0.0")
                .parse::<f64>()
                .unwrap_or_default(),
        );
    }

    fn set_switch_value(&self, value: &Value) -> () {
        let switch = self.widget.clone().downcast::<Switch>().unwrap();
        switch.set_active(
            value
                .as_str()
                .unwrap_or("false")
                .parse::<bool>()
                .unwrap_or_default(),
        );
    }

    fn set_combo_box_value(&self, value: &Value) -> () {
        let combo: ComboBoxText = self.widget.clone().downcast::<ComboBoxText>().unwrap();
        combo.set_active_id(Some("1"));
        // combo.set_active(Some(0));
    }

    fn set_color_button_value(&self, value: &Value) -> () {
        let button = self.widget.clone().downcast::<ColorButton>().unwrap();
        let rgb = FormUtils::color_str_to_rgba(value.as_str().unwrap_or("rgb(0,0,0)"));
        button.set_rgba(&rgb);
    }

    fn set_group_value(&self, value: &Value, container: &Box) -> () {
        let mut w = container.first_child();

        loop {
            let widget = w.as_ref().unwrap();
            if self.is_a::<_, CheckButton>(widget) {
                let button = widget.clone().downcast::<CheckButton>().unwrap();
                let current_label = button.label().unwrap();
                if current_label == value.as_str().unwrap() {
                    return button.set_active(true);
                }
            } else if self.is_a::<_, ToggleButton>(widget) {
                let button = widget.clone().downcast::<ToggleButton>().unwrap();
                let current_label = button.label().unwrap();
                if current_label == value.as_str().unwrap() {
                    return button.set_active(true);
                }
            }
            w = w.as_ref().unwrap().next_sibling();

            if (w.is_none()) {
                break;
            }
        }
    }

    fn set_dropdown_value(&self, value: &Value) -> () {
        let dropdown = self.widget.clone().downcast::<DropDown>().unwrap();
        let items = dropdown.model().unwrap().downcast::<StringList>().unwrap();
        let selected = items.iter::<Object>().position(|f| {
            if f.is_ok() {
                let s = f
                    .unwrap()
                    .downcast::<StringObject>()
                    .unwrap()
                    .string()
                    .to_string();
                return s == value.as_str().unwrap().to_string();
            }
            false
        });

        dropdown.set_selected(selected.unwrap() as u32);
    }
}
