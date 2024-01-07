use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use relm4::gtk::prelude::{
    EditableExt, EntryBufferExtManual, EntryExt, OrientableExt, SelectionModelExt, WidgetExt,
};
use relm4::gtk::EntryBuffer;
use relm4::typed_list_view::TypedListView;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};
use toml::Table;

use crate::profile_data_list_item::{ProfileDataListItem, ProfileDataMenuItem};
use crate::save_dialog::ProfileData;
use crate::settings_utils::SettingsUtils;
use toml::map::Map;
use toml::Value;

pub struct ProfileBrowserModel {
    hidden: bool,
    schematic: String,
    package_name: String,
    file: Option<String>,
    search: EntryBuffer,
    pub profiles: Vec<ProfileData>,
    list_view_wrapper: TypedListView<ProfileDataListItem, gtk::SingleSelection>,
}

#[derive(Debug)]
pub enum ProfileBrowserInput {
    Show(ProfileBrowserInputParams),
    Hide,
    Selected(u32),
    Clear,
    Select(String),
    FilterChange,
}

#[derive(Debug, Clone)]
pub struct ProfileBrowserInputParams {
    pub schematic: String,
    pub package_name: String,
    pub clear: bool,
    pub file: Option<String>,
}

impl ProfileBrowserInputParams {
    pub fn new(schematic: String, package_name: String, clear: bool, file: Option<String>) -> Self {
        Self {
            schematic,
            package_name,
            clear,
            file,
        }
    }
}

#[derive(Debug)]
pub enum ProfileBrowserOutput {
    Loaded(usize, String),
}

pub struct ProfileBrowserInit {}

impl ProfileBrowserModel {
    pub fn get_profile_dir(&self) -> PathBuf {
        SettingsUtils::get_config_dir()
            .join(&self.package_name)
            .join(&self.schematic)
    }

    pub fn get_loaded_profile_path(&self) -> PathBuf {
        let file = self.file.clone().unwrap_or(String::default());
        self.get_profile_dir().join(&file)
    }

    pub fn get_loaded_profile_file_as_option(&self) -> Option<String> {
        let path = self.get_loaded_profile_path();
        if !path.is_file() {
            None
        } else {
            Some(path.to_str().unwrap().to_string())
        }
    }

    pub fn get_loaded_profile_data(&self, path: &PathBuf) -> Map<String, Value> {
        match match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(e) => panic!("Could not read file! {}", e),
        }
        .parse::<Table>()
        {
            Ok(parsed) => parsed,
            Err(err) => panic!("Could not parse TOML! {}", err),
        }
    }

    pub fn is_profile_loaded(&self) -> bool {
        self.get_loaded_profile_path().is_file()
    }

    pub fn clear(&mut self) {
        self.hidden = true;
        self.file = None;
        self.search.set_text("");
        self.profiles.clear();
        self.schematic = String::default();
        self.list_view_wrapper.clear();
        self.list_view_wrapper.clear_filters();
    }

    fn add_profile_data(&mut self) {
        let mut profiles = self.load_profiles();
        profiles.sort_by_cached_key(|i| i.label.clone());

        for profile in profiles {
            self.list_view_wrapper
                .insert_sorted(ProfileDataListItem::new(profile), |a, b| {
                    a.value.label.cmp(&b.value.label)
                });
        }
    }

    fn load_profiles(&mut self) -> Vec<ProfileDataMenuItem> {
        let mut result: Vec<ProfileDataMenuItem> = vec![];
        let dir = fs::read_dir(&self.get_profile_dir());

        for entry in dir.unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_file() && path.extension().unwrap_or(&OsStr::new("")) == "toml" {
                let profile = self.get_loaded_profile_data(&path);

                let file = path.file_name().unwrap().to_str().unwrap();
                let mut profile_name = path.file_stem().unwrap().to_str().unwrap().to_string();
                let description = profile["meta"]["description"].as_str().unwrap_or("");

                if description.len() > 0 {
                    profile_name = format!("{} ({})", profile_name, description.to_string())
                }

                let profile = ProfileData::new(
                    profile_name.clone(),
                    file.to_string(),
                    profile["data"].as_table().unwrap().clone(),
                );

                self.profiles.push(profile);
                result.push(ProfileDataMenuItem::new(file.to_string(), profile_name))
            }
        }
        self.profiles.sort_by_cached_key(|i| i.title.clone());
        return result;
    }
}

