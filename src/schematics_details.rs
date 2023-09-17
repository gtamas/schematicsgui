use std::path::Path;

use relm4::gtk::prelude::WidgetExt;
use relm4::{
    gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller,
    SimpleComponent,
};

use crate::package_info::{PackageInfoInput, PackageInfoModel};
use crate::schema_view::{SchemaViewInput, SchemaViewModel};
use crate::schematic_executor::SchematicExecutorModel;
use crate::schematic_ui::{SchematicUiInput, SchematicUiModel};
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
}

impl SchematicsDetailsModel {
    fn show_package(&mut self) {
        self.tab = 0;
    }

    fn show_ui(&mut self) {
        self.tab = 1;
    }

    fn show_schema(&mut self) {
        self.tab = 2;
    }

    fn show_shell(&mut self) {
        self.tab = 3;
    }
}

#[derive(Debug)]
pub enum SchematicsDetailsInput {
    Show(Option<SettingsData>),
    ShowSchematic(String),
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
          set_tab_pos: gtk::PositionType::Top,
          set_group_name: Some("tabs"),
          #[watch]
          set_visible: !model.hidden,
          #[watch]
          set_current_page: Some(model.tab),
          append_page: (model.info.widget(), Some(&gtk::Label::new(Some("Package")))),
          append_page: (model.ui.widget(), Some(&gtk::Label::new(Some("Interface")))),
          append_page: (model.schema.widget(), Some(&gtk::Label::new(Some("Schema")))),
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
                _ => SchematicsDetailsInput::Show(None),
            });

        let schema_view = SchemaViewModel::builder().launch(true).forward(
            sender.input_sender(),
            |msg| match msg {
                _ => SchematicsDetailsInput::Show(None),
            },
        );

        let schematic_ui = SchematicUiModel::builder().launch(true).forward(
            sender.input_sender(),
            |msg| match msg {
                _ => SchematicsDetailsInput::Show(None),
            },
        );

        let schematic_executor: Controller<SchematicExecutorModel> =
            SchematicExecutorModel::builder()
                .launch(true)
                .forward(sender.input_sender(), |msg| match msg {
                    _ => SchematicsDetailsInput::Show(None),
                });

        let model = SchematicsDetailsModel {
            hidden: true,
            schema: schema_view,
            settings: None,
            tab: 0,
            info,
            ui: schematic_ui,
            executor: schematic_executor,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            SchematicsDetailsInput::ShowSchematic(schematic_name) => {
                let settings = self.settings.clone().unwrap();
                let collection = Collection::new(&settings.schematics_collection);
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
                    .send(SchematicUiInput::Show(path.clone()))
                    .unwrap();

                self.show_ui();
            }
            SchematicsDetailsInput::Show(data) => {
                let pkg = data.clone().unwrap().schematics_package;
                self.info
                    .sender()
                    .send(PackageInfoInput::Show(pkg.to_string()))
                    .unwrap();
                self.settings = Some(data.clone().unwrap());
                self.hidden = false
            }
        }
    }
}
