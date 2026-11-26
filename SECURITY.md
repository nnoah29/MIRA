# Security policy

## Reporting a vulnerability

Please **do not open a public issue** for a security problem.

Use GitHub's private reporting: on the repository page, open **Security → Report a vulnerability**. Include the version, your platform (Linux, Windows, Android), the steps to reproduce, and what an attacker gains.

You can expect an acknowledgement within a few days and a fix or a clear answer as soon as the problem is understood. Reporters are credited in the release notes unless they prefer to stay anonymous.

## What is in scope

- The mail engine, the local daemon and its protocol, the Flutter application, the packages.
- Anything that lets a received mail, a remote server or another local user read data, run code, send mail or contact the network without the user's action.

## Good to know

- Mail secrets (passwords, tokens) live in the system keyring.
- The local mail database is **not encrypted by MIRA**. This is a deliberate decision (2026-11-26): use full-disk encryption (LUKS, BitLocker). Encrypting the database with SQLCipher may be reconsidered later.
- The OAuth client identifiers embedded in the binary are not secret. Protection comes from PKCE.
- Remote images in mails are blocked until the user allows them.
- The AI assistant only proposes actions; nothing is sent or deleted without a click.

## Known limitations

- **Windows, local daemon access.** On Linux the daemon's token file and socket are restricted to your account. On Windows the daemon listens on `127.0.0.1` and relies on the token alone; the token file's permissions are not yet restricted explicitly. The default profile folder is already private to your account, so the risk is mainly a shared PC or a profile placed in a shared folder. A fix (explicit ACLs, then a named pipe) is planned and waits for testing on a Windows machine with two accounts.

## Supported versions

Only the latest release receives security fixes.
