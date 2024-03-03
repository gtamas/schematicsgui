use relm4::gtk::prelude::{BoxExt, ButtonExt, Cast, FrameExt, OrientableExt, WidgetExt};
use relm4::gtk::{Align, ApplicationWindow, DialogFlags, Inhibit, MessageDialog};
use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller};
use std::fs;
use std::path::PathBuf;

use crate::command_builder::{CommandBuilder, Param};
use crate::config_editor_dialog::{ConfigEditorDialogInput, ConfigEditorDialogModel};
use crate::default_widget_builder::DefaultWidgetBuilder;
use crate::form_utils::FormUtils;
use crate::impl_validation;
use crate::profile_browser::{
    ProfileBrowserInput, ProfileBrowserInputParams, ProfileBrowserModel, ProfileBrowserOutput,
};
use crate::save_dialog::{
    ProfileData, SaveDialogInput, SaveDialogInputParams, SaveDialogModel, SaveDialogOutput,
};
use crate::schema_parsing::Primitive;
use crate::schema_parsing::{Schema, SchemaProp, StringOrPrompt};
use crate::schematics::Collection;
use crate::settings_utils::SettingsUtils;
use crate::traits::Validator;
use crate::traits::WidgetUtils;
use crate::value_extractor::ValueExtractor;
use crate::value_loader::ValueLoader;
use crate::xwidget_builder::XWidgetBuilder;
use std::borrow::Borrow;

#[tracker::track]
pub struct SchematicUiModel {
    hidden: bool,
    loader: bool,
    json: serde_json::Value,
    schematic: String,
    #[no_eq]
    configurable: Option<ConfigurableSchematicOptions>,
    package_name: String,
    cwd: Option<String>,
    file: Option<String>,
    error: bool,
    success: bool,
    message: String,
    has_directives: bool,
    #[no_eq]
    profiles: Vec<ProfileData>,
    #[no_eq]
    save: Controller<SaveDialogModel>,
    #[no_eq]
    config: Controller<ConfigEditorDialogModel>,
    #[no_eq]
    browser: Controller<ProfileBrowserModel>,
}

impl WidgetUtils for SchematicUiModel {}
impl_validation!(SchematicUiModel);

impl SchematicUiModel {
    fn reset_view(&mut self) {
        self.set_configurable(None);
        self.set_cwd(None);
        self.set_file(None);
        self.set_success(false);
        self.hidden = false;
        self.loader = false;
        self.browser.state().get_mut().model.clear();
    }

    fn has_profiles(&self) -> bool {
        let path = SettingsUtils::get_config_dir()
            .join(self.get_package_name())
            .join(self.get_schematic());
        path.is_dir() && path.exists() && fs::read_dir(&path).unwrap().count() > 0
    }

    fn get_config_path(&self) -> PathBuf {
        PathBuf::new()
            .join(self.cwd.as_ref().unwrap_or(&String::default()))
            .join(
                self.configurable
                    .as_ref()
                    .unwrap_or(&ConfigurableSchematicOptions::default())
                    .config_file
                    .clone(),
            )
    }

    fn config_exists(&self) -> bool {
        let path = self.get_config_path();
        path.exists() && path.is_file()
    }

    fn can_submit(&self) -> bool {
        !self.has_directives || self.cwd.is_some()
    }

    fn get_loaded_profile_file(&self) -> String {
        match self.browser.model().get_loaded_profile_file_as_option() {
            None => String::from("Profile: none"),
            Some(p) => format!("Profile: {}", p),
        }
    }

    fn load_values(&self, widgets: &mut SchematicUiModelWidgets, data_id: usize) {
        let mut w = widgets
            .frame
            .child()
            .unwrap()
            .downcast::<gtk::Box>()
            .unwrap()
            .first_child();

        loop {
            let widget = w.as_ref().unwrap();

            if self.is_a::<_, gtk::Label>(widget) {
                w = w.as_ref().unwrap().next_sibling();
                continue;
            }
            let browser_model = self.browser.model();
            let profile: &ProfileData = browser_model.profiles[data_id].borrow();
            let widget_name = widget.widget_name().to_string();
            let value = profile.data.get(&widget_name);

            if value.is_some() {
                let loader = ValueLoader::new(widget);
                loader.set_value(value.unwrap(), &widget_name);
            }

            w = w.as_ref().unwrap().next_sibling();

            if w.is_none() {
                break;
            }
        }
    }

