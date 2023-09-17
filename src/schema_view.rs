use std::path::PathBuf;

use relm4::gtk::prelude::{TextBufferExt, TextViewExt, WidgetExt};
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};
use serde_json::Value;

use crate::schematics::Collection;

pub struct SchemaViewModel {
    hidden: bool,
    json: gtk::TextBuffer,
}

#[derive(Debug)]
pub enum SchemaViewInput {
    Show(PathBuf),
}

#[derive(Debug)]
pub enum SchemaViewOutput {}

#[relm4::component(pub)]
impl SimpleComponent for SchemaViewModel {
    type Input = SchemaViewInput;
    type Output = SchemaViewOutput;
    type Init = bool;

    view! {
        #[root]
        gtk::Box {
          set_hexpand: true,
          gtk::Label {
            #[watch]
            set_visible: model.hidden,
            set_hexpand: true,
            set_halign: gtk::Align::Center,
            set_label: "Please, select a schematic!"
          },
          gtk::ScrolledWindow {
          set_hscrollbar_policy: gtk::PolicyType::Never,
          gtk::TextView {
            #[watch]
            set_visible: !model.hidden,
            set_hexpand: true,
            set_buffer: Some(&model.json)
          }
          }
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SchemaViewModel {
            hidden: true,
            json: gtk::TextBuffer::default(),
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            SchemaViewInput::Show(schema_path) => {
                let schema: Value =
                    serde_json::from_str(&Collection::read_str(schema_path.to_str().unwrap()))
                        .unwrap();
                self.json
                    .set_text(&serde_json::to_string_pretty(&schema).unwrap());
                self.hidden = false
            }
        }
    }
}