use std::path::Path;

use relm4::gtk::prelude::WidgetExt;
use relm4::{
    gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller,
    SimpleComponent,
};

use crate::command_builder::Param;
use crate::package_info::{
    PackageInfoInput, PackageInfoModel, PackageInfoOutput, PartialPackageJsonData,
};
use crate::schema_view::{SchemaViewInput, SchemaViewModel};
use crate::schematic_executor::{
    SchematicExecutorInput, SchematicExecutorInputParams, SchematicExecutorModel,
    SchematicExecutorOutput,
};
use crate::schematic_ui::{
    SchematicUiInput, SchematicUiInputParams, SchematicUiModel, SchematicUiOutput,
};
use crate::schematics::Collection;
use crate::settings_utils::SettingsData;

pub struct SchematicsDetailsModel {
    hidden: bool,
    tab: u32,
    info: Controller<PackageInfoModel>,
    schema: Controller<SchemaViewModel>,
    ui: Controller<SchematicUiModel>,
    executor: Controller<SchematicExecutorModel>,
    settings: Option<SettingsData>,
    package: Option<PartialPackageJsonData>,
    schematic: String,
}

impl SchematicsDetailsModel {
    pub fn show_package(&mut self) {
        self.tab = 0;
    }

    pub fn show_schema(&mut self) {
        self.tab = 1;
    }

    pub fn show_ui(&mut self) {
        self.tab = 2;
    }

    pub fn show_shell(&mut self) {
        self.tab = 3;
    }
}

#[derive(Debug)]
pub enum SchematicsDetailsInput {
    Show(Option<SettingsData>),
    ShowSchematic(String),
    ShowExecutor(Vec<Param>),
    SetPackage(Box<PartialPackageJsonData>),
    BacktToUi,
    CwdChanged(String),
}

#[derive(Debug)]
pub enum SchematicsDetailsOutput {
    LoadPackageInfo(String),
}

pub struct SchematicsDetailsInit {}

#[relm4::component(pub)]
impl SimpleComponent for SchematicsDetailsModel {
    type Input = SchematicsDetailsInput;
    type Output = SchematicsDetailsOutput;
    type Init = bool;

    view! {
        #[root]
        gtk::Notebook {
          set_css_classes: &["tabs"],
          set_tab_pos: gtk::PositionType::Top,
          set_group_name: Some("tabs"),
          #[watch]
          set_visible: !model.hidden,
          #[watch]
          set_current_page: Some(model.tab),
          append_page: (model.info.widget(), Some(&gtk::Label::new(Some("Package")))),
          append_page: (model.schema.widget(), Some(&gtk::Label::new(Some("Schema")))),
          append_page: (model.ui.widget(), Some(&gtk::Label::new(Some("Interface")))),
          append_page: (model.executor.widget(), Some(&gtk::Label::new(Some("Execute")))),
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let info = PackageInfoModel::builder()
            .launch(true)
            .forward(sender.input_sender(), |msg| match msg {
                PackageInfoOutput::PackageData(pkg) => {
                    SchematicsDetailsInput::SetPackage(Box::new(pkg))
                }
            });

        let schema_view = SchemaViewModel::builder().launch(true).detach();

        let schematic_ui = SchematicUiModel::builder().launch(true).forward(
            sender.input_sender(),
            |msg: SchematicUiOutput| match msg {
                SchematicUiOutput::Params(p) => SchematicsDetailsInput::ShowExecutor(p),
            },
        );

        let schematic_executor: Controller<SchematicExecutorModel> =
            SchematicExecutorModel::builder()
                .launch(true)
                .forward(sender.input_sender(), |msg| match msg {
                    SchematicExecutorOutput::BackToUi => SchematicsDetailsInput::BacktToUi,
                    SchematicExecutorOutput::CwdChanged(path) => {
                        SchematicsDetailsInput::CwdChanged(path)
                    }
                });

        let model = SchematicsDetailsModel {
            hidden: true,
            schema: schema_view,
            settings: None,
            package: None,
            tab: 0,
            info,
            ui: schematic_ui,
            executor: schematic_executor,
            schematic: String::default(),
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            SchematicsDetailsInput::ShowSchematic(schematic_name) => {
                let settings = self.settings.as_ref().unwrap();
                let mut collection = Collection::new(settings.clone());
                collection.init();
                let schematic = collection.get_schematic(&schematic_name);
                let path = Path::new(&settings.schematics_collection)
                    .parent()
                    .unwrap()
                    .join(schematic.schema)
                    .canonicalize()
                    .unwrap();

                self.schema
                    .sender()
                    .send(SchemaViewInput::Show(path.clone()))
                    .unwrap();

                self.ui
                    .sender()
                    .send(SchematicUiInput::Show(SchematicUiInputParams {
                        schema_path: path.clone(),
                        schematic: schematic_name.clone(),
                        package_name: self.package.as_ref().unwrap().name.clone(),
                    }))
                    .unwrap();

                self.show_ui();
                self.schematic = schematic_name.clone();
            }
            SchematicsDetailsInput::Show(data) => {
                self.settings = Some(data.clone().unwrap());
                let pkg = &self.settings.as_ref().unwrap().schematics_package;
                self.info
                    .sender()
                    .send(PackageInfoInput::Show(pkg.to_string()))
                    .unwrap();

                self.hidden = false
            }
            SchematicsDetailsInput::BacktToUi => {
                self.show_ui();
            }
            SchematicsDetailsInput::CwdChanged(path) => {
                self.ui
                    .sender()
                    .send(SchematicUiInput::CwdChanged(path))
                    .unwrap();
            }
            SchematicsDetailsInput::SetPackage(data) => {
                self.package = Some(*data);
            }
            SchematicsDetailsInput::ShowExecutor(params) => {
                self.executor
                    .sender()
                    .send(SchematicExecutorInput::Show(SchematicExecutorInputParams {
                        params,
                        schematic: self.schematic.clone(),
                        settings: self.settings.as_ref().unwrap().clone(),
                        package_name: self.package.as_ref().unwrap().name.clone(),
                    }))
                    .unwrap();

                self.show_shell();
            }
        }
    }
}