    fn extract_values(&self, widgets: &mut SchematicUiModelWidgets) -> CommandBuilder {
        let mut w = widgets
            .frame
            .child()
            .unwrap()
            .downcast::<gtk::Box>()
            .unwrap()
            .first_child();
        let mut command = CommandBuilder::new(None);

        loop {
            let widget = w.as_ref().unwrap();

            let extractor = ValueExtractor::new(widget);
            let param = extractor.get_name_value();

            if param.is_some() {
                let p = param.unwrap();
                let c = self.configurable.clone().unwrap_or_default();

                if p.name == c.config_option && p.value == "true" {
                    command.set_configurable(p.name.clone());
                }

                if !p.name.is_empty() {
                    command.add(p);
                }
            }

            w = w.as_ref().unwrap().next_sibling();

            if w.is_none() {
                break;
            }
        }
        command
    }

    fn get_label_text(&self, prop: &SchemaProp) -> String {
        if prop.x_prompt.is_some() {
            let text = match prop.x_prompt.as_ref().unwrap() {
                StringOrPrompt::Str(s) => String::from(s),
                StringOrPrompt::Prompt(p) => String::from(&p.message),
            };

            return text;
        }

        return String::from(prop.description.as_ref().unwrap_or(&String::default()));
    }

    fn build_form(
        &self,
        parent: &gtk::Frame,
        json: &serde_json::Value,
        cwd: Option<String>,
    ) -> Option<gtk::Box> {
        let utils = FormUtils::new();
        let form = gtk::Box::new(relm4::gtk::Orientation::Vertical, 5);
        form.set_css_classes(&["ui"]);
        form.set_hexpand(true);

        match json["$id"].as_str() {
            Some(_) => {
                let empty = serde_json::Map::new();
                let props = json["properties"].as_object().unwrap_or(&empty);
                let keys = props.keys();

                for key in keys {
                    let prop_value: serde_json::Value = props.get(key).unwrap().clone();
                    match serde_json::from_value::<SchemaProp>(prop_value) {
                        Ok(prop) => {
                            let label_text = self.get_label_text(&prop);
                            form.append(&utils.label(&label_text, key, None, None));
                            if prop.x_widget.is_some() {
                                let builder = XWidgetBuilder::new(&prop, key.clone(), cwd.clone());

                                form.append(&builder.get_widget());
                            } else {
                                let builder =
                                    DefaultWidgetBuilder::new(&prop, key.clone(), cwd.clone());

                                form.append(&builder.get_widget());
                            }

                            prop
                        }
                        Err(e) => {
                            let window: ApplicationWindow = parent
                                .root()
                                .unwrap()
                                .downcast::<ApplicationWindow>()
                                .unwrap();
                            let dialog = MessageDialog::new(
                                Some(&window),
                                DialogFlags::all(),
                                gtk::MessageType::Error,
                                gtk::ButtonsType::YesNo,
                                "Oops.. an error has occured!",
                            );
                            dialog.set_secondary_text(Some(&format!(
                                "{}\n{}",
                                e, "Do you wish to try again?"
                            )));
                            dialog.show();
                            return None;
                        }
                    };
                }
                Some(form)
            }
            None => Some(form),
        }
    }
}

#[derive(Debug)]
pub struct SchematicUiInputParams {
    pub schema_path: PathBuf,
    pub schematic: String,
    pub package_name: String,
}

#[derive(Default, Debug, Clone)]
pub struct ConfigurableSchematicOptions {
    pub config_option: String,
    pub config_file: String,
}

impl ConfigurableSchematicOptions {
    pub fn new(config_option: String, config_file: String) -> Self {
        ConfigurableSchematicOptions {
            config_option,
            config_file,
        }
    }
}

#[derive(Debug)]
pub enum SchematicUiInput {
    Show(SchematicUiInputParams),
    Submit,
    ShowSave(bool),
    ShowConfig,
    ShowBrowser,
    HideBrowser,
    Selected(usize, String),
    Saved(String),
    ConfigDone,
    CwdChanged(String),
}

#[derive(Debug)]
pub enum SchematicUiOutput {
    Params(Vec<Param>, bool),
    ShowExecutor,
}

#[relm4::component(pub)]
impl Component for SchematicUiModel {
    type Input = SchematicUiInput;
    type Output = SchematicUiOutput;
    type Init = bool;
    type CommandOutput = bool;

