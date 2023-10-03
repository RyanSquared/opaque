---
layout: post
title: Abuse of Surveillance Technologies
tags: mediapanel, thoughts
date: 2022-12-09T14:03:00-0500
---
This post is made in response to the Raspberry Pi announcement, where they
hired an ex-cop who had previously built surveillance technologies based on the
Raspberry Pi. Their response to the backlash has been horrendous but I am a
technical person first and foremost so I will leave the evaluation of their
handling of the situation to others, and talk about my experience with similar
technologies here.

A few years ago, I used to work for a Raspberry Pi based digital signage
company. I was improving an established product to incorporate new features to
ensure that the product stays marketable and sustainable in the current digital
signage climate. This included features like adding new integrations to our
web configuration tool, ensuring that the device worked properly without an
Internet connection, and adding hardware functionality to the device itself,
such as HDMI-CEC and alternative audio output support.

Eventually, the owner of the company was looking to sell the product to one of
the clients. The product wasn't as profitable as he would have liked and he
wanted to spend more time working on other products that would become more
profitable in the long run. He decided to contact the people who owned the most
devices to see if they were interested in buying the company. One of those
people eventually said yes, bought the product and IP related to it, and became
my new boss.

While the product itself was mostly self-sustaining and low maintenance, the
costs put into marketing the product and paying my salary eventually caught up
to my boss. I believe this is the point he started looking at any and every
measure to ensure that the company could become profitable. At this point, an
advertising company reached out to him to integrate an optional advertising
service into the product. This would be priced at a lower cost for the clients
while ensuring that we maintain some extra profit so long as these devices
remain operational. The biggest problem was, the advertisement system would
have required a way to track engagement.

Engagement for advertising purposes typically relies on "impressions" and
"interactions", where an "impression" is whether or not someone has seen an
advertisement, and an "interaction" is whether or not someone has decided to
further research the thing being advertised. Interactions are not too important
for this -- a QR code can embed a tracker with a reference code -- but
impressions are a difficult thing for something that exists in a public space.
The only way to track it would be to see where people are looking.

With a camera.

The company wanted us to ship our devices with a hidden camera so we could
monitor where everyone was looking, so they could track whether or not people
were looking at the screens, and if so, if they were looking at the screens
during the advertisements. This means that people's identity, where they were,
who they were with, what they were looking at, and likely even more information
would be sent to a third party advertiser. This advertiser would then take the
information and give us a payout on whether or not they were engaged in some
manner with the screen.

I had interacted with most of the software that runs on these devices, and with
the software that handles the backend. From a security perspective, it was a
disaster. There were so many different ways to get into the device that I would
recommend to customers to isolate it on their networks and to have people
occasionally check up on the device to ensure it's still displaying what it was
supposed to be displaying. Putting something like the advertising system in a
device that would be that easy to compromise would be incredibly dangerous.

This was a system that was used in warehouses, clinics, and churches, with my
boss also attempting to target stores, pharmacies (because it's the US --
advertising medicine is a great idea!), and other businesses. Having that much
information about what people are doing in these locations is an incredible
security risk, and I absolutely would not support adding technology that could
track the way people are interacting with these places.

I eventually told my boss that I would not be implementing such a technology
into the product because it is a fundamentally abusable technology that could
result in a significant impact on the people it spied on. It is not something I
would have wanted to involve myself in and it's absolutely not something I
would want to support. While I can't say I've seen the effects of this being
implemented at a company, I have seen the possibilities of it being applied,
and I would never trust someone to say they have a "secure surveillance system"
after the things I've seen working at this company.
