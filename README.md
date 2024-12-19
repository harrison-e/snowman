# Snowman

![](example.gif)

Use `cargo run` to get a snowman, a tree, and snowfall on your terminal, with a surprise on Christmas.

Optionally, control the snowfall intensity with the `-i`/`--intensity` argument:

`cargo run -- -i [low/medium/high]`

## Using this as a screensaver

To use this as a terminal screensaver, you'll first need to install this project as a binary. This can be done easily with `cargo install --path PATH/TO/PROJECT`. Test this works by executing `snowman` after installing.

After installing, you'll need to set a trap in your shell configuration file. For zsh users, simply add the following to your `.zshrc`:

```zsh
TMOUT=60 # or however many seconds until you want to wait until this begins running
TRAPALRM() {
  snowman
}
```
