use relm4::gtk::prelude::{EntryBufferExtManual, GridExt, OrientableExt, WidgetExt};
use relm4::gtk::{Justification, LinkButton};
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

use crate::form_utils::FormUtils;
use crate::schematics::Collection;

use serde::{Deserialize, Serialize};

pub struct PackageInfoModel {
    hidden: bool,
    pkg_name: gtk::EntryBuffer,
    description: gtk::EntryBuffer,
    version: gtk::EntryBuffer,
    author: gtk::EntryBuffer,
    repo: gtk::EntryBuffer,
    homepage: gtk::EntryBuffer,
    bugs: gtk::EntryBuffer,
    utils: FormUtils,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PartialPackageJsonData {
    pub name: String,
    version: String,
    description: Option<String>,
    author: Option<AuthorType>,
    homepage: Option<String>,
    bugs: Option<BugsType>,
    repository: Option<RepoType>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepoObject {
    pub url: String,
    pub r#type: Option<String>,
    pub directory: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum RepoType {
    Obj(RepoObject),
    String(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BugsObject {
    pub url: String,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum BugsType {
    Obj(BugsObject),
    String(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthorObject {
    pub name: String,
    pub email: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum AuthorType {
    Obj(AuthorObject),
    String(String),
}

impl PackageInfoModel {
    fn read_package_json(&self, path: String) -> PartialPackageJsonData {
        let json_str = &Collection::read_str(&path);
        let package_json: PartialPackageJsonData = serde_json::from_str(&json_str).unwrap();
        package_json
    }

    fn populate_view(&self, package_json: PartialPackageJsonData) {
        let unknown = "Unknown".to_string();
        self.pkg_name.set_text(package_json.name);
        self.version.set_text(package_json.version);
        self.description
            .set_text(package_json.description.unwrap_or(unknown.clone()));
        self.author.set_text(match package_json.author {
            Some(AuthorType::String(s)) => s,
            Some(AuthorType::Obj(o)) => o.name,
            None => "Unknown".to_string(),
        });
        self.bugs.set_text(match package_json.bugs {
            Some(BugsType::String(s)) => s,
            Some(BugsType::Obj(o)) => o.url,
            None => "Unknown".to_string(),
        });
        self.repo.set_text(match package_json.repository {
            Some(RepoType::String(s)) => s,
            Some(RepoType::Obj(o)) => o.url,
            None => "Unknown".to_string(),
        });
        self.homepage
            .set_text(package_json.homepage.unwrap_or(unknown.clone()));
    }
}

#[derive(Debug)]
pub enum PackageInfoInput {
    Show(String),
}

#[derive(Debug)]
pub enum PackageInfoOutput {
    PackageData(PartialPackageJsonData),
}

#[relm4::component(pub)]
impl SimpleComponent for PackageInfoModel {
    type Input = PackageInfoInput;
    type Output = PackageInfoOutput;
    type Init = bool;

    view! {
        #[root]
        gtk::Box {
          set_orientation: gtk::Orientation::Horizontal,
          set_hexpand: true,
          set_css_classes: &["content_area"],
          gtk::Grid {
             set_row_spacing: 10,
             set_column_spacing: 10,
             set_hexpand: true,
             set_orientation: gtk::Orientation::Horizontal,
             attach[ 0, 0, 1, 1]: &model.utils.label("Package name:", "packageName", Some(Justification::Right), Some(vec! ["label_right"])),
             attach[ 1, 0, 1, 1] = &gtk::Text {
                set_halign: gtk::Align::Start,
                set_buffer: &model.pkg_name,
                set_width_request: 400,
             },
            attach[ 0, 1, 1, 1]: &model.utils.label("Description:", "desc", Some(Justification::Right), Some(vec! ["label_right"])),
            attach[ 1, 1, 1, 1] = &gtk::Text {
                set_halign: gtk::Align::Start,
                set_buffer: &model.description,
                set_width_request: 400,
             },
            attach[ 0, 2, 1, 1]: &model.utils.label("Version:", "version", Some(Justification::Right), Some(vec! ["label_right"])),
            attach[ 1, 2, 1, 1] = &gtk::Text {
                set_halign: gtk::Align::Start,
                set_buffer: &model.version,
                set_width_request: 400,
             },
            attach[ 0, 3, 1, 1]: &model.utils.label("Author:", "author", Some(Justification::Right), Some(vec! ["label_right"])),
            attach[ 1, 3, 1, 1] = &gtk::Text {
                set_halign: gtk::Align::Start,
                set_buffer: &model.author,
                set_width_request: 400,
             },
            attach[ 0, 4, 1, 1]: &model.utils.label("Repository:", "repo", Some(Justification::Right), Some(vec! ["label_right"])),
            attach[ 1, 4, 1, 1] = &model.utils.get_link_button("", Some("Repository")) -> LinkButton {
                #[watch]
                set_uri: &model.repo.text()
            },
            attach[ 0, 5, 1, 1]: &model.utils.label("Homepage:", "homepage", Some(Justification::Right), Some(vec! ["label_right"])),
            attach[ 1, 5, 1, 1] = &model.utils.get_link_button("", Some("Homepage")) -> LinkButton {
                #[watch]
                set_uri: &model.homepage.text()
            },
            attach[ 0, 6, 1, 1]: &model.utils.label("Bugs:", "bugs", Some(Justification::Right), Some(vec! ["label_right"])),
            attach[ 1, 6, 1, 1] = &model.utils.get_link_button("", Some("Bugs")) -> LinkButton {
                #[watch]
                set_uri: &model.bugs.text()
            },
          }
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = PackageInfoModel {
            hidden: true,
            pkg_name: gtk::EntryBuffer::default(),
            version: gtk::EntryBuffer::default(),
            author: gtk::EntryBuffer::default(),
            repo: gtk::EntryBuffer::default(),
            bugs: gtk::EntryBuffer::default(),
            description: gtk::EntryBuffer::default(),
            homepage: gtk::EntryBuffer::default(),
            utils: FormUtils::new(),
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            PackageInfoInput::Show(path) => {
                println!("{}", path);
                let package_json: PartialPackageJsonData = self.read_package_json(path);
                let _ = sender.output(PackageInfoOutput::PackageData(package_json.clone()));
                self.populate_view(package_json);
                self.hidden = false;
            }
        }
    }
}
