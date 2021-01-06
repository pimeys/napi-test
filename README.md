# N-API test

Quickly testing how an async rust library could be used from JavaScript.

First, dependencies:

- [Install rust](https://rustup.rs/)
- Install napi cli: `npm install --global @napi-rs/cli`
- Build the rust crate: `npm run-script build`
- Copy the artifact: `npm run-script artifacts`
- Start SQL Server docker image (from Prisma engines)

Now, in the project root, you should have a file, such as
`index.linux-x64-gnu.node`. Copy this to `index.node`.

Run the script `node index.js`. It should output `1`, which is the output of SQL
Server query `SELECT 1`.
