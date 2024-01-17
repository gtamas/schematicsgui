use convert_case::{Case, Casing};

#[derive(Debug, Clone, PartialEq)]
pub struct CommandBuilder {
    params: Vec<Param>,
    command: String,
    executable: String,
    options: CommandBuilderOptions,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub value: String,
    pub kind: InputType,
}

impl Param {
    pub fn new(name: String, value: String, kind: InputType) -> Self {
        Param { name, value, kind }
    }
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
    Multiselect,
    Slider,
    Numeric,
    Switch,
    Time,
    DateTime,
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
    pub escape_multiline_text: bool,
    pub quote_paths: bool,
    pub option_case: Case,
    pub pass_boolean: bool,
    pub configurable: Option<String>,
}

impl Default for CommandBuilderOptions {
    fn default() -> Self {
        CommandBuilderOptions {
            escape_multiline_text: false,
            quote_paths: false,
            pass_boolean: true,
            configurable: None,
            option_case: Case::Camel,
        }
    }
}

impl CommandBuilder {
    pub fn new(options: Option<CommandBuilderOptions>) -> Self {
        CommandBuilder {
            params: Vec::<Param>::new(),
            command: String::default(),
            executable: String::default(),
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

    fn get_param_name(&self, param: &Param) -> String {
        if self.options.pass_boolean == false {
            if param.value == "true" {
                return param.name.to_case(self.options.option_case);
            } else if param.value == "false" {
                return format!("no-{}", param.name).to_case(self.options.option_case);
            }
        }
        param.name.to_case(self.options.option_case)
    }

    fn get_param_value(&self, param: &Param) -> String {
        match param.kind {
            InputType::Checkbox | InputType::Switch | InputType::Toggle => {
                if self.options.pass_boolean == false {
                    return String::default();
                }
                return param.value.clone();
            }
            InputType::TextArea => return self.escape_str(&param.value),
            InputType::File | InputType::Dir => {
                if self.options.quote_paths {
                    return self.escape_path(&param.value);
                }
                param.value.clone()
            }

            _ => {
                if self.options.configurable.is_some()
                    && self.options.configurable.clone().unwrap() == param.name
                    && param.value == "true"
                {
                    return String::default();
                }
                return param.value.clone();
            }
        }
    }

    pub fn set_configurable(&mut self, value: String) {
        self.options.configurable = Some(value);
    }

    pub fn set_params(&mut self, params: Vec<Param>) {
        for param in params {
            self.add(param);
        }
    }

    pub fn set_executable(&mut self, executable: String) {
        self.executable = executable;
    }

    pub fn get_executable(&self) -> String {
        self.executable.clone()
    }

    pub fn set_command(&mut self, command: String) {
        self.command = command;
    }

    pub fn get_command(&self) -> String {
        self.command.clone()
    }

    pub fn add(&mut self, param: Param) {
        self.params.push(Param {
            name: self.get_param_name(&param),
            value: self.get_param_value(&param),
            kind: param.kind,
        })
    }

    pub fn to_params(&self) -> Vec<Param> {
        self.params
            .clone()
            .iter()
            .filter(|m| {
                self.options.configurable.is_none()
                    || (self.options.configurable.clone().unwrap() == m.name && m.value == "true")
            })
            .map(|m| Param {
                name: m.name.clone(),
                value: m.value.clone(),
                kind: m.kind.clone(),
            })
            .collect::<Vec<Param>>()
    }

    pub fn to_toml(&self) -> String {
        self.params
            .clone()
            .iter()
            .map(|m| format!("{}='{}'", m.name, m.value.clone()))
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn to_string(&self, separator: Option<String>) -> String {
        let separator = separator.unwrap_or(String::from(" "));
        self.to_params()
            .clone()
            .iter()
            .map(|m| format!("--{}{}{}", m.name, separator, m.value))
            .collect::<Vec<String>>()
            .join(" ")
    }
}

// {
//                     if m.kind == InputType::TextArea {
//                         self.escape_str(&m.value)
//                     } else if self.options.quote_paths
//                         && (m.kind == InputType::File || m.kind == InputType::Dir)
//                     {
//                         self.escape_path(&m.value)
//                     } else if self.options.configurable.is_some()
//                         && self.options.configurable.clone().unwrap() == m.name
//                         && m.value == "true"
//                     {
//                       String::default()
//                     } else if m.kind == InputType::Checkbox || m.kind == InputType::Switch || m.kind == InputType::Toggle {
//                       if self.options.pass_boolean == false {
//                         String::default()
//                       } else {
//                         m.value.clone()
//                       }

//                     } else {
//                         m.value.clone()
//                     }
//                 }
