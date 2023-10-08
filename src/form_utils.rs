use colors_transform::{Color, Rgb};
use relm4::gtk::gdk::RGBA;
use relm4::gtk::glib::{DateTime, GString};
use relm4::gtk::prelude::{
    Cast, ComboBoxExtManual, EntryBufferExtManual, FileExt, GtkWindowExt, IsA,
};
use relm4::gtk::{
    traits::{
        BoxExt, ButtonExt, CheckButtonExt, ColorChooserExt, DialogExt, EditableExt, EntryExt,
        FileChooserExt, OrientableExt, RangeExt, ScaleExt, TextBufferExt, TextViewExt,
        ToggleButtonExt, WidgetExt,
    },
    Align, Box, Button, Dialog, Entry, EntryBuffer, FileChooserAction, FileChooserDialog,
    FileFilter, Label, ResponseType, SpinButtonUpdatePolicy,
};
use relm4::gtk::{
    Adjustment, ApplicationWindow, Calendar, CheckButton, ColorButton, ColorChooserDialog,
    ComboBoxText, DropDown, Orientation, Scale, SpinButton, Switch, TextBuffer, TextView,
    ToggleButton, Window, Justification, WrapMode, EntryIconPosition
};

use crate::schema_parsing::{
    ChoiceEntry, ColorEntry, ColorEntryFormat, CurrentValuePosType, DateEntry, DirEntry, FileEntry,
    IconPositionType, JustificationType, MenuEntry, NumericEntry, NumericValueType,
    OrientationType, Primitive, TextEntry,
};

#[derive(Debug)]
pub struct FormUtils;

impl FormUtils {
    pub fn new() -> Self {
        FormUtils {}
    }

