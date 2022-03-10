# Architecture

The architecture is based on a similar architecture like ArgoCD (albeit more simpler). Jiemi is made of 3 main components

- Krapao: A statefulset RPC service which is in charge of syncing repository and to decrypt the SOPS files
- Miwen: Controller which watch the status of the CRD and synchronize the targeted file with the cluster at a regular interval of time
- Custom Resource Definition (CRD): Used to define the how Krapao will decrypt the encrypted files by SOPS

## What happened when a CRD is created ?

```
-------    ---------    ----------    ---------
| CRD | -> | Miwen | -> | Krapao | -> | Miwen |
-------    ---------    ----------    ---------
```

When a CRD is applied on the Cluster. The controller (Miwen) will check the field of the CRD. The controller will execute the following checklist:

- Add the CRD to the state (if not exist)
- Call krapao to clone the targeted repository defined in the CRD
- Get the decrypted template from Krapao
- Apply the decrypted templated on the Kubernetes cluster
- Then miwen will continue to watch this ressource and sync if there are any changes on the repository / crd

## What happened if the targeted file is updated on the repository

As miwen is checking the repository from time to time. Miwen will update the targeted file regularly every 3 minutes

## Components

## Gen

Gen is a binary - lib app which contains the definition of the CRD. The crate contains a set of struct and associated method which are also used by the controller. 

The CRD is generated with [kube-rs](https://github.com/kube-rs/kube-rs)

## Miwen

Miwen is the controller. It watch the CRD and check if there are any changes on the targeted repository (every 3 minutes)

> Basically we're comparing the commit hash is different from the one stored in the CRD status field

## Krapao

Krapao is a RPC server which is used for 4 tasks:

- Clone & Sync the repository defined in the CRD
- Authenticate with cloud provider
- Decrypt files encrypted with SOPS
- Exposing RPC endpoint in order for miwen to communicate with Krapao

## CRD

The CRD is pretty simple. You can find examples in the example folder or on the main README.md file. Additionally, the CRD is update by miwen whenever a Synchronization happened. Miwen will update the `Status` field of the CRD. The `Status` field contain the following items

```
Status:
  Current:
    deployed_at:      2022-03-08T16:31:30.647730803+00:00
    error_message:    <nil>
    file_to_decrypt:  test/foo.enc.yaml
    Id:               1
    Revision:         a888f02e1111beb2c543d729faa5d516ecaa9e12

    Status:  Sync
  History:
```