use std::f64::MAX;

use colors_transform::{Color, Rgb};
use relm4::gtk::gdk::RGBA;
use relm4::gtk::glib::{DateTime, GString};
use relm4::gtk::prelude::{Cast, EntryBufferExtManual, FileExt, GtkWindowExt, IsA};
use relm4::gtk::{
    traits::{
        BoxExt, ButtonExt, CheckButtonExt, ColorChooserExt, DialogExt, EditableExt, EntryExt,
        FileChooserExt, OrientableExt, RangeExt, TextBufferExt, TextViewExt, ToggleButtonExt,
        WidgetExt,
    },
    Align, Box, Button, Dialog, Entry, EntryBuffer, FileChooserAction, FileChooserDialog,
    FileFilter, Label, ResponseType,
};
use relm4::gtk::{
    Adjustment, ApplicationWindow, Calendar, CheckButton, ColorButton, ColorChooserDialog,
    ComboBoxText, DropDown, Orientation, Scale, SpinButton, Switch, TextBuffer, TextView,
    ToggleButton, Window,
};

use crate::schematic_ui::{
    ColorEntry, ColorEntryFormat, DateEntry, DirEntry, FileEntry, NumericEntry, NumericType,
    OrientationType, Primitive, TextEntry, IconPositionType,
};

#[derive(Debug)]
pub struct FormUtils;

impl FormUtils {
    pub fn new() -> Self {
        FormUtils {}
    }

    fn browse_button_with_entry(&self, input: &Entry, button: &Button) -> Box {
        let form = Box::new(relm4::gtk::Orientation::Horizontal, 5);
        form.append(input);
        form.append(button);
        form
    }

    fn get_adjustment(&self, options: NumericEntry) -> Adjustment {
        Adjustment::new(
            options.initial_value.into(),
            options.min.into(),
            options.max.into(),
            options.stepping.into(),
            options.page_increment.into(),
            options.page_size.into(),
        )
    }

    fn format_date(format: String, date: &DateTime) -> GString {
        if format != "" {
            return date.format(&format).unwrap_or(date.format("%Y-%m-%d").unwrap());
        }
        date.format_iso8601().unwrap()
    }

