## Jiemi

Jiemi is a Kubernetes controller which decrypt and apply Kubernetes specs that has been encrypted with [SOPS](https://github.com/mozilla/sops). The project is seperated into 3 seperate Rust projects.

- Gen: Crate define the CRDs and contains some methods which can be used to manipulate the CRD
- Miwen: Binary app uses as a kubernetes controller. It checks periodically the CRD and check whenever we need to update the decrypted files
- Krapao: Binary app uses to handle repository operations, authenticate with cloud provider & decrypt sops files

### Miwen

Need to install openssl@1.1 (osx)

## TODO for Krapao

- [x] Clone repository w/o ssh
- [x] Clone repository with token only
- [x] Clone repository with ssh
- [x] Synchronize repository with interval
- [x] Decrypt targeted files by using SOPS
- [x] Expose RPC endpoint to be used by the kubernetes controller
- [x] Store existing repository in a state to when restarting krapao in order to not clone the repo again...
- [x] Authenticate with GCP provider
- [ ] Authenticate with AWS provider 
- [ ] Uses gpg key
- [ ] Add more unit test

## TODO for miwen

- [x] Create a CRD (gen)
- [x] Listen to Kubernetes events
- [x] Parse CRD and read kubernetes secrets
- [x] Update the status of the CRD when something need to be synchronize
- [x] Synchronize on CRD changes (by checking the generation_id which is different from the stored one)
- [x] Synchronize changes from git repository by calling the rpc endpoint from time to time
- [x] Add a limit to the number of synchronization status in the CRD
- [ ] Add more unit test