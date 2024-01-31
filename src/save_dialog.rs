use relm4::gtk::prelude::{
    DialogExt, EntryBufferExtManual, EntryExt, FrameExt, GtkWindowExt, OrientableExt, WidgetExt,
};
use relm4::gtk::EntryBuffer;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};
use std::fs::write;
use std::path::PathBuf;
use toml::Table;

use crate::form_utils::FormUtils;
use crate::impl_validation;
use crate::schema_parsing::FsEntry;
use crate::settings_utils::SettingsUtils;
use crate::traits::Validator;
use toml::{map::Map, Value};

#[tracker::track]
#[derive(Debug)]
pub struct SaveDialogModel {
    hidden: bool,
    desc_buf: EntryBuffer,
    file_name_buf: EntryBuffer,
    data: String,
    schematic: String,
    package_name: String,
    success: bool,
    error: bool,
    message: String,
}

#[derive(Debug)]
pub struct SaveDialogInputParams {
    pub form_data: String,
    pub schematic: String,
    pub package_name: String,
    pub file: Option<String>,
    pub description: Option<String>,
    pub auto_save: bool,
}

#[derive(Debug)]
pub struct ProfileData {
    pub title: String,
    pub file: String,
    pub data: Map<String, Value>,
}

impl ProfileData {
    pub fn new(title: String, file: String, data: Map<String, Value>) -> Self {
        ProfileData { title, file, data }
    }
}

impl_validation!(SaveDialogModel);

impl SaveDialogModel {
    fn write(&self) {
        let toml = match format!(
            "[meta]\ndescription='{}'\n[data]\n{}",
            self.desc_buf.text(),
            self.data
        )
        .parse::<Table>()
        {
            Ok(s) => s,
            Err(err) => panic!("Could not parse TOML! {}", err),
        };

        let toml_str = toml::to_string_pretty(&toml).unwrap();
        let dir = self.create_config_dir();
        let file = self.file_name_buf.text();
        let file_path = dir.join(format!("{}", file));
        match write(file_path.as_os_str(), toml_str) {
            Ok(s) => s,
            Err(err) => panic!("Could not save file! {}", err),
        }
    }

    fn get_config_dir(&self) -> PathBuf {
        SettingsUtils::get_config_dir()
            .join(self.get_package_name())
            .join(self.get_schematic())
    }

    fn get_config_dir_as_string(&self) -> String {
        String::from(self.create_config_dir().to_str().unwrap())
    }

    fn create_config_dir(&self) -> PathBuf {
        let config_dir: PathBuf = self.get_config_dir();
        if !config_dir.exists() {
            match std::fs::create_dir_all(&config_dir) {
                Ok(s) => s,
                Err(err) => panic!("Could not create settings dir! {}", err),
            }
        }
        config_dir
    }

    fn reset_form(&mut self) {
        self.desc_buf.set_text("");
        self.file_name_buf.set_text("");
    }

    fn validate(&mut self) -> bool {
        let file = self.file_name_buf.text();
        let path = PathBuf::from(file.clone());

        if file.len() == 0 {
            self.print_error("The 'file' field is mandatory!");
            return false;
        } else if path.extension().unwrap_or_default() != "toml" {
            self.print_error(&format!("The '{}' is not a TOML file!", file));
            return false;
        }

        true
    }
}

#[derive(Debug)]
pub enum SaveDialogInput {
    Show(SaveDialogInputParams),
    Cancel,
    Apply,
}

#[derive(Debug)]
pub enum SaveDialogOutput {
    Apply(String),
}

pub struct SaveDialogInit {}

#[relm4::component(pub)]
impl SimpleComponent for SaveDialogModel {
    type Input = SaveDialogInput;
    type Output = SaveDialogOutput;
    type Init = bool;

    view! {
         gtk::Dialog {
            set_title: Some("Save"),
            set_default_height: 200,
            set_default_width: 600,
            set_modal: true,
            set_destroy_with_parent: true,
            set_css_classes: &["settings_dialog"],
            #[watch]
            set_visible: !model.hidden,
            add_button: ("Save", gtk::ResponseType::Apply),
            add_button: ("Cancel", gtk::ResponseType::Cancel),
            gtk::Box {
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
              gtk::Label {
                set_hexpand: true,
                set_vexpand: false,
                set_css_classes: &["label"],
                set_halign: gtk::Align::Start,
                set_label: "File name"
              },
              gtk::Frame {
                set_css_classes: &["invisible"],
                #[track = "model.changed(SaveDialogModel::schematic())"]
                set_child: Some(&FormUtils::new().file_input("file", Some(FsEntry {
                  is_new: true,
                  current_folder: Some(model.get_config_dir_as_string()),
                  mask: String::from("*.toml"),
                  ..Default::default()
                }), Some(&model.file_name_buf)))
              },
              gtk::Label {
                set_hexpand: true,
                set_vexpand: false,
                set_css_classes: &["label"],
                set_halign: gtk::Align::Start,
                set_label: "Description"
              },
              gtk::Entry {
                set_hexpand: true,
                set_css_classes: &["text_input", "setting_description"],
                set_buffer: &model.desc_buf
              }
            },
            connect_response[sender] => move |dialog, resp| {
                dialog.set_default_height(200);
                sender.input(if resp == gtk::ResponseType::Apply {
                    SaveDialogInput::Apply
                } else {
                    SaveDialogInput::Cancel
                })

            },
            connect_close_request[sender] => move |dialog| {
                sender.input(SaveDialogInput::Cancel);
                dialog.set_default_height(200);
                gtk::Inhibit(true)
            }

        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SaveDialogModel {
            hidden: true,
            data: String::default(),
            desc_buf: EntryBuffer::default(),
            file_name_buf: EntryBuffer::default(),
            schematic: String::default(),
            package_name: String::default(),
            tracker: 0,
            error: false,
            success: false,
            message: String::default(),
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            SaveDialogInput::Show(data) => {
                self.data = data.form_data;
                self.set_schematic(data.schematic);
                self.set_package_name(data.package_name);
                self.file_name_buf
                    .set_text(data.file.unwrap_or(String::default()));
                self.desc_buf
                    .set_text(data.description.unwrap_or(String::default()));

                if !data.auto_save {
                    self.hidden = false;
                } else {
                    sender.input_sender().emit(SaveDialogInput::Apply);
                }
            }
            SaveDialogInput::Apply => {
                if !self.validate() {
                    return;
                }

                self.set_error(false);

                self.write();
                sender
                    .output(SaveDialogOutput::Apply(
                        self.file_name_buf.text().to_string(),
                    ))
                    .unwrap();
                self.hidden = true;
                self.reset_form();
            }
            SaveDialogInput::Cancel => {
                self.hidden = true;
                self.reset_form();
                self.clear_error();
                self.set_error(false);
            }
        }
    }
}
