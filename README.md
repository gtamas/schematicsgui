# Schematics UI

<!-- vscode-markdown-toc -->
* 1. [What is this project about?](#Whatisthisprojectabout)
* 2. [A word of warning](#Awordofwarning)
* 3. [What do you need to use this?](#Whatdoyouneedtousethis)
* 4. [Getting started](#Gettingstarted)
* 5. [Providing feedback, asking for help, reporting bugs](#Providingfeedbackaskingforhelpreportingbugs)

<!-- vscode-markdown-toc-config
	numbering=true
	autoSave=true
	/vscode-markdown-toc-config -->
<!-- /vscode-markdown-toc -->


##  1. <a name='Whatisthisprojectabout'></a>What is this project about?

This app aims to be a GUI generator for any software based on Google's [Schematics](https://angular.io/guide/schematics) library.
Schematics provides only CLI support by default. Schematics UI can automatically generate a fully native and easy to use [GKT4](https://www.gtk.org/) GUI for any schematic. This GUI is entirely based on the JSON schema of the schematic, so it should work with any package out there.
The generated GUI is highly customizable and this requires only some JSON schema extension, so the CLI shouldn't be affected.     
Furthermore, the app also provides all the tools you need to work with schematics. You may browse contents of any schematics package, sync frequently used settings to disk creating different profiles for each schematic, reload, update or clone these profiles and of course you may also execute any schematic right from GUI.

It's a native app, written in Rust, uses [Relm4](https://relm4.org/) and [GTK-rs](https://gtk-rs.org/) rust bindings and it's gonna be available for OSX, Windows and Linux.

##  2. <a name='Awordofwarning'></a>A word of warning

This is still a work in progress. Although it should work fine with any schematics package, it was only tested with FND and NestJS so far. The app is not released yet and it's **NOT** production-ready. Currently, an alpha version is available. Use it at your own risk.

##  3. <a name='Whatdoyouneedtousethis'></a>What do you need to use this?

This is NOT a Schematics engine! It's just a GUI generator which can build UI for any schematic and execute that schematic using CLI. In order to use this, you need something like Google's [Schematics CLI](https://www.npmjs.com/package/@angular-devkit/schematics-cli) or another schematics implementation.

Once you've installed that, you are good to go.

##  4. <a name='Gettingstarted'></a>Getting started

First, you should install the app. There are to ways to do that: 

- Visit the [release page](https://github.com/gtamas/schematicsgui/releases) and grab a pre-built binary. This is the easiest and recommended way. Currently only an alpha version is available and only for OSX.
- Optionally, you can also build the app from source. This is the only way right now if you are not using a Mac. Please see the [build page](./BUILD.md) for more info.

Once you have the app installed, you can: 

- Check out the [Usage page](./USAGE.md) for general setup and usage instructions.
- See the [UI page](./UI.md) to learn more about the generated UI and the available customization options.


##  5. <a name='Providingfeedbackaskingforhelpreportingbugs'></a>Providing feedback, asking for help, reporting bugs

- Bugs and issues should be reported via our [issue tracker](https://github.com/gtamas/schematicsgui/issues)
- New features should be requested [here](https://github.com/gtamas/schematicsgui/issues/new?assignees=&labels=help+wanted&projects=&template=feature_request.md&title=%5BFEATURE%5D).
- Contact the author or ask for help by starting a [discussion](https://github.com/gtamas/schematicsgui/discussions).

Thanks and have fun!

