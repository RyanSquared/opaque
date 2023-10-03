---
layout: post
title: Plight of Preconfigured Passphrase Protection
date: 2023-06-05T18:29:00-0400
---

Warning: This is a bit of me talking about an experience. Rather than me
solving a problem, it's more me talking about the problem and moving on. I have
yet to see the problem solved with the program I'm using.

Recently, I've been looking into options for an OpenID Connect provider that I
can hook into [userdb](https://github.com/hashbang/userdb) and for uses as an
independent user database. This lead me to consider two options: Keycloak and
Authentik. I looked at both and it appeared that it would fit within
requirements I'll likely talk about in a blog post a few years from now. I
decided to go with Keycloak at least for personal (and Distrust) infra.

I write all of my infrastructure declaratively, so if necessary I can deploy as
much as possible using a set of `kustomize build` invocations.  While writing a
Keycloak Kustomization, I noticed that I could specify the initial passphrase
of the Keycloak admin using [environment variables][kc-1]. However, this value
only accepts the passphrase of the admin user, not a passphrase hash like
[ArgoCD][argo-1] or [Ingress HTTP Basic Auth][nginx-1]. In this case, it's the
admin account, so it's not as though everyone could see everyone else's
passphrase, but it's still not ideal that the passphrase is stored in a
plaintext environment variable. This issue has been raised with
[Keycloak][kc-2] but as of when I've written this article, no action has been
taken.

While I'd like to believe that running things in Kubernetes containers is
secure, I also prefer to plan for the eventuality that, no matter how hard I
prepare, there will always be some software bug or configuration mishap that
will expose some information I don't want exposed. If some component were able
to access the Kubernetes Secret that I've stored the passphrase in, for
example, they would then have access to the admin account of the instance. I
encrypt the file locally and audit all RBAC policies I put in my Kubernetes
cluster, but it would be easier to prepare for the inevitable if I could
provide a passphrase hash.

This issue also exists with Authentik ([here][authentik-1]).

[kc-1]: https://www.keycloak.org/server/configuration#_setup_of_the_initial_admin_user
[kc-2]: https://github.com/keycloak/keycloak/issues/20138
[argo-1]: https://github.com/hashbang/gitops/blob/e9bffb479037571aa29e832c5c155721bc7cc6e9/argocd/argocd-secret.enc.yaml#L9-L22
[nginx-1]: https://github.com/hashbang/gitops/blob/568b1a65baa47e5847d77bd2445ed6f8c0f0a530/monitoring/ingress.yaml#L7-L9
[authentik-1]: https://github.com/goauthentik/authentik/issues/5471
