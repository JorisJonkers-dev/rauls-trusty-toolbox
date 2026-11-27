---
status: accepted
date: 2026-11-27
---
# The Nexus API key lives in the OS keyring and never on a command line

Nexus Mods identifies a player's account by a personal API key. The key can read the account name and, for a premium
account, ask for download links, so it is a secret. `raul` needs it to look up mods and files and, later, to download.

**Where it is kept.** In order of use: the `NEXUS_API_KEY` environment variable (CI and headless machines); the
operating system's keyring through the `keyring` crate (Secret Service on Linux, Credential Manager on Windows,
Keychain on macOS); and, only when the person runs `raul nexus key set --file`, a file `nexus-api-key` in the
configuration folder with mode 0600. `settings.toml` never holds the key: it may be pasted into a bug report. It holds
only the account name, whether it is premium and when the key was last checked (`[nexus]`).

**How it travels.** `raul nexus key set` reads the key from standard input, never from an argument (argv is readable by
every process of the machine). Requests go through `curl` by the Runner (ADR-0002); the address and every header,
the key included, are handed to `curl` as a config file on its standard input (`--config -`), so the command line holds
fixed flags only. Redirects are not followed, so the key cannot be sent to another host. The key type has no `Display`
and a `Debug` that hides it, and every error text passes through a redaction that removes the key and the value of any
`apikey` header or pair.

**In the window.** Settings has a masked field with Save, Clear and Check. The job runs in process through
`raul_core::nexus` (ADR-0016): the typed key moves into the job as a `Secret`, never into an argument, an event or the
job table, and the page only receives where the key is kept, the account name, premium and the time of the last check.
Without a keyring, Save keeps the key in the 0600 key file and the page says so; `NEXUS_API_KEY` still comes first.

**The client.** `raul_core::nexus::NexusApi` is the seam: validate, mod info, file list, md5 lookup and download link.
`NexusClient` implements it over a `Transport`; tests use a fake transport and synthetic JSON, and nothing in CI
reaches Nexus. Every request sends `Application-Name: raul` and `Application-Version`. A download link is returned
only to an account Nexus allows it for; any other answer is `NeedsPremiumOrManual`, which names the file's page. Signed
links are short-lived, are never stored and are not printed by `Debug`.

## Consequences

- On Linux the keyring needs a running Secret Service (GNOME Keyring, KWallet, KeePassXC); without one `key set` says so
  and offers `--file` or the environment variable. On Windows the file fallback relies on the person's own `%APPDATA%`
  folder, not an ACL raul sets. macOS may ask the person to allow access to the Keychain entry.
- Nexus's rules for apps (the API acceptable use policy) still decide whether a personal key may be used in a public
  release; this decision is about handling the key, not about that permission.
- The run log of nvmod redacts by pattern in its own crate (`nvmod-runlog`); `raul` does not put the key in any nvmod
  call, and its own redaction is in `raul_core::nexus::redact`. Adding the `apikey` pattern to `nvmod-runlog` is a
  Component change tracked apart.
