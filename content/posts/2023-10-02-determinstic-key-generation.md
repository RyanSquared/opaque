---
layout: post
title: Deterministic Key Generation with Keyfork
date: 2023-10-02T20:15:00-0500
---

For the past few years, I've been working with Distrust as a contractor to
various companies. During that time, I've been witness to many clients
wondering how to properly secure the cryptographic "keys to their kingdom". In
the past, my usual response would be to generate the keys, encode them using a
QR code or other machine-readable yet still physical format, and store the
physical backup in an on-site safe or safe deposit box. However, after some
exploits have shown entropy generation to be somewhat weaker than expected
(such as the Milk Sad issue, or Infineon RSA key generation issue), some
developers who have been impacted by this issue (and me, even though I wasn't
directly affected) have decided to begin working on a project called [Keyfork].

In early 2012, a Bitcoin Improvement Proposal, BIP-0032, was introduced to
make key generation deterministic. Instead of a Bitcoin client creating 100 or
so keys to be backed up, a single "master key" is generated, and further keys
are derived from that. Whilst Bitcoin uses derived keys to offer further
derivation of sequential public keys, the reason Keyfork uses BIP-0032 (from
here, "bip32") is to derive multiple private keys that could be used for any
situation, such as (re-)provisioning infrastructure or generating cryptographic
keys to load onto physical hardware, such as an OpenPGP card.

BIP-0039 (from here, "bip39") was later introduced in late 2013, offering a way
to serialize a bip32 seed as a mnemonic, a combination (note: not permutation)
of 12 to 24 words, chosen from a set of 2048, offering up to 256 bits of
encoded data. This quickly became adopted by many cryptocurrency wallets as a
way of backing up and restoring cryptographic keys, as a mnemonic was easy to
write down on a sheet of paper. Hardware wallets, such as Trezor or Ledger,
come with a "recovery seed card" which can be used to write down such
mnemonics. During the research for Keyfork's usability, the convenience of a
24-word mnemonic was quickly made obvious, and it was incorporated into the
list of planned features for Keyfork. It was the first feature developed, with
the first Keyfork binary being the now-deprecated `keyfork-mnemonic-generate`.

The combination of bip32 and bip39 promised a very convenient workflow: a user
should be able to enter their mnemonic and retrieve some form of private key,
such as a hex-encoded private key, an "extended" private key (which could be
used for further derivation) or an ASCII-armored self-signed OpenPGP keychain.
Utilizing Keyfork, it should be possible to regenerate any secret data
on-demand. The repository for Keyfork is not at a production-ready state yet,
but so far it is possible to generate an OpenPGP key which other software can
use to load onto an OpenPGP card, which I hope will encourage developers to
improve the supply chain security of their projects, with signed commits and
releases. As an added bonus, it should also be made possible to store and
retrieve the types of keys that have been generated via a configuration file.

My future goals for Keyfork include provisioning hardware such as OpenPGP
cards, supporting more formats, and documenting secure use of Keyfork on
systems such as QubesOS that may take advantage of process isolation, ensuring
that even if the derived data is leaked in one VM, the mnemonic and master key
may remain secure in another VM. The most difficult part of this, however,
would be designing Keyfork to provide these security benefits with zero
compromise to the user experience. Keyfork is still in early development and
any influence about the design of the project is *greatly appreciated*!

---

If you are interested in Keyfork development, let me know! The [repo][Keyfork]
is public (with an RSS feed!) and despite our Matrix room being invite-only,
ask me for an invite at [@ryansquared:beeper.com][matrix].

[keyfork]: https://git.distrust.co/public/keyfork
[matrix]: https://matrix.to/#/@ryansquared:beeper.com
