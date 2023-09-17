use std::path::PathBuf;

use relm4::gtk::prelude::WidgetExt;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

pub struct SchematicExecutorModel {
    hidden: bool,
    data: Option<SchematicExecutorData>,
}

#[derive(Debug)]
pub struct SchematicExecutorData {
    pub path: PathBuf,
    pub command: String,
}

#[derive(Debug)]
pub enum SchematicExecutorInput {
    Show(SchematicExecutorData),
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
          gtk::Label {
            #[watch]
            set_visible: model.hidden,
            set_hexpand: true,
            set_halign: gtk::Align::Center,
            set_label: "Please, fill in the input form!"
          },
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SchematicExecutorModel {
            hidden: true,
            data: None,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            SchematicExecutorInput::Show(data) => {
                self.data = Some(data);
                self.hidden = false
            }
        }
    }
}
