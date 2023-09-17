use gtk::prelude::{ApplicationExt, BoxExt, GtkWindowExt, OrientableExt};
use relm4::gtk::traits::{ApplicationWindowExt, FrameExt, GtkApplicationExt, WidgetExt};
use relm4::{
    gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp,
    RelmWidgetExt, SimpleComponent,
};
use relm4::{menu, new_action_group, new_stateless_action};
use schematics_gui_reml::schematic_selector::{
    SchematicSelectorInput, SchematicSelectorModel, SchematicSelectorOutput,
};
use schematics_gui_reml::schematics_details::{SchematicsDetailsInput, SchematicsDetailsModel};
use schematics_gui_reml::settings::*;
use schematics_gui_reml::settings_utils::{SettingsData, SettingsUtils};

#[derive(Debug, PartialEq, Clone)]
enum AppMode {
    Initial,
    SettingsLoaded(SettingsData),
    ShowSchematic(String),
}

#[derive(Debug)]
enum AppMsg {
    SetMode(AppMode),
    CloseRequest,
    Close,
}

struct AppModel {
    mode: AppMode,
    dialog: Controller<SettingsModel>,
    selector: Controller<SchematicSelectorModel>,
    tabs: Controller<SchematicsDetailsModel>,
}

impl AppModel {
    fn setup_app_menu() {
        menu! {
          main_menu: {
              custom: "my_widget",
              "Test" => TestAction,
              "Test2" => TestAction,

              section! {
                  "Section test" => TestAction,

              },
              section! {
                  "Test" => TestAction,
                  "Test2" => TestAction,

              }
          }
        }
        relm4::main_application().set_menubar(Some(&main_menu));
    }
}

new_action_group!(WindowActionGroup, "win");
new_stateless_action!(TestAction, WindowActionGroup, "test");

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = AppMode;
    type Input = AppMsg;
    type Output = ();

    view! {
        main_window = gtk::ApplicationWindow {
            set_show_menubar: true,
            set_default_width: 800,
            set_default_height: 600,
            set_resizable: true,
            set_maximized: true,
            #[watch]
            set_can_focus: true,
            set_title: Some("Schematics GUI"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,
                gtk::Paned {
                  set_orientation: gtk::Orientation::Horizontal,

                  #[wrap(Some)]
                  set_start_child: selector = &gtk::Frame {
                    set_label: Some("Schematics"),
                    set_size_request[300]: 600,
                    set_child: Some(model.selector.widget())
                  },
                  set_resize_start_child: true,
                  set_shrink_start_child: false,
                  #[wrap(Some)]
                  set_end_child: details = &gtk::Frame {
                    set_label: Some("Details"),
                    set_size_request[600]: 600,
                    set_child: Some(model.tabs.widget())
                  },
                  set_resize_end_child: true,
                  set_shrink_end_child: false,
                  set_margin_all: 5,
                }
            },
            connect_close_request[sender] => move |_| {
                sender.input(AppMsg::CloseRequest);
                gtk::Inhibit(true)
            }
        }
    }

    fn init(
        params: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        Self::setup_app_menu();

        let dialog = SettingsModel::builder()
            .transient_for(root)
            .launch(true)
            .forward(sender.input_sender(), |msg| match msg {
                SettingsOutput::Close => AppMsg::Close,
                SettingsOutput::SettingsLoaded(settings) => {
                    AppMsg::SetMode(AppMode::SettingsLoaded(settings.clone()))
                }
            });

        let selector =
            SchematicSelectorModel::builder()
                .launch(true)
                .forward(sender.input_sender(), |msg| match msg {
                    SchematicSelectorOutput::Load => AppMsg::Close,
                    SchematicSelectorOutput::Selected(schematic) => {
                        AppMsg::SetMode(AppMode::ShowSchematic(schematic))
                    }
                });

        let tabs =
            SchematicsDetailsModel::builder()
                .launch(true)
                .forward(sender.input_sender(), |msg| match msg {
                    _ => AppMsg::Close,
                });

        let model = AppModel {
            mode: params,
            dialog,
            selector,
            tabs,
        };

        sender
            .input_sender()
            .send(AppMsg::SetMode(AppMode::Initial))
            .unwrap();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            AppMsg::SetMode(mode) => {
                self.mode = mode.clone();
                match mode {
                    AppMode::ShowSchematic(schematic) => {
                        self.tabs
                            .sender()
                            .send(SchematicsDetailsInput::ShowSchematic(schematic))
                            .unwrap();
                    }
                    AppMode::Initial => {
                        let settings_util = SettingsUtils::new();
                        if !settings_util.exists() {
                            self.dialog.sender().send(SettingsInput::Show).unwrap();
                        } else {
                            self.selector
                                .sender()
                                .send(SchematicSelectorInput::Show)
                                .unwrap();

                            self.tabs
                                .sender()
                                .send(SchematicsDetailsInput::Show(Some(settings_util.read())))
                                .unwrap();
                        }
                    }
                    AppMode::SettingsLoaded(data) => {
                        self.selector
                            .sender()
                            .send(SchematicSelectorInput::Show)
                            .unwrap();

                        self.tabs
                            .sender()
                            .send(SchematicsDetailsInput::Show(Some(data.clone())))
                            .unwrap();
                    }
                }
            }

            AppMsg::CloseRequest => {
                if !self.dialog.model().hidden {
                    self.dialog.sender().send(SettingsInput::Apply).unwrap();
                }
                // TODO: wait for this..
                sender.input_sender().send(AppMsg::Close).unwrap();
            }

            AppMsg::Close => {
                relm4::main_application().quit();
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("schematics.gui");
    relm4_icons::initialize_icons();
    app.run::<AppModel>(AppMode::Initial);
}
