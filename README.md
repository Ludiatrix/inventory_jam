# A Bevy game template

Template for a Game using the awesome [Bevy engine][bevy] featuring out of the box builds for Windows, Linux, and Web (Wasm).

# What does this template give you?

* small example ["game"](https://niklasei.github.io/bevy_game_template/)
* easy setup for running the web build using [trunk] (`trunk serve`)
* run the native version with `cargo run`
* workflow for GitHub actions creating releases for Windows, Linux, and Web (Wasm) ready for distribution
    * push a tag in the form of `v[0-9]+.[0-9]+.[0-9]+*` (e.g. `v1.1.42`) to trigger the flow
* CI workflow that checks your application on Windows and Linux on every push

# How to use this template?

 1. Click "Use this template" on the repository's page
 2. Look for `ToDo` to use your own game name everywhere
 3. [Update the icons as described below](#updating-the-icons)
 4. Start coding :tada:
    * Start the native app: `cargo run`
    * Start the web build: `trunk serve`
        * requires [trunk]: `cargo install --locked trunk`
        * requires `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
        * this will serve your app on `8080` and automatically rebuild + reload it after code changes

### Updating the icons

 1. Replace `build/windows/icon.ico` (used for windows executable and as favicon for the web-builds)
    * You can create an `.ico` file for windows by following these steps:
       1. Open your icon image in [Gimp](https://www.gimp.org/downloads/)
       2. Select the `File > Export As` menu item.
       3. Change the file extension to `.ico` (or click `Select File Type (By Extension)` and select `Microsoft Windows Icon`)
       4. Save as `build/windows/icon.ico`

### Deploy web build to GitHub pages

 1. Trigger the `deploy-github-page` workflow
 2. Activate [GitHub pages](https://pages.github.com/) for your repository
     1. Source from the `gh-pages` branch (created by the just executed action)
 3. After a few minutes your game is live at `http://username.github.io/repository`

To deploy newer versions, just run the `deploy-github-page` workflow again.

# Getting started with Bevy

You should check out the Bevy website for [links to resources][bevy-learn] and the [Bevy Cheat Book] for a bunch of helpful documentation and examples. I can also recommend the [official Bevy Discord server][bevy-discord] for keeping up to date with the development and getting help from other Bevy users.

# Known issues

Audio in web-builds can have issues in some browsers. This seems to be a general performance issue and not due to the audio itself (see [bevy_kira_audio/#9][firefox-sound-issue]).

[bevy]: https://bevyengine.org/
[bevy-learn]: https://bevyengine.org/learn/
[bevy-discord]: https://discord.gg/bevy
[firefox-sound-issue]: https://github.com/NiklasEi/bevy_kira_audio/issues/9
[Bevy Cheat Book]: https://bevy-cheatbook.github.io/introduction.html
[trunk]: https://github.com/trunk-rs/trunk
