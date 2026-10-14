{
  description = "Checked-in consumer fixture for Octet Nix parity";
  inputs.octet.url = "path:@TIGERSTYLE_PATH@";
  outputs = { self, octet }: {
    checks.@SYSTEM@.default = octet.lib.mkConsumerCheck {
      system = "@SYSTEM@";
      src = ./.;
      cargoLock = ./Cargo.lock;
    };
  };
}
