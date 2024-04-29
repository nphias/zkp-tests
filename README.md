# Chaum-Pedersen Zero Knowledge Proof

This program is an implementation of the Chaum-Pedersen ZKP protocol that allows
the registration of a user client in a server without exposing the secret / password

# Features

The code currently supports:

-  Integer cyclic group activated by default or with the `--scalar` command line option.
-  Elliptic curve secp256k1 cyclic group activated with the `--elliptic` curve command line option.
-  Support for very large integers by using the `num-bigint` Rust crate.
-  Docker containerization.

# Default parameters

For the integer and elliptic curve cyclic groups the known parameters of the algorithm are hardcoded.

1. Scalar or integer cyclic groups:

```
p = 10009
q = 5004
g = 3
h = 2892 (g^13 mod p)
```

Note that these numbers are very small. They shouldn't be use in production.

2. An elliptic curve cyclic group based on the secp256k1 curve

```
p = 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f
q = 0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141
g = (
    x:0x79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798,
    y:0x483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8
    )
h = 13 * g
```

An adapted `src/secp256k1.rs` library is included in the code. 

Note that the constant `13` for computing `h` was arbitrary selected. From what
[1] states `g` and `h` should be of prime order `q`:

```
g ^ q mod p = 1
h ^ q mod p = 1
```

# Dependencies

- `rustc` (compiler) and `rustup` (package manager)
- `cmake` for the gRCP library
- `docker` and `docker-compose` if you are going to run in a docker container

If you have all the dependencies installed, then run from the main directory the
following command:

```bash
$ cargo build --release
```

Note that this should generate on the `./src` folder a file called
`zpk_auth.rs`. This file is the interface generated with `tonic` from the
`./proto/zkp_auth.proto` Protobuf file. This file specifies the communication
protocol between server and client.

Then, test that everything works fine by executing:

```bash
$ cargo test
```

# Run locally

open 2 separate terminals, one for running the server and the other
for the client since both produce useful outputs for debugging and understanding
what happens.

Execute the server:

```bash
$ cargo run --bin server -- [--scalar(default)|--elliptic]
```

The server listens all the time for any message of any client and communicates
using the gRPC protocol.

Execute the client:

```bash
$ cargo run --bin client -- [--scalar(default)|--elliptic]
```

Note that both, the server and the client, should use the same cyclic group,
i.e, both using the integer (`--scalar`) fields, or both using the elliptic
curves field (`--elliptic`).

# Run with Docker

You will need to have `docker` and `docker-compose`. Open two terminals and in
one build the docker image and run the server:

On the other terminal, connect to the running docker container and run the
client:

The client connects to the server and then runs a for-loop that:

1. Ask for a username.
2. Ask if you want to send the solved challenge verification.
3. Logs and shows if the login was successful or not.


# References
1. [Bitcoin Rust](https://github.com/gagiuntoli/bitcoin_rust)