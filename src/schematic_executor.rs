use crate::command_builder::{CommandBuilder, InputType, Param};
use crate::form_utils::FormUtils;
use crate::impl_validation;
use crate::schema_parsing::FsEntry;
use crate::settings_utils::SettingsData;
use crate::traits::Validator;
use relm4::gtk::prelude::{
    BoxExt, ButtonExt, Cast, DialogExt, DisplayExt, EditableExt, EntryBufferExtManual, EntryExt,
    FileChooserExt, FileExt, GtkWindowExt, OrientableExt, TextBufferExt, TextViewExt, WidgetExt,
};
use relm4::gtk::{Align, EntryBuffer, Inhibit, ResponseType, TextBuffer, Window};
use relm4::RelmWidgetExt;
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};

async fn run_schematic(
    executor: String,
    command: String,
    cwd: String,
    params: Vec<Param>,
) -> Child {
    let mut cmd = Command::new(executor);

    cmd.current_dir(cwd);
    cmd.arg(command);

    for param in params {
        cmd.arg(format!("--{}", param.name));
        cmd.arg(format!("{}", param.value));
    }

    let child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child
}

#[derive(Debug)]
pub enum CommandMsg {
    Data(Child),
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
    success: bool,
    message: String,
    settings: Option<SettingsData>,
    use_dry_run: bool,
}

impl SchematicExecutorModel {
    fn reset_view(&mut self) -> () {
        self.executing = false;
        self.submitted = false;
        self.output_buf.set_text(&String::default());
        self.error_buf.set_text(&String::default());
        self.clear_error();
        self.hidden = false;
    }

    fn validate(&mut self) -> bool {
        let cwd = self.cwd_buf.text().to_string();
        let path = Path::new(&cwd);

        if cwd.len() == 0 {
            self.print_error("The cwd field is mandatory!");
            return false;
        } else if !path.exists() || !path.is_dir() {
            self.print_error(&format!(
                "The '{}' doesn't exist or it's not a directory!",
                cwd
            ));
            return false;
        }
        true
    }

    fn has_dry_run(&self) -> bool {
        if self.settings.is_none() {
            return false;
        }

        let settings = self.settings.clone().unwrap();
        settings.google_runner || settings.mbh_runner
    }

    fn set_output<T: std::io::Read + std::marker::Send + std::marker::Sync + 'static>(
        &self,
        sender: ComponentSender<Self>,
        mut stream: Option<T>,
        is_error: bool,
    ) {
        std::thread::spawn(move || {
            let out = stream.as_mut().unwrap();
            let out_reader = BufReader::new(out);
            let out_lines = out_reader.lines();
            for line in out_lines {
                let str =
                    String::from_utf8(strip_ansi_escapes::strip(&String::from(line.unwrap())))
                        .unwrap();
                if !is_error {
                    sender.input(SchematicExecutorInput::SetOutput(str));
                } else {
                    sender.input(SchematicExecutorInput::SetErrorOutput(str));
                }
            }
        });
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
    ClearOutput,
    AllowGoogleOptions(bool),
    SetCwd(String),
    CopyToClipboard,
    SetOutput(String),
    Done,
    SetErrorOutput(String),
}

