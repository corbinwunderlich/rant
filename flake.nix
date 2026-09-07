{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

    flake-parts.url = "github:hercules-ci/flake-parts";

    systems.url = "github:nix-systems/default";

    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    flake-parts,
    systems,
    git-hooks,
    ...
  } @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [git-hooks.flakeModule];

      systems = import systems;

      perSystem = {
        pkgs,
        config,
        ...
      }: {
        pre-commit.settings = {
          hooks = {
            alejandra.enable = true;

            deadnix.enable = true;

            statix.enable = true;

            taplo.enable = true;

            mdformat.enable = true;

            rustfmt.enable = true;

            clippy = {
              enable = true;
              settings = {
                extraArgs = "--fix --allow-dirty";
              };
            };
          };
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [cargo] ++ config.pre-commit.settings.enabledPackages;

          inherit (config.pre-commit) shellHook;
        };
      };
    };
}
