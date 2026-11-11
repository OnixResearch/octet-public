# Octet owns distribution packaging, not upstream product behavior. Every source
# directory is an unmodified, bounded Git archive selection admitted by Nickel.
{ pkgs, craneLib, root }:
let
  policy = builtins.fromJSON (builtins.readFile (root + "/vendor/dependencies.json"));
  sources = builtins.listToAttrs (map (source: {
    name = source.name;
    value = {
      outPath = root + "/${source.destination}";
      rev = source.revision;
    };
  }) policy.sources);
  overrideGitSource = packages: drv:
    let
      package = builtins.head packages;
      source = package.source;
      matching = builtins.filter (entry:
        source == "git+${entry.repository}?rev=${entry.revision}#${entry.revision}"
        || (entry.name == "artifact" && source == "git+ssh://git@github.com/OnixResearch/onix-artifact.git?rev=${entry.revision}#${entry.revision}")
      ) policy.sources;
    in
    if matching == [] then drv
    else drv.overrideAttrs (_: { src = sources.${(builtins.head matching).name}; });
  nickelExportSource = sources.nickel-export;
  nickelExportVendor = craneLib.vendorCargoDeps {
    src = nickelExportSource;
  };
  nickelExport = craneLib.buildPackage {
    pname = "nickel-export";
    version = "0.1.0";
    src = nickelExportSource;
    cargoVendorDir = nickelExportVendor;
    cargoExtraArgs = "-p nickel-export";
    doCheck = false;
    meta.license = pkgs.lib.licenses.agpl3Plus;
    postInstall = ''
      install -Dm644 ${nickelExportSource}/LICENSE "$out/share/licenses/nickel-export/LICENSE"
    '';
  };
  cairnSource = sources.cairn;
  cairnVendor = craneLib.vendorCargoDeps {
    src = cairnSource;
    overrideVendorGitCheckout = overrideGitSource;
  };
  cairn = craneLib.buildPackage {
    pname = "cairn";
    version = "0.1.0";
    src = cairnSource;
    cargoVendorDir = cairnVendor;
    cargoExtraArgs = "-p cairn-cli --features redb-registry";
    nativeBuildInputs = [ pkgs.pkg-config pkgs.makeWrapper pkgs.cmake ];
    buildInputs = [ pkgs.openssl pkgs.zlib ];
    doCheck = false;
    meta.license = pkgs.lib.licenses.agpl3Plus;
    postInstall = ''
      for binary in cairn ironspec; do
        wrapProgram "$out/bin/$binary" --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.nickel nickelExport ]}
      done
      mkdir -p "$out/share/licenses/cairn"
      cp -R ${cairnSource}/LICENSES ${cairnSource}/LICENSE-MIT ${cairnSource}/LICENSE-APACHE "$out/share/licenses/cairn/"
    '';
  };
in
{
  inherit sources overrideGitSource cairn;
}
