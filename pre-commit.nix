{pkgs, ...}: let
  pc = "${pkgs.pre-commit}/bin/pre-commit";
in {
  config.devshell.startup.pre-commit.text = ''
      [ -e .git/hooks/pre-commit ] || (${pc} install --install-hooks && ${pc} install --hook-type commit-msg)
    '';
}
