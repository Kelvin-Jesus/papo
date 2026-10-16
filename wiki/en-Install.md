# Install

[[Português|Instalar]] · English

Both people who will talk need `papo` installed. Details are in the
[installation docs](https://kelvin-jesus.github.io/papo/docs/instalacao.html) (in Portuguese).

## Prebuilt binary

Download the file for your system from
[Releases](https://github.com/Kelvin-Jesus/papo/releases/latest):

| System              | File                                              |
| ------------------- | ------------------------------------------------- |
| Linux x86_64        | `papo-<version>-x86_64-unknown-linux-musl.tar.gz`  |
| Linux ARM64         | `papo-<version>-aarch64-unknown-linux-musl.tar.gz` |
| macOS Apple Silicon | `papo-<version>-aarch64-apple-darwin.tar.gz`       |
| macOS Intel         | `papo-<version>-x86_64-apple-darwin.tar.gz`        |
| Windows             | `papo-<version>-x86_64-pc-windows-msvc.zip`        |

Extract it and put `papo` (or `papo.exe`) in a directory on your `PATH`. Each file has a `.sha256`
next to it so you can check its integrity.

**macOS**: if the system blocks the binary, run `xattr -d com.apple.quarantine /path/to/papo`.

**Windows on ARM**: use the x86_64 binary, which runs under emulation.

## Building from source

With Rust 1.91+:

```sh
cargo install --git https://github.com/Kelvin-Jesus/papo
```

## Robots mod (optional)

Since v0.3.0 there is the `papo-robos` mod. It draws a pixel robot for each agent in the room above
the Claude Code prompt: it types while the agent is working and shows a green check when the turn
ends. In Claude Code:

```text
/plugin install papo-robos --marketplace Kelvin-Jesus/papo
```

To see your colleague working, both sides need `papo` v0.3.0 and the mod. Details are in the
[Robôs guide](https://kelvin-jesus.github.io/papo/docs/guias/robos.html) (in Portuguese).

## Checking

```sh
papo --version
```

Next step: [[Getting Started|en-Getting-Started]].
