<!-- vscode-markdown-toc -->
* 1. [Default UI](#DefaultUI)
* 2. [The x-widget property](#Thex-widgetproperty)
	* 2.1. [text](#text)
		* 2.1.1. [More about multiline](#Moreaboutmultiline)
		* 2.1.2. [A note on icons](#Anoteonicons)
	* 2.2. [numeric](#numeric)
		* 2.2.1. [About marks](#Aboutmarks)
	* 2.3. [date](#date)
	* 2.4. [color](#color)
	* 2.5. [dir and file](#dirandfile)
	* 2.6. [choice](#choice)
	* 2.7. [menu](#menu)
	* 2.8. [About items](#Aboutitems)
		* 2.8.1. [Items special syntax](#Itemsspecialsyntax)

<!-- vscode-markdown-toc-config
	numbering=true
	autoSave=true
	/vscode-markdown-toc-config -->
<!-- /vscode-markdown-toc --># About the UI



##  1. <a name='DefaultUI'></a>Default UI

Schematics UI comes with a default widget renderer which generates a GTK UI automatically, based on the `schema.json` of any schematic. The builder will look at the properties of the schema and will build a UI using the following mapping:


|Schema prop|Widget
-------------|-------
x-prompt or description |[Label](https://docs.gtk.org/gtk4/class.Label.html)
type="string" |[Entry](https://docs.gtk.org/gtk4/class.Entry.html)
type="string" && multiline=true |[TextView](https://docs.gtk.org/gtk4/class.TextView.html)
type="boolean"| [Switch](https://docs.gtk.org/gtk4/class.Switch.html)
type="number"| [SpinButton](https://docs.gtk.org/gtk4/class.SpinButton.html)
type="string" && format="date" |[Calendar](https://docs.gtk.org/gtk4/class.Calendar.html)
type="string" && format="time" |Time input
type="string" && format="date-time" |Date & Time input
type="string" & enum |[Dropdown](https://docs.gtk.org/gtk4/class.DropDown.html)
type="array" & x-prompt.items |[Dropdown](https://docs.gtk.org/gtk4/class.DropDown.html)
enum or x-prompt.items and multiselect="true" | ListView
type="string" && format="path" |[File chooser](https://docs.gtk.org/gtk4/class.FileChooserDialog.html)

Additionally, a `submit` button will be added.

##  2. <a name='Thex-widgetproperty'></a>The x-widget property

If you don't like the default UI, you customize it or even override it. This requires adding the `x-widget` property to your schema properties which must be located on the same level as `x-prompt`. 

Let's see an example:

```json
  "properties": {
    "name": {
      "type": "string",
      "minLength": 5,
      "description": "Name of the component",
      "x-prompt": "Name of the component"
    },
    "type": {
      "type": "string",
      "enum": ["component", "module"],
      "description": "Type of react component",
      "default": "component",
      "x-prompt": "Select component type",
      "x-widget": {
        "menu": {
          "type": "radio"
        }
      }
    },
  }
```

Here, the first property is `name` which does not have any customization so it will be rendered as a text input, using the default widget builder. The second `type` property, however, has `x-widget`. In this case, the type is set to `radio`, so instead of the default dropdown menu, a group of radio buttons will be generated.

This is just a basic example. You can customize your inputs in many different ways via `x-widget`. Via such customizations, you can use most of the input widgets available in the [GTK4 Widget Library](https://docs.gtk.org/gtk4/visual_index.html) and even some custom one shipped with the app. More widgets will be added in the future.

The general form of the `x-widget` is the following:

```json
"x-widget": {
  "<type>": {
    // widgetOtions
  }
}
```

Below is a list of available widget types and the options they support:

###  2.1. <a name='text'></a>text

This renders as single or multiline text widget. 
Most of these only work with single line input right now, while `justify` and `height` are only applicable to textarea.

|Options|Type|Default|Description
--------|----|-------|------------
max_len | integer|| The user cannot type more than this number of characters 
height | integer || The height of the widget in pixels. Only works when multiline=true
placeholder | string | "" |This text will appear in the widget by default and will be removed as soon as the user starts typing
icon | string | "" | name of a icon to decorate the input with. See below
icon_position | `start`, `end` | "end" | It's either "start" or "end", adding the icon as prefix or suffix, respectively.
tooltip | string | "" | The text of the tooltip display on hover
hint_text | string | "" | Short text printed below the field as help or hint.
multiline | bool | false | If true, a textarea will be printed allowing multiple line input. 
direction | `left`, `right` | "left" | Typing direction. Default to left to right.
overwrite | bool | false | If true, it's possible to overwrite text in the input
purpose | enum | none | Describes the primary purpose of the widget. For screen readers, virtual keyboards etc. See [this page](https://docs.gtk.org/gtk4/enum.InputPurpose.html)  
hint | enum | none | Describes how the input should be handled.  For screen readers, virtual keyboards etc. See [this page](https://docs.gtk.org/gtk4/flags.InputHints.html)
justify | `left`, `right`, `center` | "left" | Text alignment. Works only with textarea.

####  2.1.1. <a name='Moreaboutmultiline'></a>More about multiline

Single line text input is an [Entry](https://docs.gtk.org/gtk4/class.Entry.html) widget while multiline is implemented as an editable [TextView](https://docs.gtk.org/gtk4/class.TextView.html)

####  2.1.2. <a name='Anoteonicons'></a>A note on icons

You can use the name of any of the GNOME or Fluent UI System icons, all 2500 of them. See the following icon catalogs for details:

[GNOME Icons](https://teams.pages.gitlab.gnome.org/Design/icon-development-kit-www/)

[Fluent UI Icons](https://master--628d031b55e942004ac95df1.chromatic.com/?path=/docs/concepts-developer-icons-icons-catalog--page)

###  2.2. <a name='numeric'></a>numeric

This renders input for numbers.

| Options | Type | Default | Description |
| ------- | ---- | ------- | ----------- |
type | `input`, `slider` | "input" | The type of the input widget. See below.
value_type | `int`, `float` | "float" | The type of the expected input value.
stepping | number | 1.0 | Increase or decrease the value by this amount when tapping arrow button or moving the slider.
max | number | Max of type | The maximum value the input accepts. If omitted, it's the max value allowed by the OS for the given type (unsigned integer 64 bit floating point)
min | number | Min of type | The minimum value the input accepts. If omitted, it's the min value allowed by OS for the given type (unsigned integer 64 bit floating point)
initial_value | number | 0 | The initial value of the input
page_increment | number | 0 | Part of the number model GTK uses. See this page for details: [GTK adjustmet](https://docs.gtk.org/gtk4/class.Adjustment.html)
page_size | number | 0 | Part of the number model GTK uses. See this page for details: [GTK adjustmet](https://docs.gtk.org/gtk4/class.Adjustment.html)
orientation | `vertical`, `horizontal` | vertical | The visual orientation of the widget.
precision | number | 2 | The number of decimals. If `value_type` is `int`, it's always 0.
wrap | bool | false | If true, the spin button’s value wraps around to the opposite limit when the upper or lower limit of the range is exceeded. Only applies to `input` type.
show_current | `top`, `right`, `bottom`, `left` | bottom | Determines where the current value is shown for slider.
marks | Mark[] | [] | If defined, labels will be placed at specified values on the scale. See below. Also see GTK docs on [marks](https://docs.gtk.org/gtk4/method.Scale.add_mark.html). Works only with `slider` type.
slider_height | int | 100 | The height of the slider in pixels. Makes sense only for sliders and only when `orientation` is `vertical` 

The `type` maps to these widget types:

| Type | Widget |
| ---- | ------ |
input | [SpinButton](https://docs.gtk.org/gtk4/class.Scale.html)
slider |  [Scale](https://docs.gtk.org/gtk4/class.Scale.html)


####  2.2.1. <a name='Aboutmarks'></a>About marks

Marks are represented by objects that look like this:

```json
{
  "value": 1.23,
  "text": "label text" 
}
```

###  2.3. <a name='date'></a>date

This allows you create date, time and date-time input widgets.

| Options | Type | Default | Description |
| ------- | ---- | ------- | ----------- |
type | `date`, `time`, `date-time` | date | The type of the widget. See below
format | string | "" | This determines how the output should be formatted as string. See [this page](https://docs.gtk.org/glib/method.DateTime.format.html)


The `type` maps to these widget types:

| Type | Widget |
| ---- | ------ |
date |  [Calendar](https://docs.gtk.org/gtk4/class.Calendar.html)
time | Custom time selector, consisting of 3 [SpinButtons](https://docs.gtk.org/gtk4/class.SpinButton.html)
date_time | Combination or the previous two.


###  2.4. <a name='color'></a>color

This renders widgets dealing with color values.


| Options | Type | Default | Description |
| ------- | ---- | ------- | ----------- |
type | `input`, `button` | input | The type of the widget, see below.
format | `hex`, `rgb`, `hsl` | hex | Determines whether the output should be [hexadecimal](https://developer.mozilla.org/en-US/docs/Web/CSS/hex-color) (hex), [hue-saturation-lightness](https://developer.mozilla.org/en-US/docs/Web/CSS/color_value/hsl) (hsl) or [red-green-blue}(https://developer.mozilla.org/en-US/docs/Glossary/RGB). All these are CSS3 compliant color codes.
alpha | bool | false | Should the color value use an alpha (transparency) channel or not
title | string | "Choose a color" | Title of the color chooser dialog

The `type` maps to these widget types:

| Type | Widget |
| ---- | ------ |
input | An [entry](https://docs.gtk.org/gtk4/class.Entry.html) with a button triggering the [color chooser dialog](https://docs.gtk.org/gtk4/class.ColorChooserDialog.html)
button | [ColorButton](https://docs.gtk.org/gtk4/class.ColorButton.html) with no entry.


###  2.5. <a name='dirandfile'></a>dir and file

These create path selector inputs allowing you to choose files and directories.


| Options | Type | Default | Description |
| ------- | ---- | ------- | ----------- |
mask | string | "*" |A standard file mask, like `*.pdf`. If defined, only the entries matching the mask will be shown in the file selector.
is_new | bool | false | If true, a text input will be added to file selector allowing you to define the name of a new file. Only appears if `is_dir` is false. 
current_folder | string | "" | If defined, file selector will start browsing from this folder.
multiple | bool | false | If true, selection of multiple entries is allowed
default_name | string | "" | Sets the default name of the new file. Makes sense only if `is_new` is true.
is_dir | bool | false | If true, only directories will be selectable
title | string | "Choose a file" | Sets the title of the file selector dialog


###  2.6. <a name='choice'></a>choice

This generates input whose value is either true or false.

| Options | Type | Default | Description |
| ------- | ---- | ------- | ----------- |
type | `checkbox`, `switch`, `toggle` | checkbox | Type of the widget, see below
label | string | "" | Optional extra label for the input. Works only with `checkbox` or `toggle`.


The `type` maps to these widget types:

| Type | Widget |
| ---- | ------ |
checkbox | [CheckButton](https://docs.gtk.org/gtk4/class.CheckButton.html)
switch | [Switch](https://docs.gtk.org/gtk4/class.Switch.html)
toggle | [ToggleButton](https://docs.gtk.org/gtk4/class.ToggleButton.html)

###  2.7. <a name='menu'></a>menu

This allows you to create inputs where the user needs to select one or more items from a pool of pre-defined options. 


|Options|Type|Default|Description
--------|----|-------|------------
type    |`dropdown, combobox, multiselect, radio, toggle` | `dropdown` |  The type of the widget, see below.
searchable | bool | false | If true, the menu items will be filterable via text search. Only combobox for now.
orientation | `vertical, horizontal` | `vertical` | widget orientation

The `type` maps to these widget types:

|Type|Widget|
-----|-------
dropdown | [DropDown](https://docs.gtk.org/gtk4/class.DropDown.html)
combobox | [Combobox](https://docs.gtk.org/gtk4/class.ComboBox.html)
radio | [CheckButton](https://docs.gtk.org/gtk4/class.CheckButton.html)
toggle | [ToggleButton](https://docs.gtk.org/gtk4/class.ToggleButton.html)
multiselect | ListView

All these are single selection widgets, except for `multiselect`, obviously. In case of `radio` and `toggle` you get a group with a button for each selectable option.

###  2.8. <a name='Aboutitems'></a>About items

The selectable items must be pre-defined in your schema. All syntaxes supported by Schematics should work.

You can define your items in one of the following ways:

As enum:

```json
"tool": {
  "type": "string",
  "enum": ["yarn", "npm"],
  "default": "yarn",
  "description": "Package Manager of the project",
  "x-prompt": "Please choose Package Manager"
}
```

As list, using an array of strings:

```json
"model": {
  "type": "string",
  "minLength": 1,
  "description": "Name of the resolver model",
  "x-prompt": {
    "message": "Choose the model",
    "type": "list",
    "items": [
      "one",
      "two"
    ]
  }
},
```


As list, using an array of objects:

```json
"model": {
  "type": "string",
  "minLength": 1,
  "description": "Name of the resolver model",
  "x-prompt": {
    "message": "Choose the model",
    "type": "list",
    "items": [
      {
        "label": "some text",
        "value": "some value"
      },
       {
        "label": "other text",
        "value": "other value"
      }
    ]
  }
},
```

####  2.8.1. <a name='Itemsspecialsyntax'></a>Items special syntax 

Besides the syntax supported by schematics engines, some special directives regarding items are also supported.
The value of the `x-prompt.items` property can be one of the following:

| Directive | Description |
| ---- | ------ |
| $modules | Items will be populated using the names of all subdirectories in current working directory.
| $models | Items will be populated using the absolute path of all files matching the following pattern `src/**/models/*.model.ts`.
| $dir:/some/path | Items will be populated using the names of all subdirectories under `/some/path`.
| $files:/some/path | Items will be populated using the absolute path of all files under `/some/path`.

Let's see some examples:


```json
 "env": {
      "type": "string",
      "description": "The env name",
      "x-prompt": {
        "message": "What is the name of the env?",
        "type": "list",
        "items": "$modules"
      }
    },
```


```json
 "env": {
      "type": "string",
      "description": "The env name",
      "x-prompt": {
        "message": "What is the name of the env?",
        "type": "list",
        "items": "$dir:/foobar"
      }
    },
```