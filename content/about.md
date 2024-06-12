---
layout: page
title: About
permalink: /about/
---

## About Me

I am Ryan Heywood, a software developer with an interest towards security and
cryptography. I've contributed to projects in the past, such as
[Hashbang][hashbang] ([GitHub][hashbang-github]) and [ALE]. My biggest recent
project has been [Keyfork], a key management and disaster recovery toolkit
written in Rust, a language I picked up by writing my blog (using [Axum]).

## Some of my Favorite Projects

### [Keyfork]

The use of BIP32 seeds and BIP39 mnemonics has become quite common in the
cryptography world and I wanted to see if I could use deterministic derivation
to generate secrets I use elsewhere, such as an OpenPGP keychain. Keyfork is a
tool that I built with Distrust for deterministic generation of cryptographic
keys.

### [Shell Server]

A lot of the work I contributed to Hashbang has been through the shell server,
a system designed to integrate with an external authentication database, built
resilient enough that we can trust that giving users the ability to run code on
our hardware wouldn't cause a significant impact on other users. The project is
an overall success, having spent years in a production status with hundreds of
users and minimal compromise, and has given me an incredible knowledge of how
Linux systems and what goes into making containers work.

### [FusionScript]

This was an experiment to see how powerful the Lua language could act as a VM
for a language with higher level constructs, like first class generators and
async operations. While the project as a whole is mostly abandoned, I learned a
lot while writing it and generally consider my time spent working on the
project to be incredibly valuable.

## Contacting Me

I can be reached through the following methods, in order of how likely you are
to reach me:

* Matrix: [@ryansquared:beeper.com](matrix://u/ryansquared:beeper.com)
* IRC: ryan:irc.hashbang.sh
* Mastodon: [@ryan@tilde.town](https://tilde.town/@ryan)

If you need to reach me in a secure manner, you can do so using PGP. My PGP key
key is [openpgp4fpr:88823A75ECAA786B0FF38B148E401478A3FBEF72][keyoxide] and is
linked through most of my social media through the keyoxide link.

## Licensing Information

Unless otherwise stated, the textual content of this website is licensed under
[CC BY-NC-SA 4.0][by-nc-sa-4.0]. Unless otherwise stated, all code snippets on
this website have been released to the public domain.

[keyoxide]: https://keyoxide.org/hkp/88823A75ECAA786B0FF38B148E401478A3FBEF72
[hashbang]: https://hashbang.sh
[hashbang-github]: https://github.com/hashbang
[ALE]: https://github.com/dense-analysis/ale
[Axum]: https://github.com/tokio-rs/axum
[Keyfork]: https://git.distrust.co/public/keyfork
[Shell Server]: https://github.com/hashbang/shell-server
[FusionScript]: https://github.com/RyanSquared/FusionScript
[by-nc-sa-4.0]: http://creativecommons.org/licenses/by-nc-sa/4.0/?ref=chooser-v1
