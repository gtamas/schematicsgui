use relm4::gtk::prelude::{BoxExt, ButtonExt, Cast, FrameExt, OrientableExt, WidgetExt};
use relm4::gtk::{Align, ApplicationWindow, DialogFlags, MessageDialog};
use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller};
use std::fs;
use std::path::PathBuf;

use crate::command_builder::{CommandBuilder, Param};
use crate::default_widget_builder::DefaultWidgetBuilder;
use crate::form_utils::FormUtils;
use crate::impl_validation;
use crate::profile_browser::{
    ProfileBrowserInput, ProfileBrowserInputParams, ProfileBrowserModel, ProfileBrowserOutput,
};
use crate::save_dialog::{
    ProfileData, SaveDialogInput, SaveDialogInputParams, SaveDialogModel, SaveDialogOutput,
};
use crate::schema_parsing::SchemaProp;
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
    package_name: String,
    file: Option<String>,
    error: bool,
    success: bool,
    message: String,
    #[no_eq]
    profiles: Vec<ProfileData>,
    #[no_eq]
    save: Controller<SaveDialogModel>,
    #[no_eq]
    browser: Controller<ProfileBrowserModel>,
}

impl WidgetUtils for SchematicUiModel {}
impl_validation!(SchematicUiModel);

impl SchematicUiModel {
    pub fn has_profiles(&self) -> bool {
        let path = SettingsUtils::get_config_dir().join(self.get_package_name()).join(self.get_schematic());
        path.is_dir() && path.exists() && fs::read_dir(&path).unwrap().count() > 0
    }

    fn get_loaded_profile_file(&self) -> String {
        match self.browser.model().get_loaded_profile_file_as_option() {
            None => String::from("Profile: none"),
            Some(p) => format!("Profile: {}", p),
        }
    }

    fn load_values(&self, widgets: &mut SchematicUiModelWidgets, data_id: usize) -> () {
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

            if (param.is_some()) {
                let p = param.unwrap();

                if p.name.len() > 0 {
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

    fn build_form(&self, parent: &gtk::Frame, json: &serde_json::Value) -> Option<gtk::Box> {
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
                            let label_text = prop.description.clone().unwrap_or(String::default());
                            form.append(&utils.label(&label_text, key, None, None));
                            if prop.x_widget.is_some() {
                                let builder = XWidgetBuilder::new(&prop, key.clone());

                                form.append(&builder.get_widget());
                            } else {
                                let builder = DefaultWidgetBuilder::new(&prop, key.clone());

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
                                format!("{}", "Oops.. an error has occured!"),
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

#[derive(Debug)]
pub enum SchematicUiInput {
    Show(SchematicUiInputParams),
    Submit,
    ShowSave(bool),
    ShowBrowser,
    HideBrowser,
    Selected(usize, String),
    // FilterChange,
    Saved(String),
}

#[derive(Debug)]
pub enum SchematicUiOutput {
    Params(Vec<Param>),
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
              set_label: &format!("{}", &model.message)
            },
          },
          gtk::Box {
            #[watch]
            set_visible: !model.hidden,
            set_hexpand: true,
            set_orientation: relm4::gtk::Orientation::Horizontal,
            append: frame = &gtk::Frame {
              set_hexpand: true,
              set_css_classes: &["ui_container"],
              #[track = "model.changed(SchematicUiModel::json())"]
              set_child: Some(&model.build_form(&frame, &model.json).unwrap())
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
            #[watch]
            set_visible: !model.hidden,
            append: submit = &gtk::Button {
              set_label: "Submit",
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

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let save = SaveDialogModel::builder()
            .transient_for(root)
            .launch(true)
            .forward(sender.input_sender(), |msg| match msg {
                SaveDialogOutput::Apply(file) => SchematicUiInput::Saved(file),
            });

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
            loader: false,
            schematic: String::default(),
            package_name: String::default(),
            error: false,
            success: false,
            message: String::default(),
            save,
            browser,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        self.reset();

        match message {
            SchematicUiInput::Show(params) => {
                let json = serde_json::from_str(&Collection::read_str(
                    params.schema_path.to_str().unwrap(),
                ))
                .unwrap();
                self.set_json(json);
                self.set_schematic(params.schematic);
                self.set_package_name(params.package_name);
                self.set_file(None);
                self.set_success(false);
                self.hidden = false;
                self.loader = false;
                self.browser.state().get_mut().model.clear();
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
            SchematicUiInput::Submit => {
                let command = self.extract_values(widgets);

                sender
                    .output_sender()
                    .emit(SchematicUiOutput::Params(command.to_params()));
            }
            SchematicUiInput::Saved(file) => {
                println!("{}", file);
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
