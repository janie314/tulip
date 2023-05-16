# tulip CLI

```
Btw, on an unrelated issue: I see that Jason actually made the pull
request to have wireguard included in the kernel.

Can I just once again state my love for it and hope it gets merged
soon? Maybe the code isn't perfect, but I've skimmed it, and compared
to the horrors that are OpenVPN and IPSec, it's a work of art.

              Linus
```

The tulip. CLI.

At the application level, Tulip is a thin abstraction over Wireguard. At the
network level, Tulip is Wireguard.

[[_TOC_]]

# Dependencies

[WireGuard](https://www.wireguard.com/),
[Rust](https://www.rust-lang.org/learn/get-started), what's in the `Cargo.toml`,
`make`, `sudo`.

# Usage for Tulip Network Users

- Create a public and private ID with `tulip gen-id`.
- To join a Tulip network, you will have to give a network administrator your
  `public_id.json`. NEVER share your `private_id.json`.
- To start a Tulip network, use `tulip start`.
- To stop a Tulip network, use `tulip stop`.
- To join a Tulip network with the iPhone or Android WireGuard app, generate a
  separate `private_id.json` and `public_id.json`, have a network administrator
  approve the new `public_id.json`, and use `tulip gen-wg-conf`.

# Usage for Tulip Network Admins

- Maintain your `tulip_network.json` and `phonebook.json` files, whose schemas
  are detailed below.
- Make sure `phonebook.json` is available at the HTTP endpoint `/phonebook.json`
  on your Tulip network's WireGuard IP address.
- To start a Tulip network, use `tulip start --server`.
- To stop a Tulip network, use `tulip stop`.
- To provision a user's `tulip_network.json` file, use `tulip gen-net-conf`.
  (Note: the user must already be added to your `phonebook.json` manually).

## `tulip_network.json`

E.g., if your Tulip network is called "Sandringham", then
`sandringham_tulip_network.json` might look like this.

```json
{
  "name": "sandringham",
  "subnet": "10.0.0.0/16",
  "user": {
    "name": "janie",
    "vpn_ip": "10.0.0.4"
  },
  "public_endpoints": [
    {
      "name": "appleton",
      "vpn_ip": "10.0.0.3",
      "public_hostname": "vpn.example.com",
      "public_key": "lNYWO/sIEmu51/2uBZQfaECU9DTw+tBl8IsgMM+XjVU=",
      "port": 23235
    }
  ]
}
```

## `phonebook.json`

```json
{
  "diana": {
    "name": "diana",
    "vpn_ip": "10.0.0.3",
    "public_key": "F9JGSvSOEIVOXyJT3iBu6HqECTz1b6TpadcuXA71jUE="
  },
  "harry": {
    "name": "harry",
    "vpn_ip": "10.0.0.2",
    "public_key": "hcKLrJd1+vrDphARIRZFMGsvBSEpmS/c3AOpaJz033Q="
  }
}
```
