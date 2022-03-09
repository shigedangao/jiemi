## Jiemi

Jiemi is a Kubernetes controller which decrypt and apply Kubernetes object that has been encrypted with [SOPS](https://github.com/mozilla/sops)
directly in the Kubernetes cluster w/o having to decrypt your file on your local machine.

## Install (TODO)

To install Jiemi. You need to apply these 2 manifests:

```
kubectl apply -f <crd.yaml path>
kubectl create ns jiemi
kubectl apply -f <manifest.yaml path>
```

The ```crd.yaml``` will register the Decryptor CRD in the Kubernetes cluster. The ```manifest.yaml``` will deploy the controller and the repository watcher on the jiemi namespace

## Usage

To decrypt your encrypted SOPS files. First, make sure that the encrypted SOPS files are located within an existing repository. It could either be public or private. Jiemi support authentication with GIT with the following methods:

- SSH
- Username + Token
- No authentication

To authenticate with one of the following method above. You can use the following options

#### Username + Token

```yaml
apiVersion: jiemi.cr/v1alpha1
kind: Decryptor
metadata:
  name: gcp-decryptor
spec:
  provider: {}
  source:
    repository:
      url: <repo>
      credentials:
        username:
            secretName: git-credentials
            key: username
        token:
            secretName: git-credentials
            key: token
        ssh:
            secretName: git-credentials
            key: ssh
    fileToDecrypt: <file to decrypt>
    sopsPath: <sops path>
```

#### SSH

```yaml
apiVersion: jiemi.cr/v1alpha1
kind: Decryptor
metadata:
  name: gcp-decryptor
spec:
  provider: {}
  source:
    repository:
      url: <repo>
      credentials:
        ssh:
            secretName: git-credentials
            key: ssh
    fileToDecrypt: <file to decrypt>
    sopsPath: <sops path>
```

Below are some examples

### Google Cloud

```yaml
apiVersion: jiemi.cr/v1alpha1
kind: Decryptor
metadata:
  name: gcp-decryptor
spec:
  provider:
    gcp:
      service_account:
        secretName: gcp-secret
        key: mask-336418-5d828a98a0a8.json
  source:
    repository:
      url: https://github.com/shigedangao/gogo.git
    fileToDecrypt: test/foo.enc.yaml
    sopsPath: test/.sops.yaml
```

### AWS

```yaml
apiVersion: jiemi.cr/v1alpha1
kind: Decryptor
metadata:
  name: aws-decryptor
spec:
  provider:
    aws:
      key_id:
        secretName: aws-secret
        key: id
      access_key:
        secretName: aws-secret
        key: access
      region:
        literal: eu-west-3
  source:
    repository:
      url: https://github.com/shigedangao/gogo.git
    fileToDecrypt: aws/foo.enc.yaml
    sopsPath: aws/.sops.yaml
```

### PGP

```yaml
apiVersion: jiemi.cr/v1alpha1
kind: Decryptor
metadata:
  name: pgp-decryptor
spec:
  provider:
    pgp:
      privateKey:
        secretName: pgp-secret
        key: private.rsa
  source:
    repository:
      url: https://github.com/shigedangao/gogo.git
    fileToDecrypt: pgp/secret.enc.yaml
    sopsPath: pgp/.sops.yaml
```

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