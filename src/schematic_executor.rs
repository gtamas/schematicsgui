use crate::command_builder::{CommandBuilder, Param};
use crate::form_utils::FormUtils;
use crate::impl_validation;
use crate::settings_utils::SettingsData;
use crate::traits::Validator;
use relm4::gtk::prelude::{
    BoxExt, ButtonExt, Cast, DialogExt, DisplayExt, EditableExt, EntryBufferExtManual, EntryExt,
    FileChooserExt, FileExt, GtkWindowExt, OrientableExt, TextBufferExt, TextViewExt, WidgetExt,
};
use relm4::gtk::{Align, EntryBuffer, ResponseType, TextBuffer, Window};
use relm4::RelmWidgetExt;
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use std::path::Path;
use std::process::{Command, Output};

async fn run_schematic(command: String, cwd: String, params: Vec<Param>) -> Output {
    // TODO: read executor from settings!
    let mut cmd = Command::new("fnd");

    cmd.current_dir(cwd);
    cmd.arg(command);
    cmd.arg("--dryRun");

    for param in params {
        cmd.arg(format!("--{}", param.name));
        cmd.arg(format!("{}", param.value));
    }

    let out = cmd.output().expect("Command failed to start");
    out
}

#[derive(Debug)]
pub enum CommandMsg {
    Data(Output),
}

pub struct SchematicExecutorModel {
    hidden: bool,
    executing: bool,
    submitted: bool,
    builder: CommandBuilder,
    command_buf: EntryBuffer,
    output_buf: TextBuffer,
    cwd_buf: EntryBuffer,
    error_buf: TextBuffer,
    error: bool,
    error_message: String,
    settings: Option<SettingsData>,
}

impl SchematicExecutorModel {
    fn validate(&mut self) -> bool {
        let cwd = self.cwd_buf.text().to_string();
        let path = Path::new(&cwd);

        if cwd.len() == 0 {
            self.set_error("The cwd field is mandatory!");
            return false;
        } else if !path.exists() || !path.is_dir() {
            self.set_error(&format!(
                "The '{}' doesn't exist or it's not a directory!",
                cwd
            ));
            return false;
        }
        true
    }
}

impl_validation!(SchematicExecutorModel);

#[derive(Debug, Clone)]
pub struct SchematicExecutorInputParams {
    pub params: Vec<Param>,
    pub schematic: String,
    pub settings: SettingsData,
}

#[derive(Debug)]
pub enum SchematicExecutorInput {
    Show(SchematicExecutorInputParams),
    Execute,
    SetCwd(String),
    CopyToClipboard,
}

#[derive(Debug)]
pub enum SchematicExecutorOutput {}

#[relm4::component(pub)]
impl Component for SchematicExecutorModel {
    type Input = SchematicExecutorInput;
    type Output = SchematicExecutorOutput;
    type Init = bool;
    type CommandOutput = CommandMsg;

    view! {
        #[root]
        r = gtk::Box {
          set_hexpand: true,
          set_orientation: gtk::Orientation::Vertical,
          gtk::Label {
            #[watch]
            set_visible: model.hidden,
            set_hexpand: true,
            set_vexpand: true,
            set_halign: gtk::Align::Center,
            set_label: "Please, select a schematic!"
          },
          gtk::Box {
            #[watch]
            set_visible: !model.hidden,
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,
            gtk::Label {
              set_hexpand: true,
              set_vexpand: false,
              set_css_classes: &["label", "label_title"],
              set_halign: gtk::Align::Start,
              set_label: "Command"
            },
            gtk::Revealer {
              set_transition_type: gtk::RevealerTransitionType::SlideDown,
              #[watch]
              set_reveal_child: model.error,
              gtk::Label {
                set_hexpand: true,
                set_vexpand: false,
                set_css_classes: &["label", "error"],
                set_halign: gtk::Align::Center,
                #[watch]
                set_label: &format!("Error: {}", &model.error_message)
              },
            },
            gtk::Label {
                set_hexpand: true,
                set_vexpand: false,
                set_css_classes: &["label"],
                set_halign: gtk::Align::Start,
                set_label: "Working directory"
            },
            gtk::Box {
              set_orientation: gtk::Orientation::Horizontal,
               gtk::Entry {
                set_hexpand: true,
                set_vexpand: false,
                set_css_classes: &["text_input", "cwd"],
                set_buffer: &model.cwd_buf,
                set_placeholder_text: Some("e.g: /foo/bar")
              },
              gtk::Button {
                set_hexpand: false,
                set_vexpand: false,
                set_height_request: 20,
                    set_icon_name: "document-open",
                    set_tooltip: "Browse file",
                    set_css_classes: &["button", "action_icon", "row_button"],
                    connect_clicked[sender] => move |button| {
                      let dialog = FormUtils::new().file_chooser("Schematics runner",&button.root().unwrap().downcast::<Window>().unwrap(),None,None);
                      let send = sender.clone();
                      dialog.connect_response(move |file_chooser, resp| {
                          match resp {
                            ResponseType::Cancel => file_chooser.close(),
                            ResponseType::Accept => {
                              let file_name = file_chooser.file().unwrap().parse_name().to_string();
                              file_chooser.close();
                              let _ = send.input_sender().send(SchematicExecutorInput::SetCwd(file_name));
                            },
                            _ => ()
                          }
                      });
                      dialog.show();

                    }
                  },
            },
            gtk::Box {
              set_orientation: gtk::Orientation::Horizontal,
              gtk::Entry {
                set_hexpand: true,
                set_vexpand: false,
                set_editable: false,
                set_can_focus: false,
                set_css_classes: &["text_input", "command_input"],
                set_buffer: &model.command_buf
              },
               gtk::Button {
                set_hexpand: false,
                set_vexpand: false,
                set_css_classes: &["button", "action_icon", "row_button"],
                set_tooltip_text: Some("copy to clipboard"),
                set_height_request: 20,
                set_icon_name: "copy",
                connect_clicked[sender] => move |_| {
                  let _ = sender.input_sender().send(SchematicExecutorInput::CopyToClipboard);
                }
              },
            },
            gtk::Button {
              set_hexpand: false,
              set_vexpand: false,
              set_label: "Execute",
              set_css_classes: &["button", "action"],
              connect_clicked[sender] => move |_| {
                let _ = sender.input_sender().send(SchematicExecutorInput::Execute);
              },
              set_halign: Align::End,
              set_valign: Align::Start,
            },
            gtk::Spinner {
              set_height_request: 100,
              set_width_request: 100,
              set_halign: gtk::Align::Center,
              set_spinning: true,
              set_css_classes: &["task_loading"],
              #[watch]
              set_visible: model.submitted && model.executing
            },
            gtk::Box {
               set_orientation: gtk::Orientation::Vertical,
               #[watch]
               set_visible: !model.executing && model.submitted,
              gtk::Label {
               set_hexpand: true,
                set_vexpand: false,
                set_halign: gtk::Align::Start,
                set_css_classes: &["label"],
                set_label: "Result"
              },
              gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                  gtk::TextView {
                  set_hexpand: true,
                  set_vexpand: true,
                  set_css_classes: &["task_output"],
                  set_buffer: Some(&model.output_buf)
                }
              },
              gtk::Label {
                set_hexpand: true,
                set_vexpand: false,
                set_css_classes: &["label"],
                set_halign: gtk::Align::Start,
                set_label: "Errors"
              },
              gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                  gtk::TextView {
                  set_hexpand: true,
                  set_vexpand: true,
                   set_css_classes: &["task_error"],
                  set_buffer: Some(&model.error_buf)
                }
              }
            },
          },
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SchematicExecutorModel {
            hidden: true,
            executing: false,
            submitted: false,
            builder: CommandBuilder::new(None),
            command_buf: EntryBuffer::default(),
            cwd_buf: EntryBuffer::default(),
            output_buf: TextBuffer::default(),
            error_buf: TextBuffer::default(),
            error: false,
            error_message: String::default(),
            settings: None,
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _: &Self::Root,
    ) {
        match message {
            CommandMsg::Data(data) => {
                self.executing = false;
                let output_str =
                    strip_ansi_escapes::strip(&String::from_utf8(data.stdout).unwrap());
                let error_str = strip_ansi_escapes::strip(&String::from_utf8(data.stderr).unwrap());
                self.output_buf
                    .set_text(&String::from_utf8_lossy(&output_str));
                self.error_buf
                    .set_text(&String::from_utf8_lossy(&error_str));
            }
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            SchematicExecutorInput::Show(data) => {
                self.executing = false;
                self.submitted = false;

                self.settings = Some(data.settings);
                self.builder.set_params(data.params);
                self.builder.set_command(data.schematic);
                self.builder
                    .set_executable(self.settings.clone().unwrap().runner_location);
                self.hidden = false;
                self.command_buf.set_text(&format!(
                    "{} {} {}",
                    self.builder.get_executable(),
                    self.builder.get_command(),
                    self.builder.to_string(None)
                ));
                self.output_buf.set_text(&String::default());
                self.error_buf.set_text(&String::default());
            }
            SchematicExecutorInput::Execute => {
                let cwd = self.cwd_buf.text().to_string();

                if !self.validate() {
                    return;
                }

                self.clear_error();
                self.executing = true;
                self.submitted = true;

                let params = self.builder.to_params();
                let command = self.builder.get_command();

                sender.oneshot_command(async move {
                    CommandMsg::Data(run_schematic(command, cwd, params).await)
                });
            }
            SchematicExecutorInput::CopyToClipboard => {
                let clip = gtk::gdk::Display::default().unwrap().clipboard();
                clip.set_text(&self.command_buf.text());
            }
            SchematicExecutorInput::SetCwd(path) => {
                self.cwd_buf.set_text(path);
            }
        }
    }
}
