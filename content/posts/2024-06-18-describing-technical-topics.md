---
layout: post
title: Describing Technical Topics
date: 2024-06-18T23:12:00-0000
---

In [a recent post], I struggled to properly demonstrate the importance of key
commitment, which happened to be the entire topic of the post. In doing so, I
caused confusion for people reading the post, and had to go back and rewrite
an entire section to more clearly represent the topic I was discussing. Posts
that I've read online often discuss a technical topic without properly defining
what the topic is about, what purpose it serves, and how it is implemented,
with my previous post contributing to this concerning amount of words thrown to
the void with no context to serve them.

When I was rewriting the section on key commitment, I decided I needed to set
myself three major goals with the section:

## Define the topic and the concept in which it exists

> Key commitment is the assurance that the combination of the ciphertext and
> authentication tag could only have been created by the key used to decrypt.
> This is not a property that exists for AES-GCM or ChaCha20-Poly1305, as it is
> possible to provide two plaintexts that, encrypted with two different keys,
> produce the same combination of ciphertext and authentication tag. Therefore,
> any system that refers to a combination of the ciphertext and authentication
> tag as its own unique message may have logical issues where two different
> keys and plaintexts produce the same combination.

Through the description, the reader is made aware of what key commitment is (a
property of encrypted text, or ciphertext, and an authentication tag), and the
cryptography cipher suites that the property does not apply to. The description
also includes what this means for those cipher suites, and a quick explanation
of why it may be a concern if this property is not included.

A user might not be reading your article because they're already aware of what
a topic is. That's what makes these descriptions so important. You're as likely
to have a subject matter expert read your article as you are to have someone
entirely new to the space.

## Express the concern with the topic that the solution tends to resolve

When writing articles of "this subject has a flaw" or "this implementation does
not uphold this property" (as is our case), it is important not only to make
this clear, but to demonstrate _why_:

> Key commitment attacks are possible because GMAC, and presumably most
> polynomial MAC functions (which would include Poly1305), do not incorporate
> the key in the generated MAC in any way.  As such, it is possible for two
> distinct keys to generate the same tag. This attack, called the Invisible
> Salamander attack, is difficult to pull off, but the fact that it's possible
> even in theory means there is room for improvement.

In the case of key commitment, the reader is informed of why the attacks are
possible, and what the result may be if this is left unmitigated. For other
topics, it may be useful to demonstrate why such a property being applied to a
specific subject may be useful (such as why reproducible builds may be useful
for Linux containers, or why rate limiting may be useful for authentication
endpoints). This section typically carries the most significant weight, as it
is the problem, for which the article and the techniques discussed within, is
the solution.

## Solve the problem

Easier said than done, now that the flaw has been identified, a solution needs
to be implemented.

> By incorporating the key, or the mechanism used to derive the key, into the
> generation of the authentication tag, we are able to commit the key to the
> message. CipherSweet chooses to incorporate BLAKE2b-MAC as a replacement for
> Poly1305 in their encryption suite, but I believe, if HMAC key derivation is
> used in the first place, incorporating an HMAC output into the AAD of the
> function provides an irreversible component that can prevent invisible
> salamander attacks while ensuring no protocol-level validation needs to
> happen (as the AEAD function will take care of validating the payload).

The solution for the problem can take many forms. Sometimes, it may be "throw
out the old system and implement the new system". Sometimes you may choose to
change cipher suites or protocols. Sometimes, as was the case with my key
commitment article, you may choose to consider what solution can be built upon
the existing foundation. It is worth considering all possible options and even
mentioning to the user solutions that may exist for similar, but alternative,
problems.

[a recent post]: /posts/the-key-to-commitment