    fn get_adjustment(options: NumericEntry) -> Adjustment {
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
            return date
                .format(&format)
                .unwrap_or(date.format("%Y-%m-%d").unwrap());
        }
        date.format_iso8601().unwrap()
    }

    fn parse_color(color_str: String) -> RGBA {
        match RGBA::parse(color_str) {
            Ok(c) => c,
            Err(_) => RGBA::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn format_color_str(format: ColorEntryFormat, rgba: &RGBA) -> String {
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

    fn get_menu_default(items: &Vec<String>, default: Option<Primitive>) -> u32 {
        let mut index = 0;
        if default.is_some() {
            let d: String = default.unwrap().into();
            index = items.iter().position(|r| r == &d).unwrap_or(0);
        }

        index as u32
    }

    fn get_string_default(default: Option<Primitive>) -> String {
        let mut d: String = String::default();

        if default.is_some() {
            d = default.unwrap().into();
        }

        d
    }

    fn get_digits(options: &NumericEntry) -> i32 {
        if options.value_type == NumericValueType::Float {
            options.clone().precision.into()
        } else {
            0
        }
    }

    pub fn browse_button_with_entry(&self, input: &Entry, button: &Button) -> Box {
        let form = Box::new(relm4::gtk::Orientation::Horizontal, 5);
        form.append(input);
        form.append(button);
        form
    }

    pub fn label(&self, text: &str, name: &str, align: Option<Justification>, css: Option<Vec<&str>>) -> Label {
        let label = Label::new(Some(&text));
        let mut css_classes = css.unwrap_or(vec! []);
        let mut css_default = vec! ["label"];
        css_classes.append(&mut css_default);
        let alignment = align.unwrap_or(Justification::Left);
        label.set_css_classes(&css_classes);
        label.set_widget_name(name);
        
        if alignment == Justification::Left {
           label.set_xalign(0.0);
        } else if alignment == Justification::Right {
           label.set_xalign(1.0);
        } else {
           label.set_xalign(0.5);
        }
       
        label.set_justify(align.unwrap_or(Justification::Left));
        label
    }

    pub fn slider(
        &self,
        name: &str,
        options: Option<NumericEntry>,
        default: Option<Primitive>,
    ) -> Box {
        let opts = options.unwrap_or_default();
        let adjustment = Self::get_adjustment(opts.clone());

        let precision = Self::get_digits(&opts);
        let min_label = self.label(&opts.min.to_string(), "min", None, None);
        let max: f64 = opts.max.into();
        let max_label = self.label(&format!("{:1$}", max, precision as usize), "max", None, None);
        let container = Box::default();

        max_label.remove_css_class("label");
        min_label.remove_css_class("label");

        container.set_orientation(Orientation::Horizontal);
        container.append(&min_label);

        let orientation = match opts.orientation {
            OrientationType::Horizontal => Orientation::Horizontal,
            OrientationType::Vertical => Orientation::Vertical,
        };
        let slider = Scale::new(orientation, Some(&adjustment));
        slider.set_css_classes(&["slider"]);
        slider.set_widget_name(name);
        let pos_type = opts.show_current.clone().into();

        if opts.show_current != CurrentValuePosType::None {
            slider.set_draw_value(true);
            slider.set_value_pos(pos_type);
        }

        for mark in opts.marks {
            slider.add_mark(mark.value, pos_type, mark.text.as_deref());
        }

        slider.set_digits(precision);

        slider.set_hexpand(true);

        if default.is_some() {
            let value: f64 = default.unwrap().into();
            slider.set_value(value);
        }

        container.append(&slider);
        container.append(&max_label);
        container.set_css_classes(&["slider_input_container"]);
        container.set_valign(Align::Baseline);
         container.set_halign(Align::Baseline);
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
        entry.set_hexpand(true);
        let buffer = entry.buffer();

        let button = self.action_button("file", Some("document-open"));
        button.set_tooltip_text({
          if is_file {
            Some("Click to browse files")
          } else {
            Some("Click to browse directories")
          }
         });
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

        let form = self.browse_button_with_entry(&entry, &button);
        form.set_css_classes(&["file_input_container"]);
        form
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
        let button = self.action_button("date", Some("work-week"));

        if default.is_some() {
            default_date = Some(default.clone().unwrap().into());
            buffer.set_text(Self::format_date(
                opts.format.clone(),
                &default_date.as_ref().unwrap(),
            ));
        }

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
            let button = dialog.add_button("OK", ResponseType::Ok);
            button.add_css_class("button");
            button.add_css_class("action");
            dialog.set_modal(true);
            dialog.set_parent(&window);
            dialog.set_transient_for(Some(&window));
            dialog.set_destroy_with_parent(true);
            let content = dialog.content_area();
            let calendar = Calendar::new();

            if default.is_some() {
                calendar.select_day(&default_date.as_ref().unwrap());
                buffer.set_text(Self::format_date(
                    opts.format.clone(),
                    &default_date.as_ref().unwrap(),
                ));
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
        form.set_css_classes(&["date_input_container"]);
        entry.set_buffer(&buffer_clone);
        entry.set_css_classes(&["date_input"]);
        entry.set_hexpand(true);
        entry.set_widget_name(name);
        form
    }

    pub fn color_button(
        &self,
        name: &str,
        options: Option<ColorEntry>,
        default: Option<Primitive>,
    ) -> ColorButton {
        let opts = options.unwrap_or_default();
        let button = ColorButton::new();
        button.set_css_classes(&["color_button"]);
        button.set_widget_name(name);
        button.set_title(&opts.title);
        button.set_modal(true);
        button.set_halign(Align::End);
        button.set_valign(Align::Start);

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
        let button = self.action_button("color", Some("color-picker"));
        let mut rgba: Option<RGBA> = None;
        let color: String;

        if default.is_some() {
            color = default.clone().unwrap().into();
            rgba = Some(Self::parse_color(color));
            buffer.set_text(Self::format_color_str(opts.format.clone(), &rgba.unwrap()));
        }

        let entry_clone = entry.clone();

        button.connect_clicked(move |button| {
            let buffer = buffer.clone();
            let opts = opts.clone();
            let window = button
                .root()
                .unwrap()
                .downcast::<ApplicationWindow>()
                .unwrap();
            let dialog = ColorChooserDialog::new(Some(&opts.title), Some(&window));

            dialog.set_use_alpha(opts.alpha);

            if entry_clone.text() != "" {
                dialog.set_rgba(&Self::parse_color(entry_clone.text().to_string()));
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
        form.set_hexpand(true);
        form.append(&entry);
        form.append(&button);
        form.set_css_classes(&["color_input_container"]);
        entry.set_buffer(&buffer_clone);
        entry.set_css_classes(&["color_input"]);
        entry.set_hexpand(true);
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
            &[]
        );
        let filter = FileFilter::new();
        filter.add_pattern(&pattern);
        dialog.set_filter(&filter);
        let cancel_button = dialog.add_button("Cancel", ResponseType::Cancel);
        cancel_button.add_css_class("button");
        cancel_button.add_css_class("action");
        let select_button = dialog.add_button("Select", ResponseType::Accept);
        select_button.add_css_class("button");
        select_button.add_css_class("action");
        dialog.add_css_class("file_chooser");
        dialog
    }

    pub fn browse_button(&self, name: &str) -> Button {
        let button = Button::default();
        button.set_css_classes(&["button", "browse_button"]);
        button.set_icon_name("document-open");
        button.set_widget_name(name);
        button
    }

     pub fn action_button(&self, name: &str, icon: Option<&str>) -> Button {
        let button = Button::default();
        button.set_css_classes(&["button", {
          if icon.is_some() {
            "action_icon"
          } else {
            "action"
          }
        }]);
        
        if icon.is_some() {
          button.set_icon_name(icon.unwrap());
        }

        button.set_widget_name(name);
        button
    }

    pub fn text_input(
        &self,
        name: &str,
        options: Option<TextEntry>,
        default: Option<Primitive>,
    ) -> Entry {
        // TODO: implement hint_text!
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
        entry.set_input_purpose(opts.purpose.into());
        entry.set_input_hints(opts.hint.into());
        entry.set_truncate_multiline(true);
        entry.set_tooltip_markup(Some(&opts.tooltip));
        entry.set_icon_tooltip_markup({
          if opts.icon_position == IconPositionType::Start {
            EntryIconPosition::Primary
          } else {
            EntryIconPosition::Secondary
          }
        }, Some(&opts.tooltip));
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
         // TODO: implement all relevant options from text_input!
        let opts = options.unwrap_or_default();
        let buffer = TextBuffer::default();
        let entry = TextView::with_buffer(&buffer);
        entry.set_css_classes(&["textarea_input"]);
        entry.set_widget_name(name);
        entry.set_editable(true);
        entry.set_height_request(opts.height);
        entry.set_visible(true);
        entry.set_input_purpose(opts.purpose.into());
        entry.set_input_hints(opts.hint.into());
        entry.set_wrap_mode(WrapMode::Word);

        if opts.justify != JustificationType::None {
            entry.set_justification(opts.justify.into());
        }

        entry.set_overwrite(opts.overwrite);

        if default.is_some() {
            let d: String = default.unwrap().into();
            buffer.set_text(&d);
        }

        entry
    }

    pub fn radio_group(
        &self,
        name: &str,
        items: &Vec<String>,
        options: Option<MenuEntry>,
        default: Option<Primitive>,
    ) -> Box {
        let opts = options.unwrap_or_default();
        let group = CheckButton::default();
        let container = Box::default();

        container.set_orientation(opts.orientation.into());

        let d: String = Self::get_string_default(default);

        for value in items {
            let mut active = false;

            if &d == value {
                active = true;
            }

            let widget =
                self.checkbox_or_radio(&name, Some(&group), None, Some(Primitive::Bool(active)));
            widget.set_label(Some(value));

            container.append(&widget);
        }

        container.set_css_classes(&["radio_group_container"]);
        container
    }

    pub fn toggle_group(
        &self,
        name: &str,
        items: &Vec<String>,
        options: Option<MenuEntry>,
        default: Option<Primitive>,
    ) -> Box {
        let opts = options.unwrap_or_default();
        let group = ToggleButton::default();
        let container = Box::default();

        container.set_orientation(opts.orientation.into());

        let d: String = Self::get_string_default(default);

        for value in items {
            let mut active = false;

            if &d == value {
                active = true;
            }

            let widget =
                self.toggle_button(&name, Some(&group), None, Some(Primitive::Bool(active)));
            widget.set_label(&value);
            container.append(&widget);
        }

        container.set_css_classes(&["toggle_group_container"]);
        container
    }

    pub fn checkbox_or_radio(
        &self,
        name: &str,
        group: Option<&CheckButton>,
        options: Option<ChoiceEntry>,
        default: Option<Primitive>,
    ) -> CheckButton {
        let opts = options.unwrap_or_default();
        let checkbox = {
            if opts.label != "" && !group.is_some() {
                CheckButton::with_label(&opts.label)
            } else {
                CheckButton::new()
            }
        };
        checkbox.set_inconsistent(false);
        checkbox.set_widget_name(name);
        if group.is_some() {
            checkbox.set_css_classes(&["radio"]);
            checkbox.set_group(group);
        } else {
            checkbox.set_css_classes(&["checkbox"]);
        }

        if default.is_some() {
            checkbox.set_active(default.unwrap().into());
        }

        checkbox
    }

    pub fn toggle_button(
        &self,
        name: &str,
        group: Option<&ToggleButton>,
        options: Option<ChoiceEntry>,
        default: Option<Primitive>,
    ) -> ToggleButton {
        let opts = options.unwrap_or_default();
        let button = {
            if opts.label != "" && !group.is_some() {
                ToggleButton::with_label(&opts.label)
            } else {
                ToggleButton::new()
            }
        };
        button.set_widget_name(name);
        button.set_halign(Align::End);
        button.set_valign(Align::Start);
        button.set_css_classes(&["toggle_button", "button", "action"]);
        if group.is_some() {
            button.set_group(group);
        }

        if default.is_some() {
            button.set_active(default.unwrap().into());
        }

        button
    }

    pub fn switch(&self, name: &str, default: Option<Primitive>) -> Switch {
        let switch = Switch::new();
        switch.set_widget_name(name);
        switch.set_css_classes(&["switch"]);
        switch.set_halign(Align::End);
        switch.set_valign(Align::Start);

        if default.is_some() {
            let d = default.unwrap().into();
            switch.set_active(d);
        }

        switch
    }

    pub fn numeric_input(
        &self,
        name: &str,
        options: Option<NumericEntry>,
        default: Option<Primitive>,
    ) -> SpinButton {
        let opts = options.unwrap_or_default();
        let adjustment = Self::get_adjustment(opts.clone());
        let number_input =
            SpinButton::new(Some(&adjustment), 0.001, Self::get_digits(&opts) as u32);
        number_input.set_width_chars(10);
        number_input.set_max_width_chars(10);
        number_input.set_widget_name(name);
        number_input.set_css_classes(&["numeric_input"]);
        number_input.set_numeric(true);
        number_input.set_wrap(opts.wrap);
        number_input.set_snap_to_ticks(true);
        number_input.set_update_policy(SpinButtonUpdatePolicy::IfValid);

        if default.is_some() {
            let d = default.unwrap().into();
            number_input.set_value(d);
        }

        number_input
    }

    pub fn dropdown(
        &self,
        name: &str,
        items: &Vec<String>,
        options: Option<MenuEntry>,
        default: Option<Primitive>,
    ) -> DropDown {
        let opts = options.unwrap_or_default();
        let dropdown = DropDown::from_strings(
            items
                .iter()
                .map(|s| s.as_ref())
                .collect::<Vec<&str>>()
                .as_slice(),
        );

        dropdown.set_widget_name(name);
        dropdown.set_css_classes(&["dropdown"]);

        if opts.searchable {
            dropdown.set_enable_search(true);
        }

        dropdown.set_selected(Self::get_menu_default(items, default));

        dropdown
    }

    pub fn combobox_text(
        &self,
        name: &str,
        items: &Vec<String>,
        options: Option<MenuEntry>,
        default: Option<Primitive>,
    ) -> ComboBoxText {
        let combo = ComboBoxText::with_entry();

        for (i, el) in items.iter().enumerate() {
            combo.append_text(&el);
        }

        combo.set_active(Some(Self::get_menu_default(items, default)));

        combo.set_widget_name(name);
        combo.set_css_classes(&["combo"]);
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
