# Jiemi

[![tests](https://github.com/shigedangao/jiemi/actions/workflows/ci.yaml/badge.svg)](https://github.com/shigedangao/jiemi/actions/workflows/ci.yaml)
[![build](https://github.com/shigedangao/jiemi/actions/workflows/docker.yaml/badge.svg)](https://github.com/shigedangao/jiemi/actions/workflows/docker.yaml)

Jiemi is an experimental Kubernetes controller which decrypt and apply Kubernetes object that has been encrypted with [SOPS](https://github.com/mozilla/sops)
directly in the Kubernetes cluster w/o having to decrypt your file on your local machine.

# Install Jiemi

```bash
kubectl create ns jiemi
kubectl apply -f https://raw.githubusercontent.com/shigedangao/jiemi/main/manifest/crd.yaml
kubectl apply -f https://raw.githubusercontent.com/shigedangao/jiemi/main/manifest/manifest.yaml
```

The ```crd.yaml``` will register the Decryptor CRD in the Kubernetes cluster. The ```manifest.yaml``` will deploy the controller and the repository watcher on the jiemi namespace

# Usage

In order for Jiemi to decrypt your encrypted files on the Kubernetes cluster. You need to make sure that your encrypted files are located in a Git repository.

## Git authentication

Jiemi support multiple way of authenticating with your Git provider. Below are the method supported:

- SSH
- Username + Token
- No authentication (Public repository...)

### Authentication with username and token

To authenticate this way. You'll need to specify your git credentials. Either by using a kubernetes secret or by specifying the value in plaintext. 

> ⚠️ Specifying the credentials by using the literal method is practical for prototyping only. It's not recommended to use this way in production

Below is an example

```yaml
apiVersion: jiemi.cr/v1alpha1
kind: Decryptor
metadata:
  name: gcp-decryptor
spec:
  ...
  source:
    repository:
      url: <repo>
      credentials:
        username:
            literal: <git username>
        token:
            secretName: git-credentials
            key: token
    ...
```

### SSH

To authenticate with SSH. You'll need to create a Kubernetes secret which hold the private key which will allow Jiemi to clone the repository. Like the username + token authentication method. You can specified the SSH key w/o by using the literal property

```yaml
apiVersion: jiemi.cr/v1alpha1
kind: Decryptor
metadata:
  name: gcp-decryptor
spec:
  ...
  source:
    repository:
      url: <repo>
      credentials:
        ssh:
            secretName: git-credentials
            key: ssh
    ...
```

## Provider supported

SOPS support many encryption methods. Not all of these encryption tools are supported in Jiemi yet. Below are the list of encryption methods that are currently supported by Jiemi

- PGP
- Google Cloud KMS
- Amazon KMS

Below are example for each of them

### Google Cloud

```yaml
apiVersion: jiemi.cr/v1alpha1
kind: Decryptor
metadata:
  name: gcp-decryptor
spec:
  provider:
    gcp:
      serviceAccount:
        secretName: <secret which contains the json key>
        key: <key name>
  source:
    repository:
      url: <repository_url>
    fileToDecrypt: <path of file to decrypt>
    sopsPath: <filepath to .sops.yaml file>
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
      keyId:
        secretName: <secret which contains the aws_access_key_id>
        key: <key name>
      accessKey:
        secretName: <secret which contains the aws_secret_access_key>
        key: <key name>
      region:
        literal: <aws region>
  source:
    repository:
      url: <repository_url>
    fileToDecrypt: <path of file to decrypt>
    sopsPath: <filepath to .sops.yaml file>
```

### PGP

> ⚠️ Your PGP key must not have any password

```yaml
apiVersion: jiemi.cr/v1alpha1
kind: Decryptor
metadata:
  name: pgp-decryptor
spec:
  provider:
    pgp:
      privateKey:
        secretName: <secret which contains the exported pgp private key>
        key: <key name>
  source:
    repository:
      url: <repository_url>
    fileToDecrypt: <path of file to decrypt>
    sopsPath: <filepath to .sops.yaml file>
```

### Vault

> ⚠️ The http address of the vault must be specified in the .sops.yaml file

```yaml
apiVersion: jiemi.cr/v1alpha1
kind: Decryptor
metadata:
  name: vault-decryptor
spec:
  provider:
    vault: {}
  source:
    repository:
      url: https://github.com/shigedangao/gogo.git
    fileToDecrypt: vault/encrypted.yaml
    sopsPath: vault/.sops.yaml
```