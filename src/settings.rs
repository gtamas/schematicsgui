use std::ops::{Index, IndexMut};

use crate::settings_utils::SettingsUtils;
use crate::{form_utils::FormUtils, settings_utils::SettingsData};
use gtk::prelude::{
    ButtonExt, CheckButtonExt, DialogExt, EntryBufferExtManual, EntryExt, FileChooserExt, FileExt,
    GridExt, GtkWindowExt, WidgetExt,
};
use relm4::gtk::ResponseType;
use relm4::{gtk::traits::OrientableExt, *};

pub struct SettingsModel {
    pub hidden: bool,
    node: gtk::EntryBuffer,
    schematic_runner: gtk::EntryBuffer,
    collection: gtk::EntryBuffer,
    package: gtk::EntryBuffer,
    show_private: bool,
    show_hidden: bool,
    google_runner: bool,
    mbh_runner: bool,
    custom_runner: bool,
    group: gtk::CheckButton,
}

impl Index<&'_ str> for SettingsModel {
    type Output = bool;
    fn index(&self, s: &str) -> &bool {
        match s {
            "show_private" => &self.show_private,
            "show_hidden" => &self.show_hidden,
            "google_runner" => &self.google_runner,
            "mbh_runner" => &self.mbh_runner,
            "custom_runner" => &self.custom_runner,
            _ => panic!("unknown field: {}", s),
        }
    }
}

