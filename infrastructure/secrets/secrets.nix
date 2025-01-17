
let
  catcolab      = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIMOMBnXy4XMcogVMXdXt9NQCyAy8FaBj53groT4TTKBA root@ip-172-31-25-34.us-east-2.compute.internal";
  catcolab-next = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIPyxORhhfO+9F2hQZ3I/EiSpfg+caWpG6c8AuG5u1XtK root@ip-172-31-14-38.us-east-2.compute.internal";
  catcolab-test = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIBZaycYjvaZ5XhxVIvFr8zXvcy1GrViCKLIZCalZuk1l root@ip-172-31-9-45.us-east-2.compute.internal"; # test (running)
  # catcolab-test = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAB56R8zRFmuMOhsoJBWNP9YrsoZwQ9JZQcYoZUHOgRo root@ip-172-31-9-138.us-east-2.compute.internal"; # test (terminated)
  owen          = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIF2sBTuqGoEXRWpBRqTBwZZPDdLGGJ0GQcuX5dfIZKb4 o@red-special";
  epatters      = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAKXx6wMJSeYKCHNmbyR803RQ72uto9uYsHhAPPWNl2D evan@epatters.org";
  shaowei       = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOV/7Vnjn7PwOC9VWyRAvsh5lUieIBHgdf4RRLkL8ZPa shaowei@gmail.com";
in
builtins.mapAttrs (_: publicKeys: {inherit publicKeys;})
  ({
      ".env.age"           = [ catcolab catcolab-next catcolab-test owen epatters shaowei ];
      "instrument.mjs.age" = [ catcolab catcolab-next catcolab-test owen epatters shaowei ];
  })