    view! {
        #[root]
        gtk::Box {
          set_hexpand: true,
          set_orientation: relm4::gtk::Orientation::Vertical,
           set_css_classes: &["content_area"],
          gtk::Label {
            #[watch]
            set_visible: model.hidden,
            set_hexpand: true,
            set_vexpand: true,
            set_halign: gtk::Align::Center,
            set_label: "Please, select a schematic!"
          },
          gtk::Label {
            #[watch]
            set_label: &model.get_loaded_profile_file(),
            set_halign: Align::Center,
            set_valign: Align::Start,
            set_xalign: 0.5,
            #[watch]
            set_visible: !model.hidden,
            set_css_classes: &["profile_bar"]
          },
           gtk::Revealer {
            set_transition_type: gtk::RevealerTransitionType::SlideDown,
            #[watch]
            set_reveal_child: model.success,
            gtk::Label {
              set_hexpand: true,
              set_vexpand: false,
              set_css_classes: &["label", "success"],
              set_halign: gtk::Align::Center,
              #[watch]
              set_label: &(model.message).to_string()
            },
          },
          gtk::LinkButton {
            #[watch]
            set_visible: model.has_directives && !model.can_submit(),
            set_halign: Align::Center,
            set_valign: Align::Start,
            set_label: "Please set the current working directory!",
            connect_activate_link[sender] => move |_| -> Inhibit {
              let _ = sender.output_sender().send(SchematicUiOutput::ShowExecutor);
              Inhibit(true)
            },
          },
          gtk::ScrolledWindow {
          #[watch]
          set_visible: !model.hidden,
          set_hscrollbar_policy: gtk::PolicyType::Never,
            gtk::Box {
                set_orientation: relm4::gtk::Orientation::Vertical,
                gtk::Box {
                set_visible: true,
                set_hexpand: true,
                set_orientation: relm4::gtk::Orientation::Horizontal,
                append: frame = &gtk::Frame {
                  set_hexpand: true,
                  set_css_classes: &["ui_container"],
                  #[track = "model.changed(SchematicUiModel::json())"]
                  set_child: Some(&model.build_form(frame.as_ref(), &model.json, model.cwd.clone()).unwrap())
                },
                gtk::Revealer {
                    set_transition_type: gtk::RevealerTransitionType::SlideLeft,
                    #[watch]
                    set_reveal_child: model.loader,
                    set_child: Some(model.browser.widget())
                  },
              },
              gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_halign: Align::End,
                set_valign: Align::Start,
                append: cfg = &gtk::Button {
                  set_label: "Config editor",
                  #[watch]
                  set_visible: model.config_exists(),
                  connect_clicked[sender] => move |_| {
                    sender.input(SchematicUiInput::ShowConfig);
                  },
                  set_css_classes: &["action"]
                },
                append: submit = &gtk::Button {
                  set_label: "Submit",
                  #[watch]
                  set_sensitive: model.can_submit(),
                  connect_clicked[sender] => move |_| {
                    sender.input(SchematicUiInput::Submit);
                  },
                  set_css_classes: &["action"]
                },
                append: save = &gtk::Button {
                  set_label: "Save",
                  set_tooltip_text: Some("Save current settings"),
                  connect_clicked[sender] => move |_| {
                    sender.input(SchematicUiInput::ShowSave(false));
                  },
                  set_css_classes: &["action"]
                },
                append: save_as = &gtk::Button {
                  set_label: "Save as..",
                  #[watch]
                  set_visible: model.browser.model().is_profile_loaded(),
                  set_tooltip_text: Some("Save settings as.."),
                  connect_clicked[sender] => move |_| {
                    sender.input(SchematicUiInput::ShowSave(true));
                  },
                  set_css_classes: &["action"]
                },
                append: load = &gtk::Button {
                  set_label: "Load",
                  #[watch]
                  set_visible: !model.loader && model.has_profiles(),
                  set_tooltip_text: Some("Load settings"),
                  connect_clicked[sender] => move |_| {
                    sender.input(SchematicUiInput::ShowBrowser);
                  },
                  set_css_classes: &["action"]
                },
                append: hide = &gtk::Button {
                  set_label: "Hide",
                  #[watch]
                  set_visible: model.loader,
                  set_tooltip_text: Some("`Hide browser"),
                  connect_clicked[sender] => move |_| {
                    sender.input(SchematicUiInput::HideBrowser);
                  },
                  set_css_classes: &["action"]
                }
              }
            }


          }

        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let save = SaveDialogModel::builder()
            .transient_for(root)
            .launch(true)
            .forward(sender.input_sender(), |msg| match msg {
                SaveDialogOutput::Apply(file) => SchematicUiInput::Saved(file),
            });

        let config = ConfigEditorDialogModel::builder()
            .transient_for(root)
            .launch(true)
            .forward(sender.input_sender(), |_| SchematicUiInput::Submit);

        let browser =
            ProfileBrowserModel::builder()
                .launch(true)
                .forward(sender.input_sender(), |msg| match msg {
                    ProfileBrowserOutput::Loaded(selected_index, file) => {
                        SchematicUiInput::Selected(selected_index, file)
                    }
                });

        let model = SchematicUiModel {
            hidden: true,
            json: serde_json::Value::default(),
            tracker: 0,
            file: None,
            profiles: vec![],
            configurable: None,
            loader: false,
            cwd: None,
            schematic: String::default(),
            package_name: String::default(),
            error: false,
            success: false,
            message: String::default(),
            save,
            config,
            browser,
            has_directives: false,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.reset();

        match message {
            SchematicUiInput::Show(params) => {
                let json: serde_json::Value = serde_json::from_str(&Collection::read_str(
                    params.schema_path.to_str().unwrap(),
                ))
                .unwrap();
                let schema = serde_json::from_value::<Schema>(json.clone()).unwrap();

                self.has_directives = schema.has_directives();

                self.reset_view();

                if schema.configurable.is_some() {
                    let config_option = schema.configurable.clone().unwrap_or_default();
                    let path_prop = schema.get_property("path").unwrap_or_default();
                    let configurable = ConfigurableSchematicOptions::new(
                        config_option,
                        path_prop
                            .default
                            .unwrap_or(Primitive::Str(String::from("")))
                            .into(),
                    );
                    self.set_configurable(Some(configurable));
                }

                self.set_json(json);
                self.set_schematic(params.schematic);
                self.set_package_name(params.package_name);
            }
            SchematicUiInput::ShowSave(save_as) => {
                let command = self.extract_values(widgets);
                let mut description: Option<String> = None;
                let browser_model = &self.browser.state().get().model;
                let path = browser_model.get_loaded_profile_path();
                if browser_model.is_profile_loaded() {
                    description = Some(
                        browser_model.get_loaded_profile_data(&path)["meta"]["description"]
                            .as_str()
                            .unwrap()
                            .to_string(),
                    );
                }
                self.save
                    .sender()
                    .send(SaveDialogInput::Show(SaveDialogInputParams {
                        form_data: command.to_toml(),
                        schematic: self.schematic.clone(),
                        package_name: self.package_name.clone(),
                        file: browser_model.get_loaded_profile_file_as_option(),
                        description,
                        auto_save: browser_model.is_profile_loaded() && !save_as,
                    }))
                    .unwrap();
            }
            SchematicUiInput::ShowBrowser => {
                let _ = self.browser.sender().send(ProfileBrowserInput::Show(
                    ProfileBrowserInputParams::new(
                        self.schematic.clone(),
                        self.package_name.clone(),
                        false,
                        None,
                    ),
                ));
                self.loader = true;
            }
            SchematicUiInput::HideBrowser => {
                self.loader = false;
            }
            SchematicUiInput::ConfigDone => {}
            SchematicUiInput::ShowConfig => {
                self.config
                    .sender()
                    .send(ConfigEditorDialogInput::Show(
                        self.get_config_path().to_str().unwrap().to_string(),
                    ))
                    .unwrap();
            }
            SchematicUiInput::CwdChanged(path) => {
                let json = self.get_mut_json();
                *json.get_mut("$id").unwrap() = serde_json::Value::String(path.clone());
                self.set_cwd(path.into());
            }
            SchematicUiInput::Submit => {
                let command = self.extract_values(widgets);

                sender.output_sender().emit(SchematicUiOutput::Params(
                    command.to_params(),
                    self.configurable.is_some(),
                ));
            }
            SchematicUiInput::Saved(file) => {
                let _ = self.browser.sender().send(ProfileBrowserInput::Show(
                    ProfileBrowserInputParams::new(
                        self.schematic.clone(),
                        self.package_name.clone(),
                        false,
                        Some(file.clone()),
                    ),
                ));
                self.set_file(Some(file));
                self.print_success("Saved");
            }
            SchematicUiInput::Selected(selected, file) => {
                self.load_values(widgets, selected);
                self.set_file(Some(file));
            }
        }

        self.update_view(widgets, sender)
    }
}
