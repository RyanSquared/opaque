---
layout: post
title: The Key to Commitment
date: 2024-06-12T04:49:00-0000
---

I recently came across a post on [threat models of encryption at rest][tmear]
and went down the rabbit hole of key commitment. As someone who is interested
in key derivation and client-provided keys (check out Keyfork!), I found it
interesting that the encryption scheme I've used in previous projects (AES-GCM)
as well as another encryption scheme [recommended] instead (XChaCha20-Poly1305)
would fall victim to key commitment attacks. As much as I'd like to immediately
start discussing key commitment, it's important to have an understanding of
authenticated encryption beforehand.

## Why AES-GCM, or AEAD in general?

While some encryption schemes can decrypt text into bogus output (but who's to
say that's not the intended output?), AES-GCM is an "Authenticated Encryption"
scheme. This means that, upon encrypting, an "authentication tag" is created,
and upon decrypting, the authentication tag is verified. If both party are not
using the same key, this results in an error in the decryption. Additionally,
if the ciphertext has been mutated by an attacker, decryption will also fail,
as the AEAD tag will be invalid.

## What is key commitment?

Key commitment is the assurance that the combination of the ciphertext and
authentication tag could only have been created by the key used to decrypt.
This is not a property that exists for AES-GCM or ChaCha20-Poly1305, as it is
possible to provide two plaintexts that, encrypted with two different keys,
produce the same combination of ciphertext and authentication tag. Therefore,
any system that refers to a combination of the ciphertext and authentication
tag as its own unique message may have logical issues where two different keys
and plaintexts produce the same combination.

Key commitment attacks are possible because GMAC, and presumably most
polynomial MAC functions (which would include Poly1305), do not incorporate the
key in the generated MAC in any way.  As such, it is possible for two distinct
keys to generate the same tag. This attack, called the Invisible Salamander
attack, is difficult to pull off, but the fact that it's possible even in
theory means there is room for improvement.

By incorporating the key, or the mechanism used to derive the key, into the
generation of the authentication tag, we are able to commit the key to the
message. CipherSweet chooses to incorporate BLAKE2b-MAC as a replacement
for Poly1305 in their encryption suite, but I believe, if HMAC key derivation
is used in the first place, incorporating an HMAC output into the AAD of the
function provides an irreversible component that can prevent invisible
salamander attacks while ensuring no protocol-level validation needs to happen
(as the AEAD function will take care of validating the payload).

### Why not validate a known value in the plaintext?

We can't be sure developers would validate the plaintext. Who's to say they
won't skip validating a block of all-zero at the start of a payload, simply
because my suggestion to use a constant-time validation function was too
complicated? It is best to implement it in an infallible manner, and Additional
Authenticated Data provides a good opportunity for this.

## When to use a Key Derivation Function?

Always. For encryption, a uniformly-random secret key is necessary. It is
always best to run whatever secret you have through some key derivation
function to get a uniformly random secret key to ensure it is suitable for
cryptographic purposes. It is important to note, ECDH *does not* provide keys
suitable for this purpose, and such keys *must* be put through an HKDF
function.

### Example: Creating a Non-Committed Message with No Authenticated Data

This example assumes that we are creating a _one time use_ key. As such, we can
derive the nonce from the key as well. If the key is being reused, _do not_
derive the nonce, and instead use a nonce that is impossible to be reused -
even a counter works better than a previously-used nonce.

```rust
// Crates used: aes_gcm, hmac, sha2
use aes_gcm::{
    aead::{consts::U12, Aead},
    Aes256Gcm, KeyInit, Nonce,
};
use hkdf::Hkdf;
use sha2::Sha256;

fn main() {
    let secret = b"AAAAAAAAAAAAAAAA";
    let hkdf = Hkdf::<Sha256>::new(None, secret);

    let mut key = [0u8; 32];
    hkdf.expand(b"key", &mut key).unwrap();
    let shared_key = Aes256Gcm::new_from_slice(&key).unwrap();

    let mut nonce = [0u8; 12];
    hkdf.expand(b"nonce", &mut nonce).unwrap();
    let nonce = Nonce::<U12>::from_slice(&nonce);

    let plaintext = "We should go out for tacos later";
    let ciphertext = shared_key.encrypt(nonce, plaintext.as_bytes()).unwrap();
    println!("{:?}", ciphertext);
}
```

This example provides zero additional authenticated data, but for the sake of
demonstration let's assume we're going to include some for the recipient, as
this will be built upon to enable key commitment.

### Example: Creating a Non-Committed Message with Authenticated Data

```rust
// Crates used: aes_gcm, hmac, sha2, bincode
use aes_gcm::{
    aead::{consts::U12, Aead, Payload},
    Aes256Gcm, KeyInit, Nonce,
};
use hkdf::Hkdf;
use sha2::Sha256;

fn main() {
    let secret = b"AAAAAAAAAAAAAAAA";
    let hkdf = Hkdf::<Sha256>::new(None, secret);

    let mut key = [0u8; 32];
    hkdf.expand(b"key", &mut key).unwrap();
    let shared_key = Aes256Gcm::new_from_slice(&key).unwrap();

    let mut nonce = [0u8; 12];
    hkdf.expand(b"nonce", &mut nonce).unwrap();
    let nonce = Nonce::<U12>::from_slice(&nonce);

    let plaintext = "We should go out for tacos later";
    let aad = b"20240612"; // Today's date
    let ciphertext = shared_key.encrypt(
        nonce,
        Payload {
    //  ^^^^^^^
    // Convert to a Payload object to attach AAD
            msg: plaintext.as_bytes(),
            aad: &bincode::serialize(&[&aad][..]).unwrap(),
        }
    ).unwrap();
    println!("{:?} {:?}", aad, ciphertext);
    //                    ^^^^
    // Send the publicly knowable AAD to the recipient
    let plaintext = shared_key.decrypt(
        nonce,
        Payload {
            msg: &ciphertext,
            aad: &bincode::serialize(&[&aad[..]][..]).unwrap(),
            // The recipient can use the AAD they got alongside the ciphertext
        },
    ).unwrap();
}
```

We use bincode here as a canonicalization format. Blindly concatenating AAD
together could result in a [canonicalization attack]. Using bincode provides
a protection against canonicalization attacks by length-prefixing parts, and by
providing a slice to bincode, we ensure we include the length of the parts in
the AAD, preventing a potential hash collision by an attacker appending
additional AAD.

Attempting to decrypt this data requires AAD to be passed in; if the ciphertext
is passed as-is, the decryption will fail. This means the AAD must be either
included in the wire format or derivable by both parties. For deriving further
data from the HKDF function, we can go with the latter.

## How to commit the key

In the above examples, we've passed an "info" parameter to the key derivation
function, to derive data and generate a brand new output. Using this mechanism,
we can generate a new data, called our "commitment", and attach it to the AAD.
We do not include this in the payload sent to the recipient, as it is assumed
the recipient can recreate this commitment as well, proving the key used to
decrypt is the key included in the commitment.

### Example: Creating a Key-Committed Message with Authenticated Data

```rust
// Crates used: aes_gcm, hmac, sha2, bincode
use aes_gcm::{
    aead::{consts::U12, Aead, Payload},
    Aes256Gcm, KeyInit, Nonce,
};
use hkdf::Hkdf;
use sha2::Sha256;

fn main() {
    let secret = b"AAAAAAAAAAAAAAAA";
    let hkdf = Hkdf::<Sha256>::new(None, secret);

    let mut key = [0u8; 32];
    hkdf.expand(b"key", &mut key).unwrap();
    let shared_key = Aes256Gcm::new_from_slice(&key).unwrap();

    let mut nonce = [0u8; 12];
    hkdf.expand(b"nonce", &mut nonce).unwrap();
    let nonce = Nonce::<U12>::from_slice(&nonce);

    let mut commitment = [0u8; 16];
    hkdf.expand(b"commitment", &mut commitment).unwrap();
    //          ^^^^^^^^^^^^^
    // Expand a new commitment field

    let plaintext = "We should go out for tacos later";
    let aad = b"20240612";
    let ciphertext = shared_key.encrypt(
        nonce,
        Payload {
            msg: plaintext.as_bytes(),
            aad: &bincode::serialize(&[&aad[..], &commitment[..]][..]).unwrap(),
            //                                   ^^^^^^^^^^^^^^^
            // Include our commitment in our AAD
        }
    ).unwrap();
    println!("{:?} {:?}", aad, ciphertext);
    // NOTE: We do _not_ include the derived commitment when sending the
    // payload to the recipient, this is what commits the key to the AAD.
    let plaintext = shared_key.decrypt(
        nonce,
        Payload {
            msg: &ciphertext,
            aad: &bincode::serialize(&[&aad[..], &commitment[..]][..]).unwrap(),
            //                                   ^^^^^^^^^^^^^^^
            // Now the recipient passes the commitment, to ensure they're using
            // the correct key.
        },
    ).unwrap();
}
```

If you are concerned about Invisible Salamander attacks, refer to your local
cryptographer for more advice on whether this solution may be relevant for you.

---

EDIT (2024-06-12T06:50:00Z-0000): Restate the importance of key commitment, the
types of key commitment attacks, and add unwrap statements to the decrypt
function.

[tmear]: https://scottarc.blog/2024/06/02/encryption-at-rest-whose-threat-model-is-it-anyway/
[recommended]: https://soatok.blog/2020/05/13/why-aes-gcm-sucks/
[canonicalization attack]: https://soatok.blog/2021/07/30/canonicalization-attacks-against-macs-and-signatures/
