---
layout: post
title: Running a Pubnix in Docker for Fun and Profit
date: 2024-09-24T21:41:00-0000
---

My first foray into using Linux as a system administrator (not simply as a
user) was when I began contributing to the Hashbang Shell Server project, a
pubnix (public UNIX system, Debian 7 or so, in this case) with a detached user
database, allowing users to connect to any system connected to the Hashbang
userdb (user database). I joined the project around 2015, contributing changes
to just about every component in the system, working my way into the admin
team. Several years later, we began migrating to an "infrastructure as code"
pattern, where we could set up servers in a reproducible pattern using
configuration files. This allowed us to set up a Debian server, point [Ansible]
to the server, and let it work its magic.

In the process of converting the servers to infrastructure-as-code, we decided
we needed a way to rapidly develop changes against clean Debian images without
constantly nuking and paving cloud VMs. During that time, we found [Packer],
which, given a base operating system, runner (VM, Docker, etc.), and
provisioner (Ansible) configuration, could generate VMs built using our
provisioner configuration. Using [QEMU] with packer lead to a much faster
development process, but we also had a builder with [Docker] which would drop
users into a shell. This was useful for the most basic testing purposes, as we
were able to confirm that configuration files looked correct and the requested
packages were appropriately installed.

A year later, we came to a shocking realization: we can just run `systemd`
inside Docker. `systemd` is the init process for Debian, managing services and
other necessary resources to "make things do" properly. Running `systemd` in
Shell Server could automatically enable SSH, which would let users be able to
log in. With this, we were able to (technically) create a pubnix in Docker,
where users could connect using their hashbang credentials.

Can I recommend users use this pattern, for making a pubnix in Docker?
Actually, no. The security features I would recommend for a pubnix (see
[hashbang's configuration][config]) rely _heavily_ on the same features Docker
and other container runtimes use for themselves, and as such, are not made
available within the container.

However, I would like to make a nice observation that I think people may have
forgotten about in the transition from "virtualize everything" to
"containerize everything", which is that you _can_ run multiple daemons, or
even an entire operating system, in your container, if you want.

If you're interested, all the configuration for the project is at
https://github.com/hashbang/shell-server.

[Ansible]: https://ansible.readthedocs.io/projects/ansible-core/en/devel/
[Packer]: https://www.packer.io/
[QEMU]: https://www.qemu.org/
[Docker]: https://docs.docker.com/engine/
[config]: https://github.com/hashbang/shell-server/blob/797cec330b69fcc022581593aba42c0cd4641133/ansible/tasks/security/main.yml