     fn parse_color(color_str: String) -> RGBA {
        match RGBA::parse(color_str) {
            Ok(c) => c,
            Err(_) => RGBA::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    fn format_color_str(format: ColorEntryFormat, rgba: &RGBA) -> String {
        let color = Rgb::from(
            rgba.red() * 255.0,
            rgba.green() * 255.0,
            rgba.blue() * 255.0,
        );

        if format == ColorEntryFormat::RGB {
            return rgba.to_string();
        } else if format == ColorEntryFormat::Hex {
            return color.to_css_hex_string();
        } else if format == ColorEntryFormat::HSL {
            return color.to_hsl().to_css_string();
        }

        rgba.to_string()
    }

    pub fn label(&self, text: &str, name: &str, align: Option<Align>) -> Label {
        let label = Label::new(Some(&text));
        label.set_css_classes(&["label"]);
        label.set_widget_name(name);
        label.set_halign(align.unwrap_or(Align::End));
        label
    }

    pub fn slider(
        &self,
        name: &str,
        options: Option<NumericEntry>,
        default: Option<Primitive>,
    ) -> Box {
        let opts = options.unwrap_or_default();
        let adjustment = self.get_adjustment(opts.clone());

        let min_label = self.label(&opts.min.to_string(), "min", None);
        let max_label = self.label(&opts.max.to_string(), "max", None);
        let label = max_label.clone();
        let container = Box::default();
        container.set_orientation(Orientation::Horizontal);
        container.append(&min_label);

        let orientation = match opts.orientation {
            OrientationType::Horizontal => Orientation::Horizontal,
            OrientationType::Vertical => Orientation::Vertical,
        };
        let slider = Scale::new(orientation, Some(&adjustment));
        slider.set_css_classes(&["slider"]);
        slider.set_widget_name(name);

        // TODO: width from CSS!
        slider.set_hexpand(true);
        slider.connect_value_changed(move |x| {
            if opts.r#type == NumericType::Float {
                label.clone().set_text(&format!("{:.2}", x.value()));
            } else {
                label.clone().set_text(&format!("{}", x.value() as i32));
            }
        });

        if default.is_some() {
            let value: f64 = default.unwrap().into();
            slider.set_value(value);
        }

        container.append(&slider);
        container.append(&max_label);
        container
    }

    pub fn file_input(
        &self,
        name: &str,
        file_opts: Option<FileEntry>,
        dir_opts: Option<DirEntry>,
    ) -> Box {
        let is_file = file_opts.is_some();

        if file_opts.is_some() && dir_opts.is_some() {
            panic!("Invalid call");
        }

        let entry = self.text_input(name, None, None);
        let buffer = entry.buffer();

        let button = self.browse_button("file");
        button.connect_clicked(move |b: &Button| {
            let buffer = buffer.clone();
            let window = b.root().unwrap().downcast::<ApplicationWindow>().unwrap();
            let dialog = FormUtils::new().file_chooser(
                "Choose a file",
                &window,
                is_file,
                Some(
                    {
                        if is_file {
                            file_opts.clone().unwrap().mask
                        } else {
                            dir_opts.clone().unwrap().mask
                        }
                    }
                    .as_str(),
                ),
            );
            dialog.show();
            dialog.connect_response(move |dialog, resp| match resp {
                ResponseType::Cancel => dialog.close(),
                ResponseType::Accept => {
                    let file_name = dialog.file().unwrap().parse_name().to_string();
                    buffer.set_text(file_name);
                    dialog.close();
                }
                _ => (),
            });
        });

        self.browse_button_with_entry(&entry, &button)
    }

    pub fn date_input(
        &self,
        name: &str,
        options: Option<DateEntry>,
        default: Option<Primitive>,
    ) -> Box {
        let opts = options.unwrap_or_default();
        let mut default_date: Option<DateTime> = None;

        let entry = Entry::new();
        let buffer = EntryBuffer::default();
        let buffer_clone = buffer.clone();
        let button = self.browse_button("date");

        if default.is_some() {
            default_date = Some(default.clone().unwrap().into());
            buffer.set_text(Self::format_date(opts.format.clone(), &default_date.as_ref().unwrap()));
          }

        button.set_icon_name("work-week");
        button.connect_clicked(move |button| {
            let buffer = buffer.clone();
            let opts = opts.clone();
            let window: ApplicationWindow = button
                .root()
                .unwrap()
                .downcast::<ApplicationWindow>()
                .unwrap();
            let dialog = Dialog::new();
            dialog.set_title(Some("Select a date"));
            dialog.add_button("OK", ResponseType::Ok);
            dialog.set_modal(true);
            dialog.set_parent(&window);
            dialog.set_transient_for(Some(&window));
            dialog.set_destroy_with_parent(true);
            let content = dialog.content_area();
            let calendar = Calendar::new();

            if default.is_some() {
                calendar.select_day(&default_date.as_ref().unwrap());
                buffer.set_text(Self::format_date(opts.format.clone(), &default_date.as_ref().unwrap()));
            }

            calendar.connect_day_selected(move |c| {
                let selection = c.date();
                buffer.set_text(Self::format_date(opts.format.clone(), &selection));
            });
            content.append(&calendar);

            dialog.show();
            dialog.connect_response(move |dialog, resp| match resp {
                ResponseType::Ok => dialog.close(),
                _ => (),
            });
        });

        let form = self.browse_button_with_entry(&entry, &button);
        entry.set_buffer(&buffer_clone);
        entry.set_css_classes(&["date_input"]);
        entry.set_widget_name(name);
        form
    }

    pub fn color_button(
        &self,
        name: &str,
        _options: Option<ColorEntry>,
        default: Option<Primitive>,
    ) -> ColorButton {
        let button = ColorButton::new();
        button.set_css_classes(&["color_button"]);
        button.set_widget_name(name);

        if default.is_some() {
            let color: String = default.unwrap().into();
            button.set_rgba(&Self::parse_color(color));
        }

        button
    }

    pub fn color_input(
        &self,
        name: &str,
        options: Option<ColorEntry>,
        default: Option<Primitive>,
    ) -> Box {
        let opts = options.unwrap_or_default();

        let entry = Entry::new();
        let buffer = EntryBuffer::default();
        let buffer_clone = buffer.clone();
        let button = self.browse_button("color");
        let mut rgba: Option<RGBA> = None;
        let color: String;

          if default.is_some() {
                color = default.clone().unwrap().into();
                rgba = Some(Self::parse_color(color));
                buffer.set_text(Self::format_color_str(opts.format.clone(), &rgba.unwrap()));
          }

        button.set_icon_name("color-picker");
        button.connect_clicked(move |button| {
            let buffer = buffer.clone();
            let opts = opts.clone();
            let window = button
                .root()
                .unwrap()
                .downcast::<ApplicationWindow>()
                .unwrap();
            let dialog = ColorChooserDialog::new(Some("Choose a color"), Some(&window));

            dialog.set_use_alpha(opts.alpha);

            if default.is_some() {
                dialog.set_rgba(&rgba.unwrap());
            }

            dialog.show();
            dialog.connect_response(move |colors, resp| match resp {
                ResponseType::Cancel => colors.close(),
                ResponseType::Ok => {
                    let selected = colors.rgba();
                    buffer.set_text(Self::format_color_str(opts.format.clone(), &selected));
                    colors.close();
                }
                _ => (),
            });
        });

        let form = Box::new(relm4::gtk::Orientation::Horizontal, 5);
        form.append(&entry);
        form.append(&button);
        entry.set_buffer(&buffer_clone);
        entry.set_css_classes(&["color_input"]);
        entry.set_widget_name(name);
        form
    }

    pub fn file_chooser(
        &self,
        title: &str,
        parent: &impl IsA<Window>,
        is_file: bool,
        filter: Option<&str>,
    ) -> FileChooserDialog {
        let action = {
            if is_file {
                FileChooserAction::Open
            } else {
                FileChooserAction::SelectFolder
            }
        };

        let pattern = match filter {
            Some(s) => s,
            None => "*",
        };
        let title: Option<GString> = GString::from_string_unchecked(String::from(title)).into();
        let dialog = FileChooserDialog::new(
            title,
            Some(parent),
            action,
            &[
                ("Cancel", ResponseType::Cancel),
                ("Select", ResponseType::Accept),
            ],
        );
        let filter = FileFilter::new();
        filter.add_pattern(&pattern);
        dialog.set_filter(&filter);
        dialog
    }

    pub fn browse_button(&self, name: &str) -> Button {
        let button = Button::default();
        button.set_css_classes(&["button", "browse_button"]);
        button.set_icon_name("document-open");
        button.set_widget_name(name);
        button
    }

    pub fn text_input(
        &self,
        name: &str,
        options: Option<TextEntry>,
        default: Option<Primitive>,
    ) -> Entry {
        let opts = options.unwrap_or_default();
        let buffer = EntryBuffer::default();
        let entry = Entry::with_buffer(&buffer);
        entry.set_css_classes(&["text_input"]);
        entry.set_widget_name(name);
        
        if opts.icon != "" {
          if opts.icon_position == IconPositionType::Start {
             entry.set_primary_icon_activatable(false);
             entry.set_primary_icon_name(Some(&opts.icon));
          } else {
            entry.set_secondary_icon_activatable(false);
            entry.set_secondary_icon_name(Some(&opts.icon));
          }
        }

        entry.set_truncate_multiline(true);
        entry.set_tooltip_markup(Some(&opts.tooltip));
        entry.set_placeholder_text(Some(&opts.placeholder));
        entry.set_max_length(opts.max_len);
        entry.set_overwrite_mode(opts.overwrite);
        EntryExt::set_alignment(&entry, opts.direction.into());
       
        if default.is_some() {
            let d: String = default.unwrap().into();
            buffer.set_text(d);
        }

        entry
    }

    pub fn textarea_input(
        &self,
        name: &str,
        options: Option<TextEntry>,
        default: Option<Primitive>,
    ) -> TextView {
        let opts = options.unwrap_or_default();
        let buffer = TextBuffer::default();
        let entry = TextView::with_buffer(&buffer);
        entry.set_css_classes(&["textarea_input"]);
        entry.set_widget_name(name);
        entry.set_editable(true);
        entry.set_height_request(opts.height);
        entry.set_visible(true);

        if default.is_some() {
            let d: String = default.unwrap().into();
            buffer.set_text(&d);
        }

        entry
    }

    pub fn radio_group(&self, name: &str, items: &Vec<String>) -> Box {
        let group = CheckButton::default();
        let container = Box::default();
        container.set_orientation(Orientation::Vertical);

        for value in items {
            let widget = self.checkbox_or_radio(&name, Some(&group), None);
            widget.set_label(Some(value));
            container.append(&widget);
        }

        container
    }

    pub fn toggle_group(&self, name: &str, items: &Vec<String>) -> Box {
        let group = ToggleButton::default();
        let container = Box::default();
        container.set_orientation(Orientation::Vertical);

        for value in items {
            let widget = self.toggle_button(&name, Some(&group), None);
            widget.set_label(&value);
            container.append(&widget);
        }

        container
    }

    pub fn checkbox_or_radio(
        &self,
        name: &str,
        group: Option<&CheckButton>,
        default: Option<Primitive>,
    ) -> CheckButton {
        let checkbox = CheckButton::new();
        checkbox.set_inconsistent(false);
        checkbox.set_widget_name(name);
        if group.is_some() {
            checkbox.set_css_classes(&["radio"]);
            checkbox.set_group(group);
        } else {
            checkbox.set_css_classes(&["checkbox"]);
        }

        if default.is_some() {
            let d = default.unwrap().into();
            checkbox.set_active(d);
        }

        checkbox
    }

    pub fn toggle_button(
        &self,
        name: &str,
        group: Option<&ToggleButton>,
        default: Option<Primitive>,
    ) -> ToggleButton {
        let button = ToggleButton::new();
        button.set_widget_name(name);
        button.set_css_classes(&["toggle_button", "button"]);
        if group.is_some() {
            button.set_group(group);
        }

        if default.is_some() {
            let d = default.unwrap().into();
            button.set_active(d);
        }

        button
    }

    pub fn switch(&self, css_class: &str, name: &str, default: Option<Primitive>) -> Switch {
        let switch = Switch::new();
        switch.set_widget_name(name);
        switch.set_css_classes(&[css_class]);
        switch.set_hexpand(false);

        if default.is_some() {
            let d = default.unwrap().into();
            switch.set_active(d);
        }

        switch
    }

    pub fn numeric_input(
        &self,
        css_class: &str,
        name: &str,
        min: Option<f64>,
        max: Option<f64>,
        stepping: Option<f64>,
        default: Option<Primitive>,
    ) -> SpinButton {
        let adjustment = Adjustment::new(
            0.0,
            min.unwrap_or(0.0),
            max.unwrap_or(MAX),
            stepping.unwrap_or(1.0),
            5.0,
            0.0,
        );
        let number_input = SpinButton::new(Some(&adjustment), 0.001, {
            if default.is_some() {
                match default.clone().unwrap() {
                    Primitive::Float(_) => 2,
                    _ => 0,
                }
            } else {
                0
            }
        });
        number_input.set_width_chars(10);
        number_input.set_max_width_chars(10);
        number_input.set_widget_name(name);
        number_input.set_css_classes(&[css_class]);

        if default.is_some() {
            let d = match default.unwrap() {
                Primitive::Float(x) => x,
                Primitive::Int(x) => f64::from(x),
                _ => 0.0,
            };
            number_input.set_value(d);
        }

        number_input
    }

    pub fn dropdown(
        &self,
        css_class: &str,
        name: &str,
        items: &Vec<String>,
        default: Option<Primitive>,
    ) -> DropDown {
        let dropdown = DropDown::from_strings(
            items
                .iter()
                .map(|s| s.as_ref())
                .collect::<Vec<&str>>()
                .as_slice(),
        );

        dropdown.set_widget_name(name);
        dropdown.set_css_classes(&[css_class]);

        if default.is_some() {
            let d: String = default.unwrap().into();
            let index = items.iter().position(|r| r == &d).unwrap_or(0);
            dropdown.set_selected(index as u32);
        }

        dropdown
    }

    pub fn combobox_text(&self, css_class: &str, name: &str, items: Vec<String>) -> ComboBoxText {
        let combo = ComboBoxText::with_entry();

        for (i, el) in items.iter().enumerate() {
            combo.append_text(&el);
        }

        combo.set_widget_name(name);
        combo.set_css_classes(&[css_class]);
        combo
    }

    //  pub fn combobox(&self, css_class: &str, name: &str, items: Vec<SchemaItem>) -> DropDown {

    //     // let model = StringList::new(items.iter().map(|s| s.as_ref()).collect::<Vec<&str>>().as_slice() );

    //     let store = ListStore::new(String::static_type());
    //     let renderer = CellRendererText::new();
    //     let col = TreeViewColumn::new();
    //     col.set_title("Picture");
    //     col.pack_start(&renderer, false);
    //     col.add_attribute(&renderer, "text", 0);

    //      for v in items {
    //       store.insert(0, &StringListItem::new("".to_string()));
    //      }

    //     let combo = DropDown::new(Some(store), None);

    //     combo.set_widget_name(name);
    //     combo.set_css_classes(&[css_class]);
    //     combo
    // }
}