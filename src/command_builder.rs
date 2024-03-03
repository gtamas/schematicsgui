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

#[derive(Default, Debug, Clone, PartialEq)]
pub enum InputType {
    #[default]
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
            option_case: Case::Kebab,
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
            return format!("`{}`", value.replace(['\r', '\n'], "\\\n"));
        }

        value.replace(['\r', '\n'], " ")
    }

    fn escape_path(&self, path: &str) -> String {
        format!("\"{}\"", path.replace(['"'], "\\\""))
    }

    fn get_param_name(&self, param: &Param) -> String {
        if !self.options.pass_boolean {
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
                if !self.options.pass_boolean {
                    return String::default();
                }
                param.value.clone()
            }
            InputType::TextArea => self.escape_str(&param.value),
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
                param.value.clone()
            }
        }
    }

    pub fn set_configurable(&mut self, value: String) {
        self.options.configurable = Some(value);
    }

    pub fn get_configurable(&self) -> String {
        self.options.configurable.clone().unwrap_or_default()
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
                    || (self.options.configurable.clone().unwrap() == m.name)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn get_param(name: &str, value: Option<&str>, kind: Option<InputType>) -> Param {
        Param {
            name: name.to_string(),
            value: value.unwrap_or_default().to_string(),
            kind: kind.unwrap_or_default(),
        }
    }

    fn get_params() -> Vec<Param> {
        vec![
            get_param("foo", Some("1"), None),
            get_param("bar", Some("foo"), None),
        ]
    }

    #[test]
    fn escape_str_single() {
        let builder = CommandBuilder::new(None);
        let raw = "foo\n\rbar";
        let escaped = builder.escape_str(raw);

        assert_eq!(escaped, "foo  bar")
    }

    #[test]
    fn escape_str_multi() {
        let builder = CommandBuilder::new(Some(CommandBuilderOptions {
            escape_multiline_text: true,
            ..Default::default()
        }));
        let raw = "foo
        bar
        baz";
        let escaped = builder.escape_str(raw);

        assert_eq!(
            escaped,
            "`foo\\
        bar\\
        baz`"
        )
    }

    #[test]
    fn escape_path() {
        let builder = CommandBuilder::new(None);
        let path = "/foo/bar/some/\"foo bar\"";
        let escaped = builder.escape_path(path);

        assert_eq!(escaped, "\"/foo/bar/some/\\\"foo bar\\\"\"")
    }

    #[test]
    fn get_param_name_no_pass_boolean_value_true() {
        let builder = CommandBuilder::new(Some(CommandBuilderOptions {
            pass_boolean: false,
            ..Default::default()
        }));

        let param = get_param("foo_bar", Some("true"), None);

        let name = builder.get_param_name(&param);

        assert_eq!(name, "fooBar")
    }

    #[test]
    fn get_param_name_no_pass_boolean_value_false() {
        let builder = CommandBuilder::new(Some(CommandBuilderOptions {
            pass_boolean: false,
            ..Default::default()
        }));

        let param = get_param("foo_bar", Some("false"), None);

        let name = builder.get_param_name(&param);

        assert_eq!(name, "noFooBar")
    }

    #[test]
    fn get_param_name_pass_boolean_value_any() {
        let builder: CommandBuilder = CommandBuilder::new(None);

        let param = get_param("foo_bar", None, None);

        let name = builder.get_param_name(&param);

        assert_eq!(name, "fooBar")
    }

    #[test]
    fn set_configurable() {
        let mut builder: CommandBuilder = CommandBuilder::new(None);

        builder.set_configurable("some".to_string());

        assert_eq!(builder.options.configurable.unwrap(), "some".to_string())
    }

    #[test]
    fn set_executable() {
        let mut builder: CommandBuilder = CommandBuilder::new(None);
        let executable = "some".to_string();

        builder.set_executable(executable.clone());

        assert_eq!(builder.executable, executable)
    }

    #[test]
    fn get_executable() {
        let mut builder: CommandBuilder = CommandBuilder::new(None);
        let executable = "some".to_string();

        builder.set_executable(executable.clone());

        assert_eq!(builder.get_executable(), executable)
    }

    #[test]
    fn set_command() {
        let mut builder: CommandBuilder = CommandBuilder::new(None);
        let command = "some".to_string();

        builder.set_command(command.clone());

        assert_eq!(builder.command, command)
    }

    #[test]
    fn get_command() {
        let mut builder: CommandBuilder = CommandBuilder::new(None);
        let command = "some".to_string();

        builder.set_command(command.clone());

        assert_eq!(builder.get_command(), command)
    }

    #[test]
    fn set_params() {
        let mut builder: CommandBuilder = CommandBuilder::new(None);
        let param = Param::default();
        let params = vec![param.clone()];

        builder.set_params(params.clone());

        assert_eq!(builder.params, params);
    }

    #[test]
    fn add() {
        let mut builder: CommandBuilder = CommandBuilder::new(None);
        let param = Param::default();

        builder.add(param.clone());

        assert_eq!(builder.params.len(), 1);
        assert_eq!(builder.params[0], param)
    }

    #[test]
    fn to_toml() {
        let mut builder: CommandBuilder = CommandBuilder::new(None);
        let params = get_params();

        builder.set_params(params);

        assert_eq!(builder.to_toml(), "foo='1'\nbar='foo'");
    }

    #[test]
    fn to_string_no_separator() {
        let mut builder: CommandBuilder = CommandBuilder::new(None);
        let params = get_params();

        builder.set_params(params);

        assert_eq!(builder.to_string(None), "--foo 1 --bar foo");
    }

    #[test]
    fn to_string_separator() {
        let mut builder: CommandBuilder = CommandBuilder::new(None);
        let params = get_params();

        builder.set_params(params);

        assert_eq!(
            builder.to_string(Some(":".to_string())),
            "--foo:1 --bar:foo"
        );
    }

    #[test]
    fn to_params_no_configurable() {
        let mut builder: CommandBuilder = CommandBuilder::new(None);
        let params = get_params();

        builder.set_params(params.clone());

        assert_eq!(builder.to_params(), params);
    }

    #[test]
    fn to_params_configurable() {
        let mut builder: CommandBuilder = CommandBuilder::new(Some(CommandBuilderOptions {
            configurable: Some("config".to_string()),
            ..Default::default()
        }));
        let mut params = get_params();
        params.push(Param {
            name: "config".to_string(),
            value: "true".to_string(),
            ..Default::default()
        });

        builder.set_params(params.clone());
        let result = builder.to_params();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, params[2].name);
    }
}
