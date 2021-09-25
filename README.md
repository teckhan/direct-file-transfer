## Starting Development

Start the app in the `dev` environment:

```bash
yarn start
```

## Packaging for Production

To package apps for the local platform:

```bash
yarn package
```

## TODO

- client ui
  - uploads + download list
  - download all as zip

- server
  - handle client cancel download?
  - zip multiple files
  - download multiple first without zip? (right now only download first)
  - change to port 80 once complete
  - clean up react icon and assets
  - turn on doUsePublicIpAddress for production
  - lint project
