## Jiemi

Jiemi is a Kubernetes controller which decrypt and apply Kubernetes object that has been encrypted with [SOPS](https://github.com/mozilla/sops). The project is divided into 3 Rust projects.

- Gen: Crate define the CRDs and contains some methods which can be used to manipulate the CRD
- Miwen: Binary app uses as a kubernetes controller. It checks periodically the CRD and check whenever we need to update the decrypted files
- Krapao: Binary app uses to handle repository operations, authenticate with cloud provider & decrypt sops files

## Note on PGP

Extract key to import on SOPS

```sh
gpg --export-secret-keys --armor <fingerprint> > private.rsa
```

On OSX we might need to export an env var due to issue with gpg. See [here](https://jhooq.com/failed-to-get-the-data-key/)

# Build dockerfile

## Miwen

docker build -f .build/Dockerfile.miwen -t jiemi/miwen .

## Krapao

docker build -f .build/Dockerfile.krapao -t jiemi/krapao .