#[relm4::component(pub)]
impl SimpleComponent for ProfileBrowserModel {
    type Input = ProfileBrowserInput;
    type Output = ProfileBrowserOutput;
    type Init = bool;

    view! {
         gtk::Box {
          set_orientation: gtk::Orientation::Vertical,
          set_hexpand: false,
          set_width_request: 300,
          gtk::Entry {
            set_buffer: &model.search,
            set_css_classes: &["search", "text_input"],
            set_vexpand: false,
            set_hexpand: false,
            set_width_request: 300,
            set_margin_bottom: 10,
            set_icon_from_icon_name[Some("system-search")]: gtk::EntryIconPosition::Primary,
            connect_changed[sender] => move |_| {
              sender.input(ProfileBrowserInput::FilterChange);
            }
          },
          gtk::ScrolledWindow {
            set_vexpand: true,
            set_hexpand: true,
            set_hscrollbar_policy: gtk::PolicyType::Never,
            set_max_content_width: 300,
            set_css_classes: &["profile_browser"],
            #[local_ref]
            my_view -> gtk::ListView {
              set_single_click_activate: true,
              connect_activate[sender] => move |_, selected| {
              sender.input(ProfileBrowserInput::Selected(selected));
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
        let list_view_wrapper: TypedListView<ProfileDataListItem, gtk::SingleSelection> =
            TypedListView::with_sorting();

        let model = ProfileBrowserModel {
            hidden: true,
            list_view_wrapper,
            schematic: String::default(),
            package_name: String::default(),
            search: EntryBuffer::default(),
            profiles: vec![],
            file: None,
        };

        let my_view = &model.list_view_wrapper.view;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            ProfileBrowserInput::Show(params) => {
                if params.clear {
                    self.clear()
                } else {
                    self.profiles.clear();
                    self.list_view_wrapper.clear();
                    self.list_view_wrapper.clear_filters();
                }

                self.schematic = params.schematic;
                self.package_name = params.package_name;
                self.add_profile_data();

                if params.file.is_some() {
                    sender.input(ProfileBrowserInput::Select(params.file.unwrap()));
                }

                self.hidden = false;
            }
            ProfileBrowserInput::Select(profile) => {
                if self.profiles.is_empty() {
                    self.add_profile_data();
                }

                let file_name = PathBuf::from(profile.clone())
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                let selected = self
                    .profiles
                    .iter()
                    .position(|p| p.file == file_name)
                    .unwrap();
                sender.input(ProfileBrowserInput::Selected(selected as u32));
                self.list_view_wrapper
                    .selection_model
                    .select_item(selected as u32, true);
            }
            ProfileBrowserInput::Hide => {
                self.hidden = true;
            }
            ProfileBrowserInput::Clear => {
                self.clear();
            }
            ProfileBrowserInput::Selected(selected) => {
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
                self.file = Some(selected_item.file);
                let _ = sender.output(ProfileBrowserOutput::Loaded(
                    profile_index,
                    self.get_loaded_profile_path().to_str().unwrap().to_string(),
                ));
            }
            ProfileBrowserInput::FilterChange => {
                let query_str = self.search.text().to_string();
                self.list_view_wrapper.pop_filter();
                self.list_view_wrapper
                    .add_filter(move |item| item.value.label.starts_with(&query_str));
                self.list_view_wrapper.set_filter_status(0, true);
            }
        }
    }
}
