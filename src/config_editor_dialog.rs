use relm4::gtk::prelude::{
    DialogExt, GtkWindowExt, OrientableExt, TextBufferExt, TextViewExt, WidgetExt,
};
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};
use serde_json::Value;
use std::fs::{self, write};

use crate::impl_validation;
use crate::traits::{JsonBuffer, Validator};
use sourceview5::prelude::ViewExt;
use sourceview5::Buffer;

#[tracker::track]
#[derive(Debug)]
pub struct ConfigEditorDialogModel {
    hidden: bool,
    config_file: String,
    json_buf: Buffer,
    success: bool,
    error: bool,
    message: String,
}

impl_validation!(ConfigEditorDialogModel);

impl JsonBuffer for ConfigEditorDialogModel {}

impl ConfigEditorDialogModel {
    fn get_text_view_content(&self) -> String {
        let start = self.json_buf.start_iter();
        let end = self.json_buf.end_iter();
        self.json_buf.text(&start, &end, false).to_string()
    }

    fn write(&self) {
        match write(
            self.config_file.as_str(),
            self.get_text_view_content().as_bytes(),
        ) {
            Ok(s) => s,
            Err(err) => panic!("Could not save file! {}", err),
        }
    }

    fn validate(&mut self) -> bool {
        let content = self.get_text_view_content();
        match serde_json::from_str::<Value>(&content) {
            Ok(s) => s,
            Err(err) => {
                self.print_error(&format!("Invalid JSON! {}", err));
                return false;
            }
        };

        if content.is_empty() {
            self.print_error("The JSON content is mandatory!");
            return false;
        }

        true
    }
}

#[derive(Debug)]
pub enum ConfigEditorDialogInput {
    Show(String),
    Cancel,
    Apply,
}

#[derive(Debug)]
pub enum ConfigEditorDialogOutput {
    Apply(String),
}

pub struct ConfigEditorInit {}

#[relm4::component(pub)]
impl SimpleComponent for ConfigEditorDialogModel {
    type Input = ConfigEditorDialogInput;
    type Output = ConfigEditorDialogOutput;
    type Init = bool;

    view! {
         gtk::Dialog {
            set_title: Some("Config Editor"),
            set_default_height: 600,
            set_default_width: 600,
            set_modal: true,
            set_destroy_with_parent: true,
            set_css_classes: &["config_editor_dialog"],
            #[watch]
            set_visible: !model.hidden,
            add_button: ("Save", gtk::ResponseType::Apply),
            add_button: ("Cancel", gtk::ResponseType::Cancel),
            gtk::Box {
              set_hexpand: true,
              set_vexpand: true,
              set_orientation: gtk::Orientation::Vertical,
              set_css_classes: &["dialog_container"],
              gtk::Revealer {
                set_transition_type: gtk::RevealerTransitionType::SlideDown,
                #[watch]
                set_reveal_child: model.error,
                gtk::Label {
                  set_hexpand: true,
                  set_vexpand: false,
                  set_css_classes: &["label", "error"],
                  set_halign: gtk::Align::Center,
                  #[watch]
                  set_label: &format!("Error: {}", &model.message)
                }
              },
              gtk::ScrolledWindow {
              set_hscrollbar_policy: gtk::PolicyType::Never,
                sourceview5::View {
                  set_editable: true,
                  set_hexpand: true,
                  set_vexpand: true,
                  set_show_line_numbers: true,
                  set_highlight_current_line: true,
                  set_tab_width: 4,
                  set_monospace: true,
                  set_buffer: Some(&model.json_buf)
                }
              }
            },
            connect_response[sender] => move |dialog, resp| {
                dialog.set_default_height(600);
                sender.input(if resp == gtk::ResponseType::Apply {
                    ConfigEditorDialogInput::Apply
                } else {
                    ConfigEditorDialogInput::Cancel
                })

            },
            connect_close_request[sender] => move |dialog| {
                sender.input(ConfigEditorDialogInput::Cancel);
                dialog.set_default_height(600);
                gtk::Inhibit(true)
            }

        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = ConfigEditorDialogModel {
            hidden: true,
            json_buf: Self::get_json_buffer(None),
            config_file: String::default(),
            tracker: 0,
            error: false,
            success: false,
            message: String::default(),
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            ConfigEditorDialogInput::Show(path) => {
                self.config_file = path.clone();
                let config: Value =
                    serde_json::from_str(fs::read_to_string(path).unwrap().as_str()).unwrap();

                self.json_buf
                    .set_text(&serde_json::to_string_pretty(&config).unwrap());
                self.hidden = false
            }
            ConfigEditorDialogInput::Apply => {
                if !self.validate() {
                    return;
                }

                self.set_error(false);
                self.write();
                self.hidden = true;
            }
            ConfigEditorDialogInput::Cancel => {
                self.hidden = true;
                self.clear_error();
                self.set_error(false);
            }
        }
    }
}
