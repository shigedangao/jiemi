## Krapao

Krapao is a RPC server which is used to synchronize repository defined in the Decryptor CRD. It's also used to authenticate with third parties provider such as Google Cloud, AWS in order to decrypt encrypted files with SOPS. Below is what's supported by the Krapao service

### Features

- [x] Clone repository w/o ssh
- [x] Clone repository with token only
- [x] Clone repository with ssh
- [x] Synchronize repository with interval
- [x] Decrypt targeted files by using SOPS
- [x] Expose RPC endpoint to be used by the kubernetes controller
- [x] Store existing repository in a state to when restarting krapao in order to not clone the repo again...
- [x] Authenticate with GCP provider
- [x] Authenticate with AWS provider 
- [x] Support pgp key - note: pgp key w/o pwd

### Unit tests

This project contain some unit tests. In order for the test to be successfull. You'll need to create an `Env.toml` file within the Krapao project. The Env.toml should contains these variables

```toml
GIT_USERNAME="<replace with your git username>"
GIT_TOKEN="<replace with your git token>"
GIT_SSH_KEY="""
<replace with your git private key>
"""
```

Once done. You can run tests with the command below:

```shell
cargo test -- --test-threads=1
```