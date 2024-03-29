use is_executable::IsExecutable;
use std::ops::{Index, IndexMut};
use std::path::Path;

use crate::form_utils::FormValue;
use crate::schema_parsing::FsEntry;
use crate::settings_utils::{Runner, SettingsUtils};
use crate::{form_utils::FormUtils, settings_utils::SettingsData};
use gtk::prelude::{
    ButtonExt, CheckButtonExt, DialogExt, EntryBufferExtManual, EntryExt, FileChooserExt, FileExt,
    GridExt, GtkWindowExt, WidgetExt,
};
use relm4::gtk::ResponseType;
use relm4::{gtk::traits::OrientableExt, *};

use crate::impl_validation;
use crate::traits::Validator;

pub struct SettingsModel {
    pub hidden: bool,
    node: gtk::EntryBuffer,
    schematic_runner: gtk::EntryBuffer,
    collection: gtk::EntryBuffer,
    package: gtk::EntryBuffer,
    show_private: bool,
    show_hidden: bool,
    runner: Runner,
    error: bool,
    success: bool,
    message: String,
}

impl SettingsModel {
    fn validate(&mut self) -> bool {
        let schematic = self.schematic_runner.text();
        let collection = self.collection.text();
        let package = self.package.text();
        let node = self.node.text();

        let values: Vec<FormValue<'_>> = vec![
            FormValue::new("node executable", node.as_str()),
            FormValue::new("schematics runner", schematic.as_str()),
            FormValue::new("schematics collection", collection.as_str()),
            FormValue::new("schematics package", package.as_str()),
        ];

        for field in values {
            let path = Path::new(field.value);
            if field.value.is_empty() {
                self.print_error(&format!("The '{}' field is mandatory!", field.name));
                return false;
            } else if !path.exists() || !path.is_file() {
                self.print_error(&format!(
                    "The '{}' doesn't exist or it's not a file!",
                    field.name
                ));
                return false;
            } else if (field.name == "node executable" || field.name == "schematics runner")
                && !path.is_executable()
            {
                self.print_error(&format!("The '{}' is not executable file!", field.name));
                return false;
            }
        }

        true
    }
}

impl Index<&'_ str> for SettingsModel {
    type Output = bool;
    fn index(&self, s: &str) -> &bool {
        match s {
            "show_private" => &self.show_private,
            "show_hidden" => &self.show_hidden,
            _ => panic!("unknown field: {}", s),
        }
    }
}

impl IndexMut<&'_ str> for SettingsModel {
    fn index_mut(&mut self, s: &str) -> &mut bool {
        match s {
            "show_private" => &mut self.show_private,
            "show_hidden" => &mut self.show_hidden,
            _ => panic!("unknown field: {}", s),
        }
    }
}

impl_validation!(SettingsModel);

#[derive(Debug)]
pub enum SettingsInput {
    Show,
    Apply,
    Cancel,
    NodeSelect(String),
    CollectionSelect(String),
    PackageSelect(String),
    RunnerSelect(String),
    ToggleCheckbox(bool, String),
}

#[derive(Debug)]
pub enum SettingsOutput {
    Close,
    SettingsLoaded(SettingsData),
}

#[relm4::component(pub)]
impl SimpleComponent for SettingsModel {
    type Init = bool;
    type Input = SettingsInput;
    type Output = SettingsOutput;

    view! {
        gtk::Dialog {
            set_title: Some("Settings"),
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
              set_margin_top: 10,
              set_css_classes: &["settings_container"],
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
                  set_label: &(model.message).to_string()
                },
              },
              gtk::Grid {
                set_row_spacing: 5,
                set_column_spacing: 5,
                set_orientation: gtk::Orientation::Horizontal,


                attach[ 0, 0, 1, 1]:  &FormUtils::new().label("node executable", "nodelLabel", None, Some(vec! ["label_right"])),
                attach[1, 0, 1, 1]: node_location = &gtk::Entry {
                  set_widget_name: "nodeInput",
                  set_hexpand: true,
                  set_css_classes: &["inputText",  "text_input"],
                  set_buffer: &model.node,
                },
                attach[ 2, 0, 1, 1]: node_browse_button = &gtk::Button {
                  set_icon_name: "document-open",
                  set_tooltip: "Browse file",
                  set_css_classes: &["node_browse_button", "button", "action_icon"],
                  connect_clicked[sender, root] => move |_| {
                    let dialog = FormUtils::new().file_chooser("Node executable",&root,None,None);
                    let send = sender.clone();
                    dialog.connect_response(move |file_chooser, resp| {
                        match resp {
                          ResponseType::Cancel => file_chooser.close(),
                          ResponseType::Accept => {
                            let file_name = file_chooser.file().unwrap().parse_name().to_string();
                            send.input(SettingsInput::NodeSelect(file_name));
                            file_chooser.close();
                          },
                          _ => ()
                        }
                    });
                    dialog.show();

                  }
                },

                attach[ 0, 1, 1, 1]:  &FormUtils::new().label("schematics collection", "schematicsCollLabel", None, Some(vec! ["label_right"])),
                attach[1, 1, 1, 1]: schematics_location = &gtk::Entry {
                  set_widget_name: "schematicsColInput",
                  set_hexpand: true,
                  set_css_classes: &["inputText",  "text_input"],
                  set_buffer: &model.collection,
                },
                attach[ 2, 1, 1, 1]: schematics_browse_button = &gtk::Button {
                  set_icon_name: "document-open",
                  set_tooltip: "Browse file",
                  set_css_classes: &["schematics_browse_button", "button", "action_icon"],
                  connect_clicked[sender, root] => move |_| {
                    let dialog = FormUtils::new().file_chooser("Schematics collection",&root,None,Some(FsEntry {
                      mask: String::from("collection.json"),
                      ..Default::default()
                    }));
                    let send = sender.clone();
                    dialog.connect_response(move |file_chooser, resp| {
                        match resp {
                          ResponseType::Cancel => file_chooser.close(),
                          ResponseType::Accept => {
                            let file_name = file_chooser.file().unwrap().parse_name().to_string();
                            send.input(SettingsInput::CollectionSelect(file_name));
                            file_chooser.close();
                          },
                          _ => ()
                        }
                    });
                    dialog.show();

                  }
                },

                attach[ 0, 2, 1, 1]:  &FormUtils::new().label("Schematics package", "schematicsPkgLabel", None, Some(vec! ["label_right"])),
                attach[1, 2, 1, 1]: package_location = &gtk::Entry {
                  set_widget_name: "schematicsPkgInput",
                  set_css_classes: &["inputText",  "text_input"],
                  set_buffer: &model.package,
                },
                attach[ 2, 2, 1, 1]: package_browse_button = &gtk::Button {
                  set_icon_name: "document-open",
                  set_tooltip: "Browse file",
                  set_css_classes: &["schematics_browse_button", "button",  "action_icon"],
                  connect_clicked[sender, root] => move |_| {
                    let dialog = FormUtils::new().file_chooser("Schematics package",&root,None,Some(FsEntry {
                      mask: String::from("package.json"),
                      ..Default::default()
                    }));
                    let send = sender.clone();
                    dialog.connect_response(move |file_chooser, resp| {
                        match resp {
                          ResponseType::Cancel => file_chooser.close(),
                          ResponseType::Accept => {
                            let file_name = file_chooser.file().unwrap().parse_name().to_string();
                            send.input(SettingsInput::PackageSelect(file_name));
                            file_chooser.close();
                          },
                          _ => ()
                        }
                    });
                    dialog.show();

                  }
                },

                attach[ 0, 3, 1, 1]:  &FormUtils::new().label("schematics runner", "schematicsRunnerLabel", None, Some(vec! ["label_right"])),
                attach[1, 3, 1, 1]: schematics_runner = &gtk::Entry {
                  set_widget_name: "schematicsRunnerInput",
                  set_css_classes: &["inputText", "text_input"],
                  set_buffer: &model.schematic_runner,
                },
                 attach[ 2, 3, 1, 1]: runner_browse_button = &gtk::Button {
                  set_icon_name: "document-open",
                  set_tooltip: "Browse file",
                  set_css_classes: &["runner_browse_button", "button", "action_icon"],
                  connect_clicked[sender, root] => move |_| {
                    let dialog = FormUtils::new().file_chooser("Schematics runner",&root,None,None);
                    let send = sender.clone();
                    dialog.connect_response(move |file_chooser, resp| {
                        match resp {
                          ResponseType::Cancel => file_chooser.close(),
                          ResponseType::Accept => {
                            let file_name = file_chooser.file().unwrap().parse_name().to_string();
                            send.input(SettingsInput::RunnerSelect(file_name));
                            file_chooser.close();
                          },
                          _ => ()
                        }
                    });
                    dialog.show();

                  }
                },
                attach[ 0, 4, 3, 1]: show_private = &gtk::CheckButton {
                  set_label: Some("Show private"),
                  set_css_classes: &["show_private_checkbox", "checkbox"],
                  #[watch]
                  set_active: model.show_private,
                  connect_toggled[sender] => move |button| {
                    sender.input(SettingsInput::ToggleCheckbox(button.is_active(), "show_private".to_string()));
                  }
                },
                attach[ 0, 5, 3, 1]: show_hidden = &gtk::CheckButton {
                  set_label: Some("Show hidden"),
                  set_css_classes: &["show_hidden_checkbox", "checkbox"],
                  #[watch]
                  set_active: model.show_hidden,
                  connect_toggled[sender] => move |button| {
                    sender.input(SettingsInput::ToggleCheckbox(button.is_active(), "show_hidden".to_string()));
                  }
                },

              }
            },
            connect_response[sender] => move |_, resp| {
                sender.input(if resp == gtk::ResponseType::Apply {
                    SettingsInput::Apply
                } else {
                    SettingsInput::Cancel
                })
            },
            connect_close_request[sender] => move |_| {
                sender.input(SettingsInput::Cancel);
                gtk::Inhibit(true)
            }
        }
    }

    fn init(
        params: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SettingsModel {
            hidden: params,
            node: gtk::EntryBuffer::default(),
            schematic_runner: gtk::EntryBuffer::default(),
            collection: gtk::EntryBuffer::default(),
            package: gtk::EntryBuffer::default(),
            show_private: false,
            show_hidden: false,
            runner: Runner::Google,
            error: false,
            success: false,
            message: String::default(),
        };

        SettingsUtils::new().init();

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            SettingsInput::Show => {
                let utils = SettingsUtils::new();
                if utils.exists() {
                    let data = utils.read();
                    self.node.set_text(data.node_binary);
                    self.package.set_text(data.schematics_package);
                    self.collection.set_text(data.schematics_collection);
                    self.schematic_runner.set_text(data.runner_location);
                    self.show_private = data.show_private;
                    self.runner = data.runner;
                }
                self.hidden = false;
            }
            SettingsInput::NodeSelect(file) => {
                self.node.set_text(file);
            }
            SettingsInput::CollectionSelect(file) => {
                self.collection.set_text(file);
            }
            SettingsInput::RunnerSelect(file) => {
                self.schematic_runner.set_text(file);
            }
            SettingsInput::PackageSelect(file) => {
                self.package.set_text(file);
            }
            SettingsInput::ToggleCheckbox(checked, field) => {
                self[&field] = checked;
            }
            SettingsInput::Apply => {
                if !self.validate() {
                    return;
                }

                self.clear_error();

                let runner_location = self.schematic_runner.text().to_string();
                let runner = if runner_location.ends_with("/schematics") {
                    Runner::Google
                } else if runner_location.ends_with("/fnd") {
                    Runner::MBH
                } else {
                    Runner::Custom
                };
                let settings = SettingsUtils::new();
                let data = SettingsData {
                    node_binary: self.node.text().to_string(),
                    runner_location: self.schematic_runner.text().to_string(),
                    schematics_collection: self.collection.text().to_string(),
                    schematics_package: self.package.text().to_string(),
                    show_private: self.show_private,
                    show_hidden: self.show_hidden,
                    runner,
                };
                settings.write(&data);
                sender
                    .output(SettingsOutput::SettingsLoaded(data.clone()))
                    .unwrap();
                self.hidden = true;
            }

            SettingsInput::Cancel => {
                let utils = SettingsUtils::new();
                self.clear_error();
                if utils.exists() {
                    self.hidden = true
                }
            }
        }
    }
}
