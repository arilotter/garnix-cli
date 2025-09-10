# garnix-cli

a 3rd party cli for https://garnix.io/ with some useful tools

```bash
garnix run [--as-branch BRANCH]
```

reads `garnix.yaml` config and builds matching nix flake attributes for your current git branch / the passed branch

## how to get

this repo is a flake u can

```nix
inputs = {
    garnix-cli = {
      url = "github:arilotter/garnix-cli";
      inputs.nixpkgs.follows = "nixpkgs";
    };
};
```

or just for the one time

```bash
nix run github:arilotter/garnix-cli -- run
```
