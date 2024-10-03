# sparrow

[![CI](https://github.com/dagregi/sparrow/workflows/CI/badge.svg)](https://github.com/dagregi/sparrow/actions)

TUI for transmission remote

## Installation

### Building from source

Dependancies:

-   rustc
-   cargo

```bash
cargo build --release
```

## Usage

Make sure the transmission daemon is running

```bash
transmission-daemon
```

Start sparrow

```bash
sparrow
```

### Options

-   -u, --url

    Specifiy the RPC url

```bash
sparrow -u http://192.168.41:9000/transmission/rpc
```

-   --username, --password

    For authentication

```bash
sparrow --username "user" --password "very_secret_password"
```

-   -h, --help

    Print help

-   -V, --version

    Print version

### Keybindings

-   Home

| Key          | Description            |
| :----------- | :--------------------- |
| `j`          | Move down              |
| `k`          | Move up                |
| `l`, `enter` | Show info              |
| `g`          | Goto top               |
| `G`          | Goto bottom            |
| `p`          | Start/stop torrent     |
| `s`          | Start all torrents     |
| `S`          | Stop all torrents      |
| `q`          | Quit                   |
| `Q`          | Quit and close session |

-   Info

| Key   | Description            |
| :---- | :--------------------- |
| `l`   | Next tab               |
| `h`   | Previous tab           |
| `Esc` | Go back                |
| `q`   | Quit                   |
| `Q`   | Quit and close session |

## TODO

-   [x] Add a component to show torrent information
-   [ ] Filter/Search for torrents in the list
-   [ ] Add a help modal/page to show keybindings
-   [ ] File viewer for the torrents
-   [ ] Method to change the priority of torrents
-   [ ] A way for adding torrents
-   [ ] A way to manage config files and keymaps

## Credits

[ratatui](https://github.com/ratatui/ratatui) for the amazing TUI library.

## Contributing

For any feature requests, ideas for improvement or bugs, please open an issue.

If you'd like to contribute, fork the repository and open a pull request.

## License

Licensed under [MIT](https://github.com/dagregi/sparrow/raw/main/LICENSE)
