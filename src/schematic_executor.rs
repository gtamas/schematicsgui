use relm4::gtk::prelude::{
    BoxExt, ButtonExt, EntryBufferExtManual, EntryExt, OrientableExt, TextBufferExt, TextViewExt,
    WidgetExt,
};
use relm4::gtk::{EntryBuffer, TextBuffer};
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};
use std::process::Command;

use crate::command_builder::{CommandBuilder, Param};

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
}

#[derive(Debug)]
pub enum SchematicExecutorOutput {}

#[relm4::component(pub)]
impl SimpleComponent for SchematicExecutorModel {
    type Input = SchematicExecutorInput;
    type Output = SchematicExecutorOutput;
    type Init = bool;

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
              set_halign: gtk::Align::Start,
              set_label: "Command"
            },
            gtk::Entry {
              set_hexpand: true,
              set_vexpand: false,
              set_buffer: &model.command_buf
            },
            gtk::Button {
              set_hexpand: false,
              set_vexpand: false,
              set_label: "Execute",
              connect_clicked[sender] => move |_| {
                let _ = sender.input_sender().send(SchematicExecutorInput::Execute);
              }
            },
            gtk::Spinner {
              set_height_request: 200,
              set_halign: gtk::Align::Center,
              set_spinning: true,
              start: (),
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
                set_label: "Result"
              },
              gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                  gtk::TextView {
                  set_hexpand: true,
                  set_vexpand: true,
                  set_buffer: Some(&model.output_buf)
                }
              },
              gtk::Label {
                set_hexpand: true,
                set_vexpand: false,
                set_halign: gtk::Align::Start,
                set_label: "Errors"
              },
              gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                  gtk::TextView {
                  set_hexpand: true,
                  set_vexpand: true,
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

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            SchematicExecutorInput::Show(data) => {
                self.executing = false;
                self.submitted = false;
                self.builder.set_params(data.params.clone());
                self.builder.set_command(data.schematic);
                self.hidden = false;
                self.command_buf.set_text(&self.builder.to_string());
                self.output_buf.set_text(&String::default());
                self.error_buf.set_text(&String::default());
            }
            SchematicExecutorInput::Execute => {
                self.executing = true;
                self.submitted = true;
                let mut cmd = Command::new("fnd");
                let params = self.builder.to_params();

                cmd.current_dir("/Users/tamas/work/mbh/mf-usl-task");
                cmd.arg(self.builder.get_command());
                cmd.arg("--dryRun");

                for param in params {
                    cmd.arg(format!("--{}", param.name));
                    cmd.arg(format!("{}", param.value));
                }

                let out = cmd.output().expect("ls command failed to start");
                self.executing = false;
                self.output_buf
                    .set_text(&String::from_utf8_lossy(&out.stdout));
                self.error_buf
                    .set_text(&String::from_utf8_lossy(&out.stderr));
            }
        }
    }
}
