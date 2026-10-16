# Roadmap

[[Português|Roadmap]] · English

Ideas for the future. **Nothing on this page is implemented**; it is a list of possible directions,
open for discussion in the [issues](https://github.com/Kelvin-Jesus/papo/issues).

## Distribution

- Packages on Homebrew (macOS/Linux), Scoop and winget (Windows).
- Signed releases (for example, code signing on macOS and Windows, and verifiable signatures on the
  files).
- Packaging papo itself as a Claude Code plugin, to install it and enable channels without the
  development flag.
- A one-line install script.

## Security

- Per-member keys and a signature on each message, so one member cannot pose as another.
- Removing members without creating a new room (secret rotation).

## Delivery

- Guaranteed delivery to every member in rooms with more than two people (today, a message without
  `to` leaves the queue at the first confirmed receipt).

## Documentation

- An English version of the full documentation. The wiki already has English pages.
