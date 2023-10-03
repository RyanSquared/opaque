---
layout: post
title: Behind Locked Doors
tags: qubes, gnupg, openpgp, ssh, webauthn, security
last-edited: 2022-10-02 01:40 -0400
date: 2022-09-20T19:41:00-0400
---
**Foreword:** This blog post will likely change as time goes on and I learn
more about Qubes and how I can adapt my home-built SSH solution to be on par
with the provided SSH solution. I hope to keep the structure of the post
mostly consistent.

![A locked chamber, imposing, cyberpunk, vaporwave](/assets/images/2022/09/behind-locked-doors-hero.png)

Image generated from Stable Diffusion: A locked chamber, imposing, cyberpunk, vaporwave

Recently, I've started using an operating system at work called [QubesOS]. The
operating system is pretty fancy, it gives me a way to separate all of my
applications into little environment-specific containers. It's very useful as a
contractor, as I can have many clients at once. With the default 4 workspaces
that Qubes provides me, that gives me the ability to move between my personal
Qube, my Distrust Qube, and a couple Qubes I have set up for clients. It offers
me an excellent way to make sure that both the visual component - the programs
that I'm using, whether it be Firefox, Thunderbird, or a terminal - and the
data components, such as the actual project files themselves, are all
self-contained.

However, each of these clients, in one way or another, needs me to be able to
interact with them cryptographically in a way that I can trust. This can be
unlocking a password database, signing a GitHub commit, connecting to a server
via SSH, or logging into a service that incorporates a WebAuthn authentication
flow. I need a way to be able to take all four of these qubes and hook them
into a central point of authority that I'm able to trust with access to
cryptographic operations involving my private key.

# The Vault

QubesOS by default comes with a qube called the "vault". This is a component
whose compromise would come second only to dom0. It is the centralized
authority for all of my sensitive cryptographic operations. If I want to do
anything that requires access to PGP, SSH, or WebAuthn, it would be run through
this component first. For my personal system, it's based off the Debian 11
template, so while most of these commands should have a Fedora alternative (or
otherwise, if Qubes starts offering new TemplateVMs), the instructions I give
are targeted towards a Debian system.

The first thing I wanted to do was to make sure that my smartcard could
properly attach to the Vault. I used the [sys-usb] setup to connect my
smartcard, a [Yubikey], to the qube. I was able to confirm that the device was
visible from inside the qube by running `lsusb`. From there, since `gpg2` and
`gpg-agent` were installed by default, I was able to run `gpg --card-status` to
confirm that the 

**Note:** At this point, if the `gpg --card-status` command fails, you should
stop what you're doing and make sure that `gpg2` is installed in your vault
qube and that the `gpg-agent` systemd user service has been started.

The Qubes team has created a tool called [qubes-gpg-client] and
`qubes-gpg-client-wrapper` which allows an almost seamless replacement for
`gpg` and `gpg2` for programs such as Git, Thunderbird, and otherwise. It is
close enough to the point where I personally symlink `/usr/local/bin/gpg{,2}`
to `/usr/bin/qubes-gpg-client-wrapper`. I also make sure to put `export
QUBES_GPG_DOMAIN=vault` in my `.zshrc` file, to ensure any program I run will
know which qube to connect to. This can be tested by running
`QUBES_GPG_DOMAIN=vault qubes-gpg-client-wrapper --list-keys`, which will
prompt me for access before performing the operation, which is incredible when
compared to alternative solutions. Be sure to run `qubes-gpg-import-key` to add
your GPG key to the system, and run `qubes-gpg-client-wrapper -K` to ensure you
have access to the private key.

While the Qubes team offers an out-of-the-box option for GnuPG and OpenPGP
operations, their support for SSH is less than ideal. In a previous version of
this article, I included an [adapted version] of the [community Split SSH]
guide, but I now have [a post detailing an improved version][improved-ssh].

<!-- TODO: this section isn't the best. Please add more information about U2F
and WebAuthn, and what they do to protect your privacy compared to TOTP and SMS
2FA. -->

# WebAuthn

I also followed the instructions for the [Qubes U2F Proxy] to the letter to
get a WebAuthn workflow working to the same device I have attached for PGP and
SSH and despite being slightly slow at times, for the most part it works fine.
I'm able to test this by going to [https://webauthn.io](https://webauthn.io).
WebAuthn is a protocol created by the [FIDO Alliance] to create an origin-based
signature system that, when run in a trusted environment such as a browser or
desktop session, can't be spoofed. This is due to the fact each cryptographic
signature is tied to an origin. Therefore, if I have a WebAuthn setup
configured for `github.com`, `guthib.com` can't spoof it because they'd *also*
make the device believe that it's requesting a signature for `github.com`.

A friend of mine recently [blogged about 2FA methods][xe-push-2fa-c-h]. It's an
interesting read that I think gives a good oversight into how the entire
security ecosystem relies on the possibility of human mistake. SMS and TOTP 2FA
are phishable by not having *anything* tied to the domain name, while
notification 2FA causes notification fatigue while also not being phishing
resistant as there's no way to verify the request is coming from an authentic
source - while the issue mentioned in the aforementioned post could have been
due to absentmindedness, there's no inherent way of verifying the origin of the
request.

**Note:** I have had minor issues with CloudFlare which I haven't been able to
debug yet, so if anyone runs into similar issues with CloudFlare and WebAuthn
not working in Qubes U2F Proxy, I'd be happy to either hear a solution or at
least hear that it's not just me.

[QubesOS]: https://qubes-os.org/
[Yubikey]: https://www.yubico.com/products/yubikey-5-overview/
[sys-usb]: https://www.qubes-os.org/doc/usb-qubes/
[qubes-gpg-client]: https://www.qubes-os.org/doc/split-gpg/
[adapted version]: https://github.com/hashbang/book/blob/master/content/docs/security/qubes/vault.md#creating-a-socket-for-ssh
[community Split SSH]: https://github.com/Qubes-Community/Contents/blob/master/docs/configuration/split-ssh.md
[Qubes U2F Proxy]: https://www.qubes-os.org/doc/u2f-proxy/
[FIDO Alliance]: https://fidoalliance.org/
[xe-push-2fa-c-h]: https://xeiaso.net/blog/push-2fa-considered-harmful
[improved-ssh]: /2022/10/02/exploring-qubes-rpc.html
