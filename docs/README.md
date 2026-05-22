# openproteo docs

This directory is the [Docusaurus](https://docusaurus.io) site that
publishes to <https://sigilweaver.app/openproteo/>.

## Develop

```sh
bun install
bun run start    # local dev server with hot reload
bun run build    # static build into ./build
```

Pages live under [`docs/`](docs/); sidebar order is configured in
[`sidebars.ts`](sidebars.ts). The build is wired into CI via
`.github/workflows/ci.yml`.
