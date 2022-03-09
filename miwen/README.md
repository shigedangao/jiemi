## Miwen

Miwen is a Kubernetes controller which is used to synchronize CRD with the cluster. It's also used to check repository from time to time specified in the CRD. If changes has been detected, Miwen will synchronize the associated files define in the CRD

### Features

- [x] Create a CRD (gen)
- [x] Listen to Kubernetes events
- [x] Parse CRD and read kubernetes secrets
- [x] Update the status of the CRD when something need to be synchronize
- [x] Synchronize on CRD changes (by checking the generation_id which is different from the stored one)
- [x] Synchronize changes from git repository by calling the rpc endpoint from time to time
- [x] Add a limit to the number of synchronization status in the CRD
- [x] Add more unit test
- [x] Delete an item of the state when a CRD is removed

### Configure

OSX required you to install `openssl@1.1`

### Unit tests

This project contain some unit tests. To run these tests you'll need to have a running local kubernetes cluster. Then you need to apply this file

```
kubectl apply -f example/test/decryptor.yaml
```

Once done, you can run tests with the command

```
cargo test
```