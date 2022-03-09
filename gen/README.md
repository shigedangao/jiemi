## Gen

Gen is a lib which contains the CRD. It also contain a binary app which is used to generate the CRD in the YAML format.

### Usage

By running the command below. This will generate the `crd.yaml` file which can be used in the Kubernetes cluster to add the Decryptor CRD

```
cargo run <path option>
```

### Unit test

This project contain some unit tests. To run these tests you'll need to have a running local kubernetes cluster. Then you need to apply this file

```
kubectl apply -f example/test/secret.yaml
```

Once done, you can run tests with the command

```
cargo test
```