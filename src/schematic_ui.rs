use relm4::gtk::prelude::{
    BoxExt, ButtonExt, Cast, EditableExt, EntryBufferExtManual, EntryExt, FrameExt, OrientableExt,
    WidgetExt,
};
use relm4::gtk::{Align, ApplicationWindow, DialogFlags, EntryBuffer, MessageDialog};
use relm4::typed_list_view::TypedListView;
use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller};
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use toml::Table;

use crate::command_builder::{CommandBuilder, Param};
use crate::default_widget_builder::DefaultWidgetBuilder;
use crate::form_utils::FormUtils;
use crate::profile_browser::{ProfileBrowserModel, ProfileBrowserOutput};
use crate::profile_data_list_item::{ProfileDataListItem, ProfileDataMenuItem};
use crate::save_dialog::{
    ProfileData, SaveDialogInput, SaveDialogInputParams, SaveDialogModel, SaveDialogOutput,
};
use crate::schema_parsing::SchemaProp;
use crate::schematics::Collection;
use crate::settings_utils::SettingsUtils;
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
    file: Option<String>,
    search: EntryBuffer,
    #[no_eq]
    profiles: Vec<ProfileData>,
    #[no_eq]
    save: Controller<SaveDialogModel>,
    #[no_eq]
    load: Controller<ProfileBrowserModel>,
    #[no_eq]
    list_view_wrapper: TypedListView<ProfileDataListItem, gtk::SingleSelection>,
}

impl WidgetUtils for SchematicUiModel {}

impl SchematicUiModel {
    fn get_profile_dir(&self) -> PathBuf {
        SettingsUtils::get_config_dir().join(&self.schematic)
    }

    fn load_profiles(&mut self) -> Vec<ProfileDataMenuItem> {
        let mut result: Vec<ProfileDataMenuItem> = vec![];
        let dir = fs::read_dir(&self.get_profile_dir());

        for entry in dir.unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_file() && path.extension().unwrap_or( &OsStr::new("")) == "toml" {
                let profile = match match fs::read_to_string(&path) {
                    Ok(contents) => contents,
                    Err(e) => panic!("Could not read file! {}", e),
                }
                .parse::<Table>()
                {
                    Ok(parsed) => parsed,
                    Err(err) => panic!("Could not parse TOML! {}", err),
                };

                let file = path.file_name().unwrap().to_str().unwrap();
                let mut profile_name = path.file_stem().unwrap().to_str().unwrap().to_string();
                let description = profile["meta"]["description"].as_str().unwrap_or("");

                if description.len() > 0 {
                    profile_name = format!("{} ({})", profile_name, description.clone().to_string())
                }

                let profile = ProfileData::new(
                    profile_name.clone(),
                    file.clone().to_string(),
                    profile["data"].as_table().unwrap().clone(),
                );

                self.profiles.push(profile);
                result.push(ProfileDataMenuItem::new(
                    file.clone().to_string(),
                    profile_name,
                ))
            }
        }
        return result;
    }

    fn get_loaded_profile(&self) -> String {
        let file = self.file.clone().unwrap_or(String::default());
        let path = SettingsUtils::get_config_dir()
            .join(&self.schematic)
            .join(&file);
        format!("Profile: {}", {
            if file.len() == 0 {
                "none"
            } else {
                path.to_str().unwrap()
            }
        })
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
            let profile: &ProfileData = self.profiles[data_id].borrow();
            let widget_name = widget.widget_name().to_string();
            let value = profile.data.get(&widget_name);

            println!("{} {:?} {:?}", widget_name, value, widget);

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
            let param: Param = extractor.get_name_value();

            if param.name.len() > 0 {
                command.add(param);
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
                form.append(&utils.label(
                    &json["title"].as_str().unwrap_or("").replace("schema", ""),
                    "schema",
                    None,
                    Some(vec!["label_title"]),
                ));
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
}

#[derive(Debug)]
pub enum SchematicUiInput {
    Show(SchematicUiInputParams),
    Submit,
    ShowSave,
    ShowBrowser,
    HideBrowser,
    Selected(u32),
    FilterChange,
    Saved(String),
    Loaded(String),
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
               set_label: &model.get_loaded_profile(),
               set_halign: Align::Center,
               set_valign: Align::Start,
               set_xalign: 0.5,
               set_css_classes: &["profile_bar"]
          },
          gtk::Box {
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
                gtk::Box {
                  set_orientation: gtk::Orientation::Vertical,
                  set_hexpand: false,
                  set_width_request: 300,
                  gtk::Entry {
                    set_buffer: &model.search,
                    set_css_classes: &["search", "inputText"],
                    set_vexpand: false,
                    set_hexpand: false,
                    set_width_request: 300,
                    set_margin_bottom: 10,
                    set_icon_from_icon_name[Some("system-search")]: gtk::EntryIconPosition::Primary,
                    connect_changed[sender] => move |_| {
                      sender.input(SchematicUiInput::FilterChange);
                    }
                  },
                  gtk::ScrolledWindow {
                    set_vexpand: true,
                    set_hexpand: true,
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_max_content_width: 300,
                    #[local_ref]
                    my_view -> gtk::ListView {
                      set_single_click_activate: true,
                      connect_activate[sender] => move |_, selected| {
                      sender.input(SchematicUiInput::Selected(selected));
                      }
                    }
                  }
                }
              },
          },
          gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_halign: Align::End,
            set_valign: Align::Start,
            append: submit = &gtk::Button {
              set_label: "Submit",
              #[watch]
              set_visible: !model.hidden,
              connect_clicked[sender] => move |_| {
              sender.input(SchematicUiInput::Submit);
              },
              set_css_classes: &["action"]
            },
            append: save = &gtk::Button {
              set_label: "Save",
              #[watch]
              set_visible: !model.hidden,
              set_tooltip_text: Some("Save current settings"),
              connect_clicked[sender] => move |_| {
                sender.input(SchematicUiInput::ShowSave);
              },
              set_css_classes: &["action"]
            },
            append: load = &gtk::Button {
              set_label: "Load",
              #[watch]
              set_visible: !model.loader,
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
        let list_view_wrapper: TypedListView<ProfileDataListItem, gtk::SingleSelection> =
            TypedListView::with_sorting();

        let save = SaveDialogModel::builder()
            .transient_for(root)
            .launch(true)
            .forward(sender.input_sender(), |msg| match msg {
                SaveDialogOutput::Apply(file) => SchematicUiInput::Saved(file),
            });

        let load = ProfileBrowserModel::builder()
            .transient_for(root)
            .launch(true)
            .forward(sender.input_sender(), |msg| match msg {
                ProfileBrowserOutput::Load(data) => SchematicUiInput::Loaded(data),
            });

        let model = SchematicUiModel {
            hidden: true,
            list_view_wrapper,
            json: serde_json::Value::default(),
            tracker: 0,
            file: None,
            profiles: vec![],
            loader: false,
            schematic: String::default(),
            search: EntryBuffer::default(),
            save,
            load,
        };

        let my_view = &model.list_view_wrapper.view;

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
                self.set_file(None);
                self.hidden = false
            }
            SchematicUiInput::ShowSave => {
                let command = self.extract_values(widgets);
                self.save
                    .sender()
                    .send(SaveDialogInput::Show(SaveDialogInputParams {
                        form_data: command.to_toml(),
                        schematic: self.schematic.clone(),
                        file: self.file.clone(),
                    }))
                    .unwrap();
            }
            SchematicUiInput::ShowBrowser => {
                let mut profiles = self.load_profiles();
                profiles.sort();

                self.list_view_wrapper.clear();
                self.list_view_wrapper.clear_filters();

                for profile in profiles {
                    self.list_view_wrapper
                        .append(ProfileDataListItem::new(profile));
                }
                self.loader = true;
                // self.load
                //     .sender()
                //     .send(ProfileBrowserInput::Show(config_dir))
                //     .unwrap();
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
                self.set_file(Some(file));
            }
            SchematicUiInput::Loaded(data) => {}
            SchematicUiInput::Selected(selected) => {
                let selected_item = self
                    .list_view_wrapper
                    .get(selected)
                    .unwrap()
                    .borrow()
                    .value
                    .clone();
                let profile_index = self
                    .profiles
                    .iter()
                    .position(|p| p.file == selected_item.file)
                    .unwrap();
                self.load_values(widgets, profile_index);
                // println!("{:?}, {:?}", selected_item, self.profiles[profile_index]);
            }
            SchematicUiInput::FilterChange => {
                let query_str = self.search.text().to_string();
                self.list_view_wrapper.pop_filter();
                self.list_view_wrapper
                    .add_filter(move |item| item.value.label.starts_with(&query_str));
                self.list_view_wrapper.set_filter_status(0, true);
            }
        }

        self.update_view(widgets, sender)
    }
}
