#[derive(Debug, Clone, PartialEq)]
pub struct CommandBuilder {
    params: Vec<Param>,
    command: String,
    options: CommandBuilderOptions,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub value: String,
    pub kind: InputType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputType {
    Text,
    TextArea,
    ColorButton,
    ColorInput,
    Date,
    File,
    Dir,
    Checkbox,
    Toggle,
    RadioGroup,
    ToggleGroup,
    DropDown,
    Combobox,
    Slider,
    Numeric,
    Switch,
}

impl Default for Param {
    fn default() -> Self {
        Param {
            kind: InputType::Text,
            name: String::default(),
            value: String::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandBuilderOptions {
    escape_multiline_text: bool,
    quote_paths: bool,
}

impl Default for CommandBuilderOptions {
    fn default() -> Self {
        CommandBuilderOptions {
            escape_multiline_text: false,
            quote_paths: false,
        }
    }
}

impl CommandBuilder {
    pub fn new(options: Option<CommandBuilderOptions>) -> Self {
        CommandBuilder {
            params: Vec::<Param>::new(),
            command: String::default(),
            options: options.unwrap_or_default(),
        }
    }

    fn escape_str(&self, value: &str) -> String {
        if self.options.escape_multiline_text {
            return format!("`{}`", value.replace(&['\r', '\n'], "\\\n"));
        }

        value.replace(&['\r', '\n'], " ")
    }

    fn escape_path(&self, path: &str) -> String {
        format!("\"{}\"", path.replace(&['"'], "\""))
    }

    pub fn set_params(&mut self, params: Vec<Param>) {
        self.params = params;
    }

    pub fn set_command(&mut self, command: String) {
        self.command = command;
    }

    pub fn get_command(&self) -> String {
        self.command.clone()
    }

    pub fn add(&mut self, param: Param) {
        self.params.push(param)
    }

    pub fn to_params(&self) -> Vec<Param> {
        self.params.clone()
    }

    pub fn to_string(&self) -> String {
        self.params
            .clone()
            .iter()
            .map(|m| {
                format!("--{} {}", m.name, {
                    if m.kind == InputType::TextArea {
                        self.escape_str(&m.value)
                    } else if self.options.quote_paths
                        && (m.kind == InputType::File || m.kind == InputType::Dir)
                    {
                        self.escape_path(&m.value)
                    } else {
                        m.value.clone()
                    }
                })
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
}
