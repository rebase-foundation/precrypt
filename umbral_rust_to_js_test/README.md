This folder is for debugging an issue in serializing between rust and js. It shows how two scripts, one rust and the other js, can take the same inputs and encrypt capsules that are incompatible with the other script.

Running rust in /rust folder:
```
cargo install --path .
cargo run
```

Running js in /js folder:
```
yarn install 
node capsule_test.js
```
