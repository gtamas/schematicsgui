use colors_transform::{AlphaColor, Color, Hsl, Rgb};
use relm4::gtk::gdk::RGBA;
use relm4::gtk::glib::{DateTime, GString, TimeZone};
use relm4::gtk::prelude::{
    Cast, ComboBoxExtManual, EntryBufferExtManual, FileChooserExtManual, FileExt, GtkWindowExt, IsA,
};
use relm4::gtk::{
    traits::{
        BoxExt, ButtonExt, CheckButtonExt, ColorChooserExt, DialogExt, EditableExt, EntryExt,
        FileChooserExt, OrientableExt, RangeExt, ScaleExt, SelectionModelExt, TextBufferExt,
        TextViewExt, ToggleButtonExt, WidgetExt,
    },
    Align, Box, Button, Dialog, Entry, EntryBuffer, FileChooserAction, FileChooserDialog,
    FileFilter, Label, ResponseType, SpinButtonUpdatePolicy,
};
use relm4::gtk::{
    Adjustment, ApplicationWindow, Calendar, CheckButton, ColorButton, ColorChooserDialog,
    ComboBoxText, DropDown, EntryIconPosition, Justification, LinkButton, ListView, MultiSelection,
    Orientation, Scale, SpinButton, Switch, TextBuffer, TextView, ToggleButton, Window, WrapMode,
};

use relm4::gtk::gio::File;
use relm4::typed_list_view::TypedListView;

use crate::schema_parsing::{
    ChoiceEntry, ColorEntry, ColorEntryFormat, CurrentValuePosType, DateEntry, DateEntryType,
    FsEntry, IconPositionType, IntOrFloat, JustificationType, MenuEntry, NumericEntry,
    NumericValueType, OrientationType, Primitive, TextEntry, TimeInput,
};
use crate::string_list_item::StringListItem;
use crate::traits::WidgetUtils;

#[derive(Debug)]
pub struct FormUtils;

#[derive(Debug)]
pub struct FormValue<'l> {
    pub name: &'l str,
    pub value: &'l str,
}

impl<'l> FormValue<'l> {
    pub fn new(name: &'l str, value: &'l str) -> FormValue<'l> {
        FormValue { name, value }
    }
}

impl WidgetUtils for FormUtils {}