impl IndexMut<&'_ str> for SettingsModel {
    fn index_mut(&mut self, s: &str) -> &mut bool {
        match s {
            "show_private" => &mut self.show_private,
            "show_hidden" => &mut self.show_hidden,
            "google_runner" => &mut self.google_runner,
            "mbh_runner" => &mut self.mbh_runner,
            "custom_runner" => &mut self.custom_runner,
            _ => panic!("unknown field: {}", s),
        }
    }
}

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
            set_default_width: 400,
            set_modal: true,
            set_destroy_with_parent: true,
            #[watch]
            set_visible: !model.hidden,
            add_button: ("Save", gtk::ResponseType::Apply),
            add_button: ("Cancel", gtk::ResponseType::Cancel),
            gtk::Box {
              set_orientation: gtk::Orientation::Horizontal,
              set_margin_top: 10,
              gtk::Grid {
                set_row_spacing: 10,
                set_column_spacing: 10,
                set_orientation: gtk::Orientation::Horizontal,
                attach[ 0, 0, 1, 1]: &FormUtils::new().label("node", "nodeLabel", None),
                attach[ 1, 0, 1, 1]: node_location = &gtk::Entry {
                  set_widget_name: "nodeInput",
                  set_css_classes: &["inputText"],
                  set_buffer: &model.node,
                },
                attach[ 2, 0, 1, 1]: browse_node_button = &gtk::Button {
                  set_label: "browse",
                  set_css_classes: &["node_browse_button", "button"],
                  connect_clicked[sender, root] => move |_| {
                    let dialog = FormUtils::new().file_chooser("Node binary",&root,true,Some("node"));
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

                attach[ 0, 1, 1, 1]:  &FormUtils::new().label("schematics collection", "schematicsCollLabel", None),
                attach[1, 1, 1, 1]: schematics_location = &gtk::Entry {
                  set_widget_name: "schematicsColInput",
                  set_css_classes: &["inputText"],
                  set_buffer: &model.collection,
                },
                attach[ 2, 1, 1, 1]: schematics_browse_button = &gtk::Button {
                  set_label: "browse",
                  set_css_classes: &["schematics_browse_button", "button"],
                  connect_clicked[sender, root] => move |_| {
                    let dialog = FormUtils::new().file_chooser("Schematics collection",&root,true,Some("collection.json"));
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

                attach[ 0, 2, 1, 1]:  &FormUtils::new().label("Schematics package", "schematicsPkgLabel", None),
                attach[1, 2, 1, 1]: package_location = &gtk::Entry {
                  set_widget_name: "schematicsPkgInput",
                  set_css_classes: &["inputText"],
                  set_buffer: &model.package,
                },
                attach[ 2, 2, 1, 1]: package_browse_button = &gtk::Button {
                  set_label: "browse",
                  set_css_classes: &["schematics_browse_button", "button"],
                  connect_clicked[sender, root] => move |_| {
                    let dialog = FormUtils::new().file_chooser("Schematics package",&root,true,Some("package.json"));
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

                attach[ 0, 3, 1, 1]:  &FormUtils::new().label("schematics runner", "schematicsRunnerLabel", None),
                attach[1, 3, 1, 1]: schematics_runner = &gtk::Entry {
                  set_widget_name: "schematicsRunnerInput",
                  set_css_classes: &["inputText"],
                  set_buffer: &model.schematic_runner,
                },
                 attach[ 2, 3, 1, 1]: runner_browse_button = &gtk::Button {
                  set_label: "browse",
                  set_css_classes: &["runner_browse_button", "button"],
                  connect_clicked[sender, root] => move |_| {
                    let dialog = FormUtils::new().file_chooser("Schematics runner",&root,true,None);
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
                attach[ 0, 6, 3, 1]: google_runner = &gtk::CheckButton {
                  set_group: Some(&model.group),
                  set_label: Some("The selected runner is Google runner"),
                  set_css_classes: &["google_runner_checkbox", "checkbox"],

                  set_active: model.google_runner,
                  connect_toggled[sender] => move |button| {
                     sender.input(SettingsInput::ToggleCheckbox(button.is_active(), "google_runner".to_string()));
                  }
                },
                attach[ 0, 7, 3, 1]: mbh_runner = &gtk::CheckButton {
                  set_group: Some(&model.group),
                  set_label: Some("The selected runner is MBH runner"),
                  set_css_classes: &["google_runner_checkbox", "checkbox"],

                  set_active: model.mbh_runner,
                  connect_toggled[sender] => move |button| {
                     sender.input(SettingsInput::ToggleCheckbox(button.is_active(), "mbh_runner".to_string()));
                  }
                },

                 attach[ 0, 8, 3, 1]: custom_runner = &gtk::CheckButton {
                  set_group: Some(&model.group),
                  set_label: Some("The selected runner is a custom runner"),
                  set_css_classes: &["custom_runner_checkbox", "checkbox"],
                  set_active: model.custom_runner,
                  connect_toggled[sender] => move |button| {
                     sender.input(SettingsInput::ToggleCheckbox(button.is_active(), "custom_runner".to_string()));
                  }
                }



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
            google_runner: false,
            mbh_runner: false,
            custom_runner: true,
            group: gtk::CheckButton::new(),
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
                    self.node.set_text(data.node_location);
                    self.collection.set_text(data.schematics_package);
                    self.schematic_runner.set_text(data.runner_location);
                    self.show_private = data.show_private;
                    self.google_runner = data.google_runner;
                    self.mbh_runner = data.mbh_runner;
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
                let settings = SettingsUtils::new();
                let data = SettingsData {
                    node_location: self.node.text().to_string(),
                    runner_location: self.schematic_runner.text().to_string(),
                    schematics_collection: self.collection.text().to_string(),
                    schematics_package: self.package.text().to_string(),
                    show_private: self.show_private,
                    show_hidden: self.show_hidden,
                    google_runner: self.google_runner,
                    mbh_runner: self.mbh_runner,
                    custom_runner: self.custom_runner,
                };
                settings.write(&data);
                sender
                    .output(SettingsOutput::SettingsLoaded(data.clone()))
                    .unwrap();
                self.hidden = true;
            }

            SettingsInput::Cancel => {
                let utils = SettingsUtils::new();
                if utils.exists() {
                    self.hidden = true
                }
            }
        }
    }
}
