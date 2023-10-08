use relm4::gtk::prelude::{
    BoxExt, ButtonExt, EntryBufferExtManual, EntryExt, OrientableExt, TextBufferExt, TextViewExt,
    WidgetExt, DisplayExt,
};
use relm4::gtk::{EntryBuffer, TextBuffer, Align};
use relm4::{gtk, ComponentParts, ComponentSender, Component};
use std::process::{Command, Output};
use crate::command_builder::{CommandBuilder, Param};

async fn run_schematic(command: String, params: Vec<Param>) -> Output {
    // TODO: read executor from settings!
   let mut cmd = Command::new("fnd");

    cmd.current_dir("/Users/tamas/work/mbh/mf-usl-task");
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
    error_buf: TextBuffer,
}

#[derive(Debug, Clone)]
pub struct SchematicExecutorInputParams {
    pub params: Vec<Param>,
    pub schematic: String,
}

#[derive(Debug)]
pub enum SchematicExecutorInput {
    Show(SchematicExecutorInputParams),
    Execute,
    CopyToClipboard
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
        gtk::Box {
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
            gtk::Box {
              set_orientation: gtk::Orientation::Horizontal,
              gtk::Entry {
                set_hexpand: true,
                set_vexpand: false,
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
            output_buf: TextBuffer::default(),
            error_buf: TextBuffer::default(),
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
                let output_str = strip_ansi_escapes::strip(&String::from_utf8(data.stdout).unwrap());
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
                self.builder.set_params(data.params.clone());
                self.builder.set_command(data.schematic);
                self.hidden = false;
                // TODO: read executor from settings!
                self.command_buf.set_text(&format!("fnd {} {}", self.builder.get_command(), self.builder.to_params_string()));
                self.output_buf.set_text(&String::default());
                self.error_buf.set_text(&String::default());
            }
            SchematicExecutorInput::Execute => {
              self.executing = true;
              self.submitted = true;
              let params = self.builder.to_params();
              let command = self.builder.get_command();
              sender.oneshot_command(async move {
                    CommandMsg::Data(run_schematic(command, params).await)
                });
            },
            SchematicExecutorInput::CopyToClipboard => {
              let clip = gtk::gdk::Display::default().unwrap().clipboard();
              clip.set_text(&self.command_buf.text());
            }
        }
    }
}
