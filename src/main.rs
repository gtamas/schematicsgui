use gtk::prelude::{ApplicationExt, BoxExt, GtkWindowExt, OrientableExt};
use relm4::actions::{AccelsPlus, RelmAction, RelmActionGroup};
use relm4::gtk::traits::{ApplicationWindowExt, FrameExt, GtkApplicationExt, WidgetExt};
use relm4::gtk::{CssProvider, ShortcutsSection, ShortcutsWindow};
use relm4::{
    gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp,
    RelmWidgetExt, SimpleComponent,
};
use schematics_gui_reml::about::AppAboutDialog;
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
    ShowAbout,
    ShowSettings,
    ShowShortCuts,
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("style.css"));

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

struct AppModel {
    mode: AppMode,
    dialog: Controller<SettingsModel>,
    selector: Controller<SchematicSelectorModel>,
    tabs: Controller<SchematicsDetailsModel>,
}

impl AppModel {}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = AppMode;
    type Input = AppMsg;
    type Output = ();

    view! {
        main_window = gtk::ApplicationWindow {
            set_default_width: 1200,
            set_default_height: 800,
            set_resizable: true,
            set_maximized: true,
            #[watch]
            set_can_focus: true,
            set_title: Some("Schematics GUI"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                gtk::Paned {
                  set_orientation: gtk::Orientation::Horizontal,
                  #[wrap(Some)]
                  set_start_child: selector = &gtk::Frame {
                    set_hexpand: true,
                    set_css_classes: &["selector_container"],
                    #[wrap(Some)]
                    set_label_widget: label = &gtk::Box {
                      set_css_classes: &["left_header_container"],
                      set_hexpand: true,
                      gtk::Label {
                        set_css_classes: &["left_header_label"],
                        set_label: &"Schematics",
                        set_hexpand: true,
                        set_xalign: 0.0,
                      },
                    },
                    // set_size_request[300]: 700,
                    set_child: Some(model.selector.widget())
                  },
                  set_resize_start_child: true,
                  set_shrink_start_child: false,
                  #[wrap(Some)]
                  set_end_child: details = &gtk::Frame {
                    set_css_classes: &["tabs_container"],
                    set_label: Some("Details"),
                    // set_size_request[600]: 700,
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

        relm4::menu! {
            main_menu: {
              "Options" {
                "About" => AboutAction,
                "Settings" => SettingsAction,
                "Help" => HelpAction,
                "Quit" => QuitAction,
              }
            }
        };

        let app = relm4::main_application();

        app.set_accelerators_for_action::<AboutAction>(&["<primary>A"]);
        app.set_accelerators_for_action::<QuitAction>(&["<primary>Q"]);
        app.set_accelerators_for_action::<SettingsAction>(&["<primary>S"]);
        app.set_accelerators_for_action::<HelpAction>(&["<primary>H"]);

        let about_action: RelmAction<AboutAction> = {
            let sender = sender.clone();
            RelmAction::new_stateless(move |_| sender.input(AppMsg::ShowAbout))
        };

        let close_action: RelmAction<QuitAction> = {
            let sender = sender.clone();
            RelmAction::new_stateless(move |_| sender.input(AppMsg::Close))
        };

        let settings_action: RelmAction<SettingsAction> = {
            let sender = sender.clone();
            RelmAction::new_stateless(move |_| sender.input(AppMsg::ShowSettings))
        };

        let help_action: RelmAction<HelpAction> = {
            let sender = sender.clone();
            RelmAction::new_stateless(move |_| sender.input(AppMsg::ShowShortCuts))
        };

        // let action2: RelmAction<ExampleU8Action> =
        //     RelmAction::new_stateful_with_target_value(&0, |_, state, _value| {
        //         *state ^= 1;
        //         dbg!(state);
        //     });

        let mut group = RelmActionGroup::<WindowActionGroup>::new();
        group.add_action(about_action);
        group.add_action(settings_action);
        group.add_action(help_action);
        group.add_action(close_action);
        group.register_for_widget(&widgets.main_window);

        app.set_menubar(Some(&main_menu));

        println!("{:?}", app.list_action_descriptions());

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
                            let settings = settings_util.read();
                            self.selector
                                .sender()
                                .send(SchematicSelectorInput::Show(settings.clone()))
                                .unwrap();

                            self.tabs
                                .sender()
                                .send(SchematicsDetailsInput::Show(Some(settings.clone())))
                                .unwrap();
                        }
                    }
                    AppMode::SettingsLoaded(data) => {
                        self.selector
                            .sender()
                            .send(SchematicSelectorInput::Show(data.clone()))
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

            AppMsg::ShowAbout => {
                AppAboutDialog::show();
            }

            AppMsg::ShowSettings => {
                self.dialog.sender().send(SettingsInput::Show).unwrap();
            }

            AppMsg::ShowShortCuts => {
                let section = ShortcutsSection::builder()
                    .section_name("Shortcuts")
                    .build();

                let builder = ShortcutsWindow::builder().section_name("");
            }
        }
    }
}

relm4::new_action_group!(WindowActionGroup, "win");

relm4::new_stateless_action!(AboutAction, WindowActionGroup, "about");
relm4::new_stateless_action!(QuitAction, WindowActionGroup, "quit");
relm4::new_stateless_action!(SettingsAction, WindowActionGroup, "settings");
relm4::new_stateless_action!(HelpAction, WindowActionGroup, "help");
// relm4::new_stateful_action!(ExampleU8Action, WindowActionGroup, "example2", u8, u8);

fn main() {
    let app = RelmApp::new("schematics.gui");
    relm4_icons::initialize_icons();
    relm4::main_application().connect_startup(|_| load_css());
    app.run::<AppModel>(AppMode::Initial);
}