impl Default for FormUtils {
    fn default() -> Self {
        Self::new()
    }
}

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

    pub fn format_date(format: String, date: &DateTime) -> GString {
        if !format.is_empty() {
            return date
                .format(&format)
                .unwrap_or(date.format("%Y-%m-%d %H:%M:%S").unwrap());
        }
        date.format_iso8601().unwrap()
    }

    fn parse_color(color_str: String) -> RGBA {
        match RGBA::parse(color_str) {
            Ok(c) => c,
            Err(_) => RGBA::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn get_link_button(&self, uri: &str, label: Option<&str>) -> LinkButton {
        let button = LinkButton::with_label(uri, label.unwrap_or(uri));
        button.set_width_request(400);
        button.set_halign(Align::Start);
        button
            .child()
            .unwrap()
            .downcast::<Label>()
            .unwrap()
            .set_xalign(0.0);
        button
    }

    pub fn color_str_to_rgba(color: &str) -> RGBA {
        let default = Rgb::from(0.0, 0.0, 0.0);
        let mut color_value = Rgb::from(default.get_red(), default.get_green(), default.get_blue());
        if color.starts_with('#') {
            color_value = Rgb::from_hex_str(color).unwrap_or(default);
        } else if color.starts_with("rgb") {
            color_value = color.parse::<Rgb>().unwrap_or(default);
        } else if color.contains('%') {
            let hsl = color.parse::<Hsl>().unwrap_or(Hsl::from(0.0, 0.0, 0.0));
            color_value = hsl.to_rgb();
        }

        RGBA::new(
            color_value.get_red() / 255.0,
            color_value.get_green() / 255.0,
            color_value.get_blue() / 255.0,
            color_value.get_alpha(),
        )
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

    fn get_menu_default(items: &[String], default: Option<Primitive>) -> u32 {
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

    fn get_digits(options: &NumericEntry, default: &Option<Primitive>) -> i32 {
        if options.value_type == NumericValueType::Float {
            if default.is_some() {
                let dstr: String = default.clone().unwrap().into();
                let has_decimals = dstr.contains('.');
                if has_decimals {
                    return dstr.split('.').last().unwrap().len() as i32;
                } else {
                    return 0;
                }
            }
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

    pub fn label(
        &self,
        text: &str,
        name: &str,
        align: Option<Justification>,
        css: Option<Vec<&str>>,
    ) -> Label {
        let label = Label::new(Some(text));
        let mut css_classes = css.unwrap_or_default();
        let mut css_default = vec!["label"];
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

        let precision = Self::get_digits(&opts, &default);
        let min_label = self.label(&opts.min.to_string(), "min", None, None);
        let max: f64 = opts.max.into();
        let max_label = self.label(
            &format!("{:1$}", max, precision as usize),
            "max",
            None,
            None,
        );
        let container = Box::default();

        max_label.remove_css_class("label");
        min_label.remove_css_class("label");

        let orientation = match opts.orientation {
            OrientationType::Horizontal => Orientation::Horizontal,
            OrientationType::Vertical => Orientation::Vertical,
        };

        container.set_orientation(orientation);
        container.append(&min_label);

        let slider = Scale::new(orientation, Some(&adjustment));
        slider.set_css_classes(&["slider"]);
        slider.set_widget_name(name);
        let pos_type = opts.show_current.clone().into();

        if orientation == Orientation::Vertical {
            max_label.set_xalign(0.5);
            min_label.set_xalign(0.5);
            slider.set_height_request(opts.slider_height);
        }

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

        container.set_widget_name(name);
        container.append(&slider);
        container.append(&max_label);
        container.set_css_classes(&["slider_input_container"]);

        if orientation == Orientation::Vertical {
            container.set_halign(Align::Start);
        } else {
            container.set_halign(Align::Baseline);
        }

        container.set_valign(Align::Baseline);

        container
    }

    pub fn file_input(
        &self,
        name: &str,
        options: Option<FsEntry>,
        buf: Option<&EntryBuffer>,
    ) -> Box {
        let opts = options.unwrap_or_default();
        let action = {
            if !opts.is_dir {
                if opts.is_new {
                    FileChooserAction::Save
                } else {
                    FileChooserAction::Open
                }
            } else {
                FileChooserAction::SelectFolder
            }
        };

        let title = if let Some(custom_title) = opts.clone().title {
            custom_title
        } else if opts.is_dir {
            String::from("Choose a directory")
        } else {
            String::from("Choose a file")
        };

        let entry = self.text_input(name, None, None);

        if let Some(buf_value) = buf {
            entry.set_buffer(buf_value);
        }

        entry.set_hexpand(true);
        let buffer = entry.buffer();

        let button = self.action_button("file", Some("document-open"));
        button.set_tooltip_text({
            if !opts.is_dir {
                Some("Click to browse files")
            } else {
                Some("Click to browse directories")
            }
        });
        button.connect_clicked(move |b: &Button| {
            let buffer = buffer.clone();
            let win = b.root().unwrap().downcast::<ApplicationWindow>();
            let dialog: FileChooserDialog = if let Ok(win_value) = win {
                FormUtils::new().file_chooser(&title, &win_value, Some(action), Some(opts.clone()))
            } else {
                FormUtils::new().file_chooser(
                    &title,
                    &b.root().unwrap().downcast::<Dialog>().unwrap(),
                    Some(action),
                    Some(opts.clone()),
                )
            };

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
        form.set_widget_name(name);
        form.set_css_classes(&["file_input_container"]);
        form
    }

    pub fn date_dialog_input(
        &self,
        name: &str,
        options: Option<DateEntry>,
        default: Option<Primitive>,
    ) -> Box {
        let opts = options.unwrap_or_default();

        let entry = Entry::new();
        let buffer = EntryBuffer::default();
        let buffer_clone = buffer.clone();
        let button = self.action_button("date", Some("work-week"));

        if default.is_some() {
            let default_date: Option<DateTime> = Some(default.clone().unwrap().into());
            buffer.set_text(Self::format_date(
                opts.format.clone(),
                default_date.as_ref().unwrap(),
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
                let d = DateTime::from_iso8601(&buffer.text(), Some(&TimeZone::utc()));
                calendar.select_day(&d.unwrap_or(DateTime::now_utc().unwrap()));
            }

            calendar.connect_day_selected(move |c| {
                let selection = c.date();
                buffer.set_text(Self::format_date(opts.format.clone(), &selection));
            });
            content.append(&calendar);

            dialog.show();
            dialog.connect_response(move |dialog, resp| {
                if resp == ResponseType::Ok {
                    dialog.close()
                }
            });
        });

        let form = self.browse_button_with_entry(&entry, &button);
        form.set_widget_name(name);
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
        button.set_use_alpha(opts.alpha);
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
        let color: String;

        if default.is_some() {
            color = default.clone().unwrap().into();
            let rgba = Self::parse_color(color);
            buffer.set_text(Self::format_color_str(opts.format.clone(), &rgba));
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
        form.set_widget_name(name);
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
        action: Option<FileChooserAction>,
        options: Option<FsEntry>,
    ) -> FileChooserDialog {
        let opts = options.unwrap_or_default();
        let mut action = action.unwrap_or(FileChooserAction::Open);

        if opts.is_dir {
            action = FileChooserAction::SelectFolder;
        }

        let title: Option<GString> = GString::from_string_unchecked(String::from(title)).into();
        let dialog = FileChooserDialog::new(title, Some(parent), action, &[]);
        let filter = FileFilter::new();

        if opts.current_folder.is_some() {
            let folder = File::for_path(opts.current_folder.unwrap());
            match dialog.set_current_folder(Some(&folder)) {
                Ok(b) => b,
                Err(e) => panic!("{}", e),
            };
        }

        if opts.default_name.is_some() {
            dialog.set_current_name(&opts.default_name.unwrap());
        }

        if opts.multiple {
            dialog.set_select_multiple(true);
        }

        filter.add_pattern(&opts.mask);
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

        if let Some(icon_value) = icon {
            button.set_icon_name(icon_value);
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

        if !opts.icon.is_empty() {
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
        entry.set_icon_tooltip_markup(
            {
                if opts.icon_position == IconPositionType::Start {
                    EntryIconPosition::Primary
                } else {
                    EntryIconPosition::Secondary
                }
            },
            Some(&opts.tooltip),
        );
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
                self.checkbox_or_radio(name, Some(&group), None, Some(Primitive::Bool(active)));
            widget.set_label(Some(value));

            container.append(&widget);
        }

        container.set_widget_name(name);
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
                self.toggle_button(name, Some(&group), None, Some(Primitive::Bool(active)));
            widget.set_label(value);
            container.append(&widget);
        }

        container.set_widget_name(name);
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
            if !opts.label.is_empty() && group.is_none() {
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

        if let Some(default_value) = default {
            checkbox.set_active(default_value.into());
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
            if !opts.label.is_empty() && group.is_none() {
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

        if let Some(default_value) = default {
            button.set_active(default_value.into());
        }

        button
    }

    pub fn switch(
        &self,
        name: &str,
        _options: Option<ChoiceEntry>,
        default: Option<Primitive>,
    ) -> Switch {
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

    pub fn date_input(
        &self,
        name: &str,
        _options: Option<DateEntry>,
        default: Option<Primitive>,
    ) -> Box {
        let container = Box::default();

        container.set_css_classes(&["date_input_container"]);
        container.set_widget_name(name);
        container.set_orientation(Orientation::Vertical);
        container.set_hexpand(false);
        container.set_halign(Align::Start);
        let calendar = Calendar::new();
        calendar.set_hexpand(false);
        calendar.set_vexpand(false);
        calendar.set_halign(Align::Fill);
        calendar.set_valign(Align::Start);

        if default.is_some() {
            let def: String = default.unwrap_or_default().into();
            let d = DateTime::from_iso8601(&def, Some(&TimeZone::utc()));
            calendar.select_day(&d.unwrap_or(DateTime::now_utc().unwrap()));
        }

        container.append(&calendar);
        container
    }

    pub fn date_time_input(
        &self,
        name: &str,
        options: Option<DateEntry>,
        default: Option<Primitive>,
    ) -> Box {
        let container = Box::default();

        container.set_css_classes(&["date_time_input_container"]);
        container.set_widget_name(name);
        container.set_orientation(Orientation::Vertical);
        container.set_hexpand(false);
        container.set_halign(Align::Start);
        let time_input = self.time_input(name, options.clone(), default.clone());
        let calendar = self
            .date_input(name, options, default)
            .first_child()
            .unwrap()
            .downcast::<Calendar>()
            .unwrap();

        container.append(&calendar);
        container.append(&time_input);
        container
    }

    pub fn time_input(
        &self,
        name: &str,
        options: Option<DateEntry>,
        default: Option<Primitive>,
    ) -> Box {
        let opts = options.unwrap_or_default();
        let container = Box::default();

        container.set_widget_name(name);
        container.set_orientation(Orientation::Horizontal);
        container.set_css_classes(&["time_input_container"]);

        if opts.r#type == DateEntryType::DateTime {
            container.set_homogeneous(true);
        }

        let default: TimeInput = default.unwrap_or_default().into();

        let num_opts = NumericEntry {
            max: IntOrFloat::Int(24),
            value_type: NumericValueType::Int,
            wrap: true,
            orientation: OrientationType::Vertical,
            ..Default::default()
        };
        let hour = self.numeric_input(
            name,
            Some(num_opts.clone()),
            Some(Primitive::Int(default.0)),
        );
        let minute = self.numeric_input(
            name,
            Some(NumericEntry {
                max: IntOrFloat::Int(60),
                ..num_opts.clone()
            }),
            Some(Primitive::Int(default.1)),
        );
        let seconds = self.numeric_input(
            name,
            Some(NumericEntry {
                max: IntOrFloat::Int(60),
                ..num_opts
            }),
            Some(Primitive::Int(default.2)),
        );

        container.append(&hour);
        container.append(&minute);
        container.append(&seconds);
        container
    }

    pub fn numeric_input(
        &self,
        name: &str,
        options: Option<NumericEntry>,
        default: Option<Primitive>,
    ) -> SpinButton {
        let opts = options.unwrap_or_default();
        let adjustment = Self::get_adjustment(opts.clone());
        let number_input = SpinButton::new(
            Some(&adjustment),
            0.001,
            Self::get_digits(&opts, &default) as u32,
        );
        number_input.set_width_chars(10);
        number_input.set_orientation(opts.orientation.into());
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

    pub fn multiselect_input(
        &self,
        name: &str,
        items: &[String],
        options: Option<MenuEntry>,
        default: Option<Primitive>,
    ) -> ListView {
        let _opts = options.unwrap_or_default();
        let mut list: TypedListView<StringListItem, MultiSelection> = TypedListView::with_sorting();
        list.view.set_widget_name(name);
        list.view.set_css_classes(&["dropdown"]);
        list.view.enables_rubberband();

        for item in items.iter() {
            list.append(StringListItem::new(item.to_string()));
        }

        let selection = list.selection_model;

        if default.is_some() {
            let d: Vec<String> = default.unwrap().into();
            for item in d.iter() {
                let index: usize = items.iter().position(|r| r == item).unwrap_or(0);
                selection.select_item(index as u32, false);
            }
        }

        list.view
    }

    pub fn dropdown(
        &self,
        name: &str,
        items: &[String],
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
        items: &[String],
        // TODO: Implement searchable and orientation options if it makes sense.
        _options: Option<MenuEntry>,
        default: Option<Primitive>,
    ) -> ComboBoxText {
        let combo = ComboBoxText::with_entry();

        for (_, el) in items.iter().enumerate() {
            combo.append_text(el);
        }

        combo.set_active(Some(Self::get_menu_default(items, default)));

        combo.set_widget_name(name);
        combo.set_css_classes(&["combo"]);
        combo
    }
}
