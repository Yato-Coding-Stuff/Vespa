# Vespa

A Cross-Platform, CLI, Hollow Knight: Franchise ModInstaller.

> [!WARNING]
> This is (at the moment) a purely command line based application.
> If you are not familiar or comfortable with the usage of such tools
> I would **strongly** recommend to check out [Lumafly](https://github.com/...) for HK and [Cogfly](https:/github.com/...) for SK instead.

Inspired by [Colophon](https://github.com/dpinela/colophon).
Shoutout to them, check out their work :D

People familiar with UNIX package management will find this program to be very intuitive,
as the commands and implementation are modeled specifically after these tools.

Written entirely in rust.
Blazingly fast, yada yada yknow how it goes

## Usage rundown

This is a quick rundown of how to use vespa.
For a full run-down of all available commands, run `vespa --help` or check out the [Commands](#commands) section.

### to install your first mod

```bash
$ vespa profile create profile_name
$ vespa install mod_name # Select the correct one should there be multiple options available
```

### disabling and enabling mods

Installed mods are automatically enabled by default, should you want to disable them,
use `vespa disable mod_name`.
For more details on this see [Disable](#disable) and [Enable](#enable).

### running the game

The `vespa run` command will copy the files to the game directory and run the game.
When the game is closed, the files will be removed again.

Should you want to run the game without having to use `vespa run`,
prime the currently defaulted profile with `vespa prime`.
This will copy the files to the game directory and keep them there until `vespa unprime` is called,
allowing you to run the game through whatever means you want.
For more info see [Prime](#prime) and [Unprime](#unprime).

### Configuration

// TODO:
// add other platforms
The config file can be found at the follow locations:
Linux:

- `$HOME/.config/vespa/config.toml`

It contains the following configurable settings:

- `game_switcher` - The game to use by default. [!NOTE] This should be set via the `vespa set-game game_name` command. Which is not yet implemented...
- `default_profile` - The profile to use by default. [!NOTE] This should be set via the `vespa profile set-default profile_name` command.
- `hollow_knight_path` - The path to the Hollow Knight Game directory.
- `silk_song_path` - The path to the Silk Song Game directory.
- `index_path` - The path where the index file is stored. [!WARNING] This should not be changed unless you have a very good reason to.

## Commands

### Game

The `--game` flag can be used to specify what game to manage mods for.
It can be appended to every single command.

Should you be using vespa for only one game, you can set the default game using the `game_switcher` setting in the config file or via the `vespa set-game game_name` command, which is not yet implemented...

### Profile

#### List

`vespa profile list` will list all created profiles for whatever --game is specified.

#### Create

`vespa profile create [profile_name]` will create a new profile with the specified profile name.
This profile will be automatically set as the default profile when either the `--set-default` flag is used or no default profile has been set yet.

#### Delete

`vespa profile delete [profile_name]` will delete the specified profile.

#### Set default

`vespa profile set-default [profile_name]` will set the specified profile as the default profile.

### Install

`vespa install [mod_name/s]` will install the specified mod/s into the currently defaulted profile.
For every specified mod, if a direct match is found, you will directly be asked for confirmation.
If no direct match is found, you will be asked to select the correct mod to install first out of a list of the best 3 matches.

### InstallLocal

`vespa install-local [mod_path]` will install the local mod at the specified path into the currently defaulted profile.
Currently only supports .zip files.

### Uninstall

`vespa uninstall [mod_name/s]` will uninstall the specified mod/s from the currently defaulted profile.
For every specified mod, if a direct match is found, you will directly be asked for confirmation.
If no direct match is found, you will be asked to select the correct mod to uninstall first out of a list of the best 3 matches.

This commands supports the flag `--force`, which will uninstall the mod even if other packages depend on it.

### Update

`vespa update` can be run either with `[mod_name/s]` or without.
If no mod names are specified, all installed mods will be updated.

### Disable

`vespa disable [mod_name/s]` will disable the specified mod/s from the currently defaulted profile.
This is achieved by renaming the mod to include a `.disabled` suffix.

### Enable

`vespa enable [mod_name/s]` will enable the specified mod/s from the currently defaulted profile.
This is achieved by removing the `.disabled` suffix from the mod name.

### List

`vespa list` will by default list all installed mods for the currently defaulted profile.

It also supports the following flags:

- `[mod_name/s]` - Filters for the specified mod/s.
- `--available` - Lists all mods available for download instead of installed mods.
- `all_versions` - Lists all versions of the specified mod/s.

### Show

`vespa show [mod_name]` will show further information about the specified mod.
This includes the following information:

- name
- latest version
- description
- download url
- dependencies

### Run

`vespa run` will run the game using the currently defaulted profile.
It will prime the profile, run the game, and on exit unprime the profile.
See [Prime](#prime) and [Unprime](#unprime) for more info.

### Prime

`vespa prime` will copy the files currently contained in the defaulted profile to the selected games directory and keep them there until `vespa unprime` is called.
To achieve this a transaction file is created under `$HOME/.config/vespa/runtime/[game_name].json`.
This transaction file will be used to restore the files when `vespa unprime` is called or the program encounters some kind of failure state.

Should the files already exist( as is the case for HK's modding api structure),
these files will first be backed up under `.vespa-backup/`

### Unprime

`vespa unprime` will restore the files from the transaction file created by `vespa prime`.