#[derive(Debug)]
pub enum SchematicExecutorOutput {
    BackToUi,
    CwdChanged(String),
}

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
          set_css_classes: &["content_area"],
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
                set_label: &format!("Error: {}", &model.message)
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
                      let dialog = FormUtils::new().file_chooser("Working directory",&button.root().unwrap().downcast::<Window>().unwrap(),None,Some(FsEntry {
                        is_dir: true,
                        ..Default::default()
                      }));
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
            gtk::Box {
              set_orientation: gtk::Orientation::Vertical,
              #[watch]
              set_visible: model.has_dry_run(),
              gtk::Label {
                    set_hexpand: true,
                    set_vexpand: false,
                    set_css_classes: &["label"],
                    set_halign: gtk::Align::Start,
                    set_label: "Use dry run"
                },
                append: dry_run = &gtk::Switch {
                    set_hexpand: false,
                    set_vexpand: false,
                    set_active: false,
                    set_halign: gtk::Align::End,
                    set_valign: gtk::Align::Start,
                    set_css_classes: &["switch"],
                    connect_state_set[sender] => move |_,state| {
                      sender.input(SchematicExecutorInput::AllowGoogleOptions(state));
                      Inhibit(false)
                    }
                },
            },
              gtk::Box {
              set_orientation: gtk::Orientation::Horizontal,
              set_halign: Align::End,
              set_valign: Align::Start,
              gtk::Button {
                  set_hexpand: false,
                  set_vexpand: false,
                  set_label: "Edit",
                  set_css_classes: &["button", "action"],
                  connect_clicked[sender] => move |_| {
                    let _ = sender.output_sender().send(SchematicExecutorOutput::BackToUi);
                  },
                  set_tooltip_text: Some("Clear output and errors"),
                  #[watch]
                  set_visible: model.submitted && !model.executing,
                  #[watch]
                  set_sensitive: model.submitted && !model.executing
              },
              gtk::Button {
                  set_hexpand: false,
                  set_vexpand: false,
                  set_label: "Clear",
                  set_css_classes: &["button", "action"],
                  connect_clicked[sender] => move |_| {
                    let _ = sender.input_sender().send(SchematicExecutorInput::ClearOutput);
                  },
                  set_tooltip_text: Some("Clear output and errors"),
                  #[watch]
                  set_visible: model.submitted && !model.executing,
                  #[watch]
                  set_sensitive: model.submitted && !model.executing
              },
              gtk::Button {
                set_hexpand: false,
                set_vexpand: false,
                set_label: "Execute",
                set_tooltip_text: Some("Run schematic"),
                set_css_classes: &["button", "action"],
                connect_clicked[sender] => move |_| {
                  let _ = sender.input_sender().send(SchematicExecutorInput::Execute);
                },
                #[watch]
                  set_sensitive: !model.executing
              },
              gtk::Spinner {
                set_height_request: 25,
                set_width_request: 25,
                set_halign: gtk::Align::Center,
                set_spinning: true,
                set_css_classes: &["task_loading"],
                #[watch]
                set_visible: model.executing,
              },
            },
            gtk::Box {
               set_orientation: gtk::Orientation::Vertical,
               #[watch]
               set_visible: true,
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
                  set_buffer: Some(&model.output_buf),
                }
              },
              gtk::Label {
                set_hexpand: true,
                set_vexpand: false,
                set_css_classes: &["label"],
                set_halign: gtk::Align::Start,
                set_label: "Errors / Warnings"
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
            success: true,
            message: String::default(),
            settings: None,
            use_dry_run: false,
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _: &Self::Root,
    ) {
        match message {
            CommandMsg::Data(mut child) => {
                self.set_output::<ChildStdout>(sender.clone(), child.stdout.take(), false);
                self.set_output::<ChildStderr>(sender.clone(), child.stderr.take(), true);

                std::thread::spawn(move || {
                    child.wait().unwrap();
                    sender.input(SchematicExecutorInput::Done);
                });
            }
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            SchematicExecutorInput::Show(data) => {
                self.reset_view();

                self.settings = Some(data.settings);
                self.builder.set_params(data.params);
                self.builder.set_command(data.schematic);
                self.builder
                    .set_executable(self.settings.clone().unwrap().runner_location);

                self.command_buf.set_text(&format!(
                    "{} {} {}",
                    self.builder.get_executable(),
                    self.builder.get_command(),
                    self.builder.to_string(None)
                ));
            }
            SchematicExecutorInput::Execute => {
                let cwd = self.cwd_buf.text().to_string();

                if !self.validate() {
                    return;
                }

                self.clear_error();
                self.executing = true;
                self.submitted = true;

                let mut params = self.builder.to_params();

                if self.has_dry_run() && self.use_dry_run {
                    params.push(Param::new(
                        String::from("dry-run"),
                        String::from("true"),
                        InputType::Text,
                    ));
                    params.push(Param::new(
                        String::from("no-interactive"),
                        String::from("true"),
                        InputType::Text,
                    ));
                }

                let command = self.builder.get_command();
                let executor = self.settings.as_ref().unwrap().runner_location.clone();

                sender.oneshot_command(async move {
                    CommandMsg::Data(run_schematic(executor, command, cwd, params).await)
                });
            }
            SchematicExecutorInput::CopyToClipboard => {
                let clip = gtk::gdk::Display::default().unwrap().clipboard();
                clip.set_text(&self.command_buf.text());
                    
            }
            SchematicExecutorInput::AllowGoogleOptions(allow) => {
                self.use_dry_run = allow;
            }
            SchematicExecutorInput::SetOutput(str) => {
                self.output_buf.insert_at_cursor(&format!("{}\n", str));
            }
            SchematicExecutorInput::SetErrorOutput(str) => {
                self.error_buf.insert_at_cursor(&format!("{}\n", str));
            }
            SchematicExecutorInput::Done => {
                self.executing = false;
            }
            SchematicExecutorInput::ClearOutput => {
                self.output_buf.set_text("");
                self.error_buf.set_text("");
            }
            SchematicExecutorInput::SetCwd(path) => {
                self.cwd_buf.set_text(path.clone());
                let _ = sender
                    .output_sender()
                    .send(SchematicExecutorOutput::CwdChanged(path));
            }
        }
    }
}
