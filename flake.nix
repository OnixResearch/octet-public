{
  description = "Octet — Custom lints enforcing Tiger Style coding principles under closed interfaces";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    cairn = {
      url = "github:OnixResearch/cairn-public/e25e782f8dafac98b906b48b9d15c965996e18f9";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    dylint-src = {
      url = "github:trailofbits/dylint/0e0a71eefe6f01563d11acc6e1d7af1d505934a9";
      flake = false;
    };
    valence = {
      url = "github:OnixResearch/valence-public/99cbe41f742035c37b6a7b503e4748e2e5ce2bea";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nickel-export = {
      url = "github:OnixResearch/nickel-export/257fafc1c746f1faf156207043a4c826bfb16d49";
      flake = false;
    };
    # r[impl octet.nickel_toolchain.cohort]
    nickel-1-17-0.url = "github:nickel-lang/nickel/1320a983e6c3d1e2fb53dd2464b084b4903b1426";
    aeneas = {
      url = "github:AeneasVerif/aeneas/bc2eb6f941369fa3c695e30598b5758b11e4f0de";
    };
    aeneasCharon = {
      url = "github:AeneasVerif/charon/cb50ff16b9f1066b8a97dc06da704de2da2fa41c";
      flake = false;
    };
    lean4-src = {
      url = "github:leanprover/lean4/68218e876d2a38b1985b8590fff244a83c321783";
      flake = false;
    };
    mathlib-src = {
      url = "github:leanprover-community/mathlib4/v4.31.0";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      rust-overlay,
      flake-utils,
      cairn,
      dylint-src,
      valence,
      nickel-export,
      nickel-1-17-0,
      aeneas,
      aeneasCharon,
      lean4-src,
      mathlib-src,
      ...
    }:
    let
      # Consumer-check helper — usable from any downstream flake.
      # Runs Octet lints against consumer code via nix flake check.
      mkConsumerCheck =
        {
          system,
          src,
          packages ? [ ],
          cargoExtraArgs ? null,
          cargoLock ? null,
          nativeBuildInputs ? [ ],
          buildInputs ? [ ],
        }:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          tsRustToolchain = pkgs.rust-bin.fromRustupToolchainFile "${self}/rust-toolchain.toml";
          tsHostTriple = pkgs.stdenv.hostPlatform.rust.rustcTarget;
          tsToolchainChannel =
            (builtins.fromTOML (builtins.readFile "${self}/rust-toolchain.toml")).toolchain.channel;
          tsRustupToolchain =
            if pkgs.lib.hasSuffix tsHostTriple tsToolchainChannel then
              tsToolchainChannel
            else
              "${tsToolchainChannel}-${tsHostTriple}";
          tsCraneLib = (crane.mkLib pkgs).overrideToolchain tsRustToolchain;
          tsRustToolchainLibPath = pkgs.lib.makeLibraryPath [ tsRustToolchain ];

          tsRustupShim = pkgs.writeShellScriptBin "rustup" ''
            toolchain="''${RUSTUP_TOOLCHAIN:-${tsRustupToolchain}}"
            case "$1" in
              which) which "$2" ;;
              show)
                if [ "$2" = "active-toolchain" ]; then
                  echo "$toolchain (default)"
                else
                  echo "$toolchain"
                fi
                ;;
              *) echo "rustup shim: unsupported: $*" >&2; exit 1 ;;
            esac
          '';
          tsCargoShim = pkgs.writeShellScriptBin "cargo" ''
            export RUSTUP_TOOLCHAIN="''${RUSTUP_TOOLCHAIN:-${tsRustupToolchain}}"
            exec ${tsRustToolchain}/bin/cargo "$@"
          '';
          tsDylintShimPath = pkgs.lib.makeBinPath [
            tsCargoShim
            tsRustupShim
          ];

          octetPkg = self.packages.${system}.default;
          driverPkg = self.packages.${system}.dylint-driver;
          sharedLibSuffix = if pkgs.stdenv.hostPlatform.isDarwin then ".dylib" else ".so";
          dllPrefix = if pkgs.stdenv.hostPlatform.isDarwin then "lib" else "lib";
          # Dylint parses the library filename as lib<name>@<toolchain>.<ext>
          octetDylintLibDir = pkgs.runCommand "octet-dylint-lib" { } ''
            mkdir -p $out/lib
            ln -s ${octetPkg}/lib/${dllPrefix}octet${sharedLibSuffix} \
              $out/lib/${dllPrefix}octet@${tsRustupToolchain}${sharedLibSuffix}
          '';
          octetDylintLibPath = "${octetDylintLibDir}/lib/${dllPrefix}octet@${tsRustupToolchain}${sharedLibSuffix}";

          checkNativeBuildInputs = nativeBuildInputs ++ [
            pkgs.pkg-config
            tsCargoShim
            tsRustupShim
          ];

          consumerVendorDir =
            if cargoLock != null then tsCraneLib.vendorCargoDeps { inherit cargoLock; } else null;

          packageArgs = builtins.concatStringsSep " " (map (p: "-p ${p}") packages);
          cargoCheckTail =
            if cargoExtraArgs != null && cargoExtraArgs != "" then " -- ${cargoExtraArgs}" else "";
        in
        pkgs.runCommand "octet-consumer-check"
          {
            nativeBuildInputs = checkNativeBuildInputs;
            inherit buildInputs;
            LD_LIBRARY_PATH = tsRustToolchainLibPath;
          }
          ''
            export PATH="${tsDylintShimPath}:$PATH"
            export HOME="$PWD/home"
            mkdir -p "$HOME"
            cp -r ${src} source
            chmod -R u+w source
            cd source
            export TIGERSTYLE_DRIVER_PATH="${driverPkg}/bin/dylint-driver"
            export TIGERSTYLE_LINT_LIB="${octetDylintLibPath}"
            export TIGERSTYLE_TOOLCHAIN="${tsRustupToolchain}"
            ${pkgs.lib.optionalString (consumerVendorDir != null) ''
              export CARGO_HOME="$PWD/.cargo-home"
              mkdir -p "$CARGO_HOME"
              cp "${consumerVendorDir}/config.toml" "$CARGO_HOME/config.toml"
            ''}
            ${self.packages.${system}.cargo-octet}/bin/cargo-octet check ${packageArgs}${cargoCheckTail}
            mkdir -p "$out"
          '';

      # Generated-policy-artifact consumer helper — usable from downstream
      # flakes when they want the Nickel-authored policy bundle exported by this
      # flake, then consumed by octet-standards in one reproducible check.
      mkPolicyArtifactConsumerCheck =
        {
          system,
          src,
          family,
          sourceSubdir ? ".",
          sourceSubdirs ? null,
          expectedDiagnostics ? 0,
          requiredPolicyIds ? [ ],
          extraArgs ? [ ],
          nativeBuildInputs ? [ ],
          buildInputs ? [ ],
        }:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          selectedSourceSubdirs = if sourceSubdirs == null then [ sourceSubdir ] else sourceSubdirs;
          sourceArgFor = subdir: if subdir == "." then "--source source" else "--source source/${subdir}";
          sourceArgs = builtins.concatStringsSep " " (map sourceArgFor selectedSourceSubdirs);
          extraCheckArgs = builtins.concatStringsSep " " extraArgs;
          requiredPolicyAssertions = builtins.concatStringsSep "\n" (
            map (policyId: ''
              jq --exit-status \
                'any(.policy_evaluations[]; .policy_id == "${policyId}")' \
                "$report_json" >/dev/null
            '') requiredPolicyIds
          );
        in
        pkgs.runCommand "octet-policy-artifact-consumer-check"
          {
            nativeBuildInputs = nativeBuildInputs ++ [
              pkgs.jq
              self.packages.${system}.octet-standards
            ];
            inherit buildInputs;
          }
          ''
            artifact_dir=$(mktemp -d)
            report_json=$(mktemp)

            ${self.packages.${system}.octet-policy-artifacts}/bin/octet-policy-artifacts "$artifact_dir"
            test -s "$artifact_dir/octet-policies.json"
            test -s "$artifact_dir/octet-check-config.toml"
            test -s "$artifact_dir/octet-seed-manifests.json"

            cp -r ${src} source
            chmod -R u+w source
            octet-standards check \
              --policy-artifacts-dir "$artifact_dir" \
              ${sourceArgs} \
              --family ${family} \
              --output json \
              ${extraCheckArgs} > "$report_json"

            jq --exit-status \
              '.diagnostics | length == ${builtins.toString expectedDiagnostics}' \
              "$report_json" >/dev/null
            ${requiredPolicyAssertions}
            mkdir -p "$out"
          '';
    in
    {
      lib = { inherit mkConsumerCheck mkPolicyArtifactConsumerCheck; };
    }
    // flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        nickel = nickel-1-17-0.packages.${system}.default;

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        miraiRevision = "aae0a56f91d9a0a62073bb86e7b2c9793cefc24b";
        miraiSource = pkgs.fetchFromGitHub {
          owner = "endorlabs";
          repo = "MIRAI";
          rev = miraiRevision;
          hash = "sha256-+H0JIbrwxMi3uMuxBqgVBzNkDYom8X616rH9y7YqIXg=";
          name = "endorlabs-mirai-source-${miraiRevision}";
        };
        miraiSourceProvider = pkgs.runCommand "octet-mirai-source-provider-1.1.12" { } ''
          mkdir -p "$out"
          ln -s ${miraiSource} "$out/source"
          cat > "$out/provider.json" <<'EOF'
          {
            "schema_version": "octet-mirai-source-provider/v1",
            "repository": "https://github.com/endorlabs/MIRAI",
            "revision": "${miraiRevision}",
            "license": "MIT",
            "version": "1.1.12",
            "rust_toolchain": "nightly-2025-01-10",
            "target": "x86_64-unknown-linux-gnu",
            "execution_admitted": false,
            "blocker_code": "stable-machine-output-unavailable"
          }
          EOF
        '';
        hostTriple = pkgs.stdenv.hostPlatform.rust.rustcTarget;
        toolchainChannel = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml)).toolchain.channel;
        rustupToolchain =
          if pkgs.lib.hasSuffix hostTriple toolchainChannel then
            toolchainChannel
          else
            "${toolchainChannel}-${hostTriple}";

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter =
            path: type:
            let
              inUiTests = pkgs.lib.hasSuffix "/ui_tests" path || pkgs.lib.hasInfix "/ui_tests/" path;
              inTestFixtures =
                pkgs.lib.hasSuffix "/tests/fixtures" path || pkgs.lib.hasInfix "/tests/fixtures/" path;
              inCairnCobPilot =
                pkgs.lib.hasSuffix "/fixtures/cairn-cob-pilot" path
                || pkgs.lib.hasInfix "/fixtures/cairn-cob-pilot/" path;
              inCairnCobConflictPilot =
                pkgs.lib.hasSuffix "/fixtures/cairn-cob-conflict-pilot" path
                || pkgs.lib.hasInfix "/fixtures/cairn-cob-conflict-pilot/" path;
              inCairnLifecycle = pkgs.lib.hasSuffix "/.cairn" path || pkgs.lib.hasInfix "/.cairn/" path;
              inDocs = pkgs.lib.hasSuffix "/docs" path || pkgs.lib.hasInfix "/docs/" path;
              inStandards = pkgs.lib.hasSuffix "/standards" path || pkgs.lib.hasInfix "/standards/" path;
              inTemplates = pkgs.lib.hasSuffix "/templates" path || pkgs.lib.hasInfix "/templates/" path;
              inHooks = pkgs.lib.hasSuffix "/hooks" path || pkgs.lib.hasInfix "/hooks/" path;
              inGitHub = pkgs.lib.hasSuffix "/.github" path || pkgs.lib.hasInfix "/.github/" path;
              isPreCommitHooks = pkgs.lib.hasSuffix "/.pre-commit-hooks.yaml" path;
              isReadme = builtins.baseNameOf path == "README.md";
            in
            (craneLib.filterCargoSources path type)
            || inUiTests
            || inTestFixtures
            || inCairnCobPilot
            || inCairnCobConflictPilot
            || inCairnLifecycle
            || inDocs
            || inStandards
            || inTemplates
            || inHooks
            || inGitHub
            || isPreCommitHooks
            || isReadme;
        };
        fmtSrc = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter =
            path: type: (craneLib.filterCargoSources path type) || builtins.baseNameOf path == "rustfmt.toml";
        };
        uiDriverSrc = ./tests/support;

        nativeBuildInputs = with pkgs; [
          clang
          git
          mold
          pkg-config
        ];

        buildInputs =
          with pkgs;
          [ openssl ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];

        # Dylint driver and UI harness still shell out through cargo/rustup.
        # These shims re-inject the toolchain name Dylint expects.
        rustup-shim = pkgs.writeShellScriptBin "rustup" ''
          toolchain="''${RUSTUP_TOOLCHAIN:-${rustupToolchain}}"
          case "$1" in
            which) which "$2" ;;
            show)
              if [ "$2" = "active-toolchain" ]; then
                echo "$toolchain (default)"
              else
                echo "$toolchain"
              fi
              ;;
            *) echo "rustup shim: unsupported: $*" >&2; exit 1 ;;
          esac
        '';

        cargo-shim = pkgs.writeShellScriptBin "cargo" ''
          export RUSTUP_TOOLCHAIN="''${RUSTUP_TOOLCHAIN:-${rustupToolchain}}"
          exec ${rustToolchain}/bin/cargo "$@"
        '';

        dylintShimPath = pkgs.lib.makeBinPath [
          cargo-shim
          rustup-shim
          rustToolchain
        ];
        rustToolchainLibPath = pkgs.lib.makeLibraryPath [ rustToolchain ];
        verusToolchainProfile = builtins.fromJSON (
          builtins.readFile ./standards/verus-toolchain/generated/profile.json
        );
        aeneasLeanProviderProfile = builtins.fromJSON (
          builtins.readFile ./standards/proof-rails/aeneas-lean/generated/profile.json
        );
        aeneasRevision = aeneasLeanProviderProfile.aeneas_revision;
        aeneasCharonRevision = aeneasLeanProviderProfile.charon_revision;
        aeneasProviderSource = aeneas.outPath;
        mathlibSource = mathlib-src;
        aeneasExecutable = aeneas.packages.${system}.aeneas;
        charonExecutable = aeneas.packages.${system}.charon;
        leantar = pkgs.stdenvNoCC.mkDerivation {
          pname = "leantar";
          version = "0.1.19";
          src = pkgs.fetchurl {
            url = "https://github.com/digama0/leangz/releases/download/v0.1.19/leantar-v0.1.19-x86_64-unknown-linux-musl.tar.gz";
            hash = "sha256-gbsl5DF5FTVZdEj0h+amMhoboMZuCEbAerwSrboTmjk=";
          };
          unpackPhase = "tar -xzf $src";
          installPhase = ''
            install -Dm755 leantar-v0.1.19-x86_64-unknown-linux-musl/leantar "$out/bin/leantar"
          '';
        };
        leanExecutable = pkgs.lean4.overrideAttrs (old: {
          version = "4.31.0";
          src = lean4-src;
          nativeBuildInputs = old.nativeBuildInputs ++ [ leantar ];
          cmakeFlags = [
            "-DUSE_GITHASH=OFF"
            "-DINSTALL_LICENSE=OFF"
            "-DUSE_MIMALLOC=OFF"
          ];
          postPatch = ''
            substituteInPlace src/CMakeLists.txt \
              --replace-fail 'set(GIT_SHA1 "")' 'set(GIT_SHA1 "v4.31.0")'
            rm -rf src/lake/examples/git/
          '';
          patches = [ ];
        });
        mathlibVersion = "4.31.0";
        approvedAeneasTheoremStatementBlake3 = "08e7c2d1fb1e21b2e7a174340a9f5b87c3acc634c591c6102624cd3a231f1125";
        stagedAeneasPreludeImportCount = 3;
        mathlibLakeHash = "sha256-wq4OBVgChJi59Widl+vuAPoq+MNZfFmdJwcsREW5RIo=";
        mathlibLakeDependencies = pkgs.stdenvNoCC.mkDerivation {
          pname = "octet-aeneas-mathlib-lake-dependencies";
          version = mathlibVersion;
          src = mathlibSource;
          nativeBuildInputs = [
            pkgs.cacert
            pkgs.gitMinimal
            pkgs.jq
          ];
          dontConfigure = true;
          buildPhase = ''
            runHook preBuild
            export HOME="$TMPDIR"
            export GIT_SSL_CAINFO="$NIX_SSL_CERT_FILE"
            mkdir -p "$TMPDIR/packages"
            jq --compact-output '.packages[] | select(.type == "git")' lake-manifest.json \
              | while IFS= read -r package; do
                package_name=$(echo "$package" | jq --raw-output '.name')
                package_url=$(echo "$package" | jq --raw-output '.url')
                package_revision=$(echo "$package" | jq --raw-output '.rev')
                git clone --filter=blob:none --no-checkout "$package_url" "$TMPDIR/packages/$package_name"
                git -C "$TMPDIR/packages/$package_name" checkout "$package_revision" --quiet
                rm -rf "$TMPDIR/packages/$package_name/.git"
              done
            runHook postBuild
          '';
          installPhase = ''
            runHook preInstall
            mv "$TMPDIR/packages" "$out"
            runHook postInstall
          '';
          dontFixup = true;
          outputHashMode = "recursive";
          outputHash = mathlibLakeHash;
        };
        productionVerusVersion = verusToolchainProfile.verifier.release;
        productionVerusSourceRevision = verusToolchainProfile.verifier.source_revision;
        productionVerusReleaseUrl = verusToolchainProfile.verifier.release_url;
        productionVerusReleaseSha256 = verusToolchainProfile.verifier.release_nix_sri;
        productionVerusReleaseSha256Hex = verusToolchainProfile.verifier.release_sha256_hex;
        productionVerusRustVersion = verusToolchainProfile.verifier.rust_version;
        productionVerusToolchainLabel = "${productionVerusRustVersion}-${hostTriple}";
        productionVerusRustToolchain = pkgs.rust-bin.stable.${productionVerusRustVersion}.default;
        productionVerusRustcLibPath = "${productionVerusRustToolchain}/lib";
        verusfmtAssets = builtins.listToAttrs (
          map (asset: {
            name = asset.target;
            value = asset;
          }) verusToolchainProfile.formatter.assets
        );
        verusfmtAsset = verusfmtAssets.${system} or null;
        verusfmtPolicyExitCode = 2;
        cargoOctetRuntimePath = pkgs.symlinkJoin {
          name = "cargo-octet-runtime-path";
          paths = [
            cargo-shim
            rustup-shim
            rustToolchain
            pkgs.stdenv.cc
            pkgs.clang
            pkgs.mold
            pkgs.ast-grep
            pkgs.cargo-llvm-cov
            pkgs.cargo-mutants
            octetVerus
            octetProductionVerus
          ];
        };
        checkNativeBuildInputs = nativeBuildInputs ++ [
          cargo-shim
          rustup-shim
          pkgs.git
          pkgs.python3
        ];
        sharedLibraryPrefix = if pkgs.stdenv.hostPlatform.isWindows then "" else "lib";
        sharedLibrarySuffix =
          if pkgs.stdenv.hostPlatform.isDarwin then
            ".dylib"
          else if pkgs.stdenv.hostPlatform.isWindows then
            ".dll"
          else
            ".so";
        octetLibraryName = "${sharedLibraryPrefix}octet${sharedLibrarySuffix}";
        octetUiLibraryName = "${sharedLibraryPrefix}octet@${rustupToolchain}${sharedLibrarySuffix}";

        valenceCutoverManifest = builtins.fromJSON (
          builtins.readFile ./standards/valence-core-cutover/generated-manifest.json
        );
        valenceCargoSource = valenceCutoverManifest.standalone_source.cargo_source;
        valenceRevision = valenceCutoverManifest.standalone_source.revision;
        valenceInputRevision = valence.rev;
        nickelExportCutoverManifest = builtins.fromJSON (
          builtins.readFile ./standards/nickel-export-cutover/generated-manifest.json
        );
        nickelExportCargoSource = nickelExportCutoverManifest.standalone_source.cargo_source;
        nickelExportRevision = nickelExportCutoverManifest.standalone_source.revision;
        nickelExportInputRevision = nickel-export.rev;
        cargoVendorDir = craneLib.vendorCargoDeps {
          inherit src;
          overrideVendorGitCheckout =
            packages: drv:
            if
              pkgs.lib.any (
                package: package.name == "valence-core" && package.source == valenceCargoSource
              ) packages
            then
              drv.overrideAttrs (_old: {
                src = valence;
              })
            else if
              pkgs.lib.any (
                package: package.name == "nickel-export-core" && package.source == nickelExportCargoSource
              ) packages
            then
              drv.overrideAttrs (_old: {
                src = nickel-export;
              })
            else
              drv;
        };
        workspaceVendorDir = cargoVendorDir;

        cargoArtifacts = craneLib.buildDepsOnly {
          inherit
            src
            nativeBuildInputs
            buildInputs
            cargoVendorDir
            ;
          RUSTUP_TOOLCHAIN = rustupToolchain;
        };

        octet = craneLib.buildPackage {
          inherit
            src
            cargoArtifacts
            nativeBuildInputs
            buildInputs
            cargoVendorDir
            ;
          doCheck = false;
        };

        cargoOctetUnwrapped = craneLib.buildPackage {
          inherit
            src
            cargoArtifacts
            nativeBuildInputs
            buildInputs
            cargoVendorDir
            ;
          pname = "cargo-octet";
          version = "0.1.0";
          cargoExtraArgs = "-p cargo-octet";
          doCheck = false;
        };

        cargoOctetFreshBuildA = craneLib.buildPackage {
          inherit
            src
            cargoArtifacts
            nativeBuildInputs
            buildInputs
            cargoVendorDir
            ;
          pname = "cargo-octet-fresh-build-a";
          version = "0.1.0";
          cargoExtraArgs = "-p cargo-octet";
          doCheck = false;
        };

        cargoOctetFreshBuildB = craneLib.buildPackage {
          inherit
            src
            cargoArtifacts
            nativeBuildInputs
            buildInputs
            cargoVendorDir
            ;
          pname = "cargo-octet-fresh-build-b";
          version = "0.1.0";
          cargoExtraArgs = "-p cargo-octet";
          doCheck = false;
        };

        fingerprintDocsDownstreamSmoke = pkgs.writeShellApplication {
          name = "octet-fingerprint-docs-downstream-smoke";
          runtimeInputs = [
            pkgs.coreutils
            pkgs.findutils
            pkgs.gnused
            pkgs.jq
          ];
          text = ''
            repo_root="''${1:-${src}}"
            scratch="$(mktemp -d)"
            trap 'rm -rf "$scratch"' EXIT

            mkdir -p "$scratch/consumer/src"
            cat > "$scratch/consumer/Cargo.toml" <<'EOF'
            [package]
            name = "octet-fingerprint-downstream-smoke"
            version = "0.1.0"
            edition = "2024"
            EOF
            cat > "$scratch/consumer/src/lib.rs" <<'EOF'
            pub mod human_summary;
            pub mod json_output;
            pub mod lifecycle;
            pub mod source_scanner;
            pub mod warning_summary;
            EOF

            cp "$repo_root/docs/examples/function-fingerprint-human-summary/reviewed.rs" "$scratch/consumer/src/human_summary.rs"
            cp "$repo_root/docs/examples/function-fingerprint-json-output/reviewed.rs" "$scratch/consumer/src/json_output.rs"
            cp "$repo_root/docs/examples/function-fingerprint-lifecycle/reviewed.rs" "$scratch/consumer/src/lifecycle.rs"
            cp "$repo_root/docs/examples/function-fingerprint-source-scanner/reviewed.rs" "$scratch/consumer/src/source_scanner.rs"
            cp "$repo_root/docs/examples/function-fingerprint-warning-summary/reviewed.rs" "$scratch/consumer/src/warning_summary.rs"
            chmod -R u+w "$scratch/consumer"

            set +e
            ${cargoOctetUnwrapped}/bin/cargo-octet fingerprint --check --output-format=json "$scratch/consumer/src" > "$scratch/check.json" 2> "$scratch/check.err"
            check_status="$?"
            set -e
            test "$check_status" -eq 1
            jq -e '.schema_version == 1 and .mode == "check"' "$scratch/check.json" > /dev/null
            jq -e '.records | length == 7' "$scratch/check.json" > /dev/null
            jq -e '[.records[].name] | index("bounded_retry_delay_ms") and index("capped_batch_size") and index("capped_retries") and index("describe_batch") and index("explicit_generated_boundary") and index("missing_gate_metadata") and index("unreviewed_macro_path")' "$scratch/check.json" > /dev/null
            jq -e '[.records[].name] | index("generated_boundary") | not' "$scratch/check.json" > /dev/null
            jq -e '[.records[] | select(.macro_warning == true)] | length == 1' "$scratch/check.json" > /dev/null
            jq -e '[.records[] | select(.review == null or .evidence == null)] | length == 1' "$scratch/check.json" > /dev/null
            jq -e '[.records[] | select(.status == "match")] | length == 7' "$scratch/check.json" > /dev/null

            sed -i 's/requested.min(5)/requested.min(8)/' "$scratch/consumer/src/json_output.rs"
            set +e
            ${cargoOctetUnwrapped}/bin/cargo-octet fingerprint --check --output-format=json "$scratch/consumer/src/json_output.rs" > "$scratch/drift.json" 2> "$scratch/drift.err"
            drift_status="$?"
            set -e
            test "$drift_status" -eq 1
            jq -e '.records | length == 1' "$scratch/drift.json" > /dev/null
            jq -e '.records[0].name == "capped_retries" and .records[0].status == "mismatch"' "$scratch/drift.json" > /dev/null

            set +e
            ${cargoOctetUnwrapped}/bin/cargo-octet fingerprint --diff "$scratch/consumer/src/json_output.rs" > "$scratch/diff.txt" 2> "$scratch/diff.err"
            diff_status="$?"
            ${cargoOctetUnwrapped}/bin/cargo-octet fingerprint --diff --output-format=json "$scratch/consumer/src/json_output.rs" > "$scratch/diff.json" 2> "$scratch/diff-json.err"
            diff_json_status="$?"
            set -e
            test "$diff_status" -eq 1
            test "$diff_json_status" -eq 1
            grep -F "octet fingerprint diff" "$scratch/diff.txt" > /dev/null
            grep -F "capped_retries mismatch" "$scratch/diff.txt" > /dev/null
            grep -F "summary total=1 match=0 mismatch=1 missing=0 written=0 macro_warnings=0 metadata_missing=0" "$scratch/diff.txt" > /dev/null
            jq -e '.schema_version == 1 and .mode == "diff"' "$scratch/diff.json" > /dev/null
            jq -e '.records | length == 1' "$scratch/diff.json" > /dev/null
            jq -e '.records[0].name == "capped_retries" and .records[0].status == "mismatch" and .records[0].macro_warning == false' "$scratch/diff.json" > /dev/null
            diff_human_name="$(grep -F "capped_retries mismatch" "$scratch/diff.txt" | sed -E 's/.* ([^ ]+) mismatch .*/\1/')"
            diff_json_name="$(jq -r '.records[0].name' "$scratch/diff.json")"
            test "$diff_human_name" = "$diff_json_name"

            ${cargoOctetUnwrapped}/bin/cargo-octet fingerprint --update --output-format=json "$scratch/consumer/src/json_output.rs" > "$scratch/update.json"
            jq -e '.records[0].name == "capped_retries" and .records[0].status == "written"' "$scratch/update.json" > /dev/null
            ${cargoOctetUnwrapped}/bin/cargo-octet fingerprint --check --output-format=json "$scratch/consumer/src/json_output.rs" > "$scratch/repaired.json"
            jq -e '.records[0].name == "capped_retries" and .records[0].status == "match"' "$scratch/repaired.json" > /dev/null
          '';
        };

        octetStandards = craneLib.buildPackage {
          inherit
            src
            cargoArtifacts
            nativeBuildInputs
            buildInputs
            cargoVendorDir
            ;
          pname = "octet-standards";
          version = "0.1.0";
          cargoExtraArgs = "-p octet-standards";
          doCheck = false;
        };

        octetStandardsNixClosureManifest =
          pkgs.runCommand "octet-standards-nix-closure-manifest"
            {
              nativeBuildInputs = [
                pkgs.b3sum
                pkgs.python3
              ];
              exportReferencesGraph = [
                "octet-standards-closure"
                octetStandards
              ];
            }
            ''
              mkdir -p "$out"
              cp octet-standards-closure "$out/octet-standards-closure.graph"
              GRAPH="$out/octet-standards-closure.graph" python3 - <<'PY' > "$out/octet-standards-closure.graph.canonical"
              import os
              from pathlib import Path
              text = Path(os.environ["GRAPH"]).read_text()
              lines = text.splitlines()
              index = 0
              records = []
              while index < len(lines):
                  while index < len(lines) and not lines[index].strip():
                      index += 1
                  if index >= len(lines):
                      break
                  path = lines[index].strip()
                  index += 1
                  while index < len(lines) and not lines[index].strip():
                      index += 1
                  count = int(lines[index].strip())
                  index += 1
                  refs = []
                  for _ in range(count):
                      while index < len(lines) and not lines[index].strip():
                          index += 1
                      refs.append(lines[index].strip())
                      index += 1
                  records.append((path, sorted(refs)))
              for path, refs in sorted(records):
                  print(path)
                  print(len(refs))
                  for ref in refs:
                      print(ref)
              PY
              graph_digest=$(b3sum "$out/octet-standards-closure.graph.canonical" | cut -d' ' -f1)
              printf '%s\n' "$graph_digest" > "$out/octet-standards-closure.graph.blake3"
              cat > "$out/proof-nix-closure.txt" <<EOF
              proof-verifier-nix-derivation: ${octetStandards.drvPath}
              proof-verifier-nix-output: ${octetStandards}
              proof-verifier-nix-closure: exportReferencesGraph:octet-standards-closure
              proof-verifier-nix-closure-graph-blake3: $graph_digest
              EOF
            '';

        octetVerus = pkgs.writeShellApplication {
          name = "octet-verus";
          runtimeInputs = with pkgs; [
            coreutils
            gnugrep
          ];
          text = ''
            identity="octet-verus@fixture-v1"
            digest="9f5a7f8c9e1d2c3b4a59687766554433221100ffeeddccbbaa99887766554433"
            pass_signal="proof-result: pass octet-addition-bound"

            if [ "''${1:-}" = "--identity" ]; then
              printf 'proof-verifier-mode: fixture\n'
              printf 'proof-verifier: %s\n' "$identity"
              printf 'proof-verifier-blake3: %s\n' "$digest"
              printf 'proof-verifier-binary: octet-verus\n'
              printf 'proof-verifier-binary-blake3: %s\n' "$digest"
              printf 'proof-toolchain: verus-fixture-wrapper/v1\n'
              printf 'proof-toolchain-blake3: %s\n' "$digest"
              printf 'trust-boundary: fixture verifier validates proof plumbing only\n'
              exit 0
            fi

            if [ "$#" -eq 0 ]; then
              echo "octet-verus: expected at least one proof file" >&2
              exit 2
            fi

            for proof in "$@"; do
              test -s "$proof"
              grep --fixed-strings 'verus!' "$proof" >/dev/null
              if grep --fixed-strings 'ensures false' "$proof" >/dev/null; then
                echo "octet-verus: rejected false postcondition in $proof" >&2
                exit 1
              fi
            done

            printf '%s\n' "$pass_signal"
          '';
        };

        aeneasLeanProviderClosure =
          pkgs.runCommand "octet-aeneas-lean-provider-closure-${system}"
            {
              nativeBuildInputs = with pkgs; [
                b3sum
                coreutils
                gnugrep
                gnused
                jq
              ];
            }
            ''
              set -eu
              revision_length=40
              staged_prelude_import_count=${toString stagedAeneasPreludeImportCount}
              aeneas_source="${aeneasProviderSource}"
              charon_source="${aeneasCharon}"
              mathlib_source="${mathlibSource}"
              mathlib_dependencies="${mathlibLakeDependencies}"
              mathlib_lake_hash="${mathlibLakeHash}"
              aeneas_binary="${aeneasExecutable}/bin/aeneas"
              charon_binary="${charonExecutable}/bin/charon"
              lean_binary="${leanExecutable}/bin/lean"
              lake_binary="${leanExecutable}/bin/lake"
              leantar_binary="${leantar}/bin/leantar"
              network_sandbox_binary="${pkgs.bubblewrap}/bin/bwrap"
              charon_pin="$aeneas_source/charon-pin"
              expected_aeneas_revision="${aeneasRevision}"
              actual_aeneas_revision="${aeneas.rev}"
              expected_charon_revision="${aeneasCharonRevision}"
              profile_path="${./standards/proof-rails/aeneas-lean/generated/profile.json}"

              if [ "$actual_aeneas_revision" != "$expected_aeneas_revision" ]; then
                echo "Aeneas revision mismatch: $actual_aeneas_revision != $expected_aeneas_revision" >&2
                exit 1
              fi
              if [ ! -s "$charon_pin" ]; then
                echo "Aeneas provider closure is missing charon-pin" >&2
                exit 1
              fi
              actual_charon_revision=$(grep --only-matching --extended-regexp "[0-9a-f]{$revision_length}" "$charon_pin" | head --lines=1)
              if [ "$actual_charon_revision" != "$expected_charon_revision" ]; then
                echo "Aeneas Charon revision mismatch: $actual_charon_revision != $expected_charon_revision" >&2
                exit 1
              fi
              if [ ! -d "$charon_source" ]; then
                echo "Aeneas provider closure is missing the pinned Charon source" >&2
                exit 1
              fi
              if [ ! -d "$mathlib_source" ]; then
                echo "Aeneas provider closure is missing the pinned mathlib source" >&2
                exit 1
              fi
              if [ ! -d "$mathlib_dependencies" ]; then
                echo "Aeneas provider closure is missing the pinned Mathlib Lake dependency closure" >&2
                exit 1
              fi
              for executable in "$aeneas_binary" "$charon_binary" "$lean_binary" "$lake_binary" "$leantar_binary" "$network_sandbox_binary"; do
                if [ ! -x "$executable" ]; then
                  echo "Aeneas provider closure is missing executable: $executable" >&2
                  exit 1
                fi
              done

              charon_pin_blake3=$(b3sum "$charon_pin" | cut --delimiter=' ' --fields=1)
              profile_blake3=$(b3sum "$profile_path" | cut --delimiter=' ' --fields=1)
              aeneas_binary_blake3=$(b3sum "$aeneas_binary" | cut --delimiter=' ' --fields=1)
              charon_binary_blake3=$(b3sum "$charon_binary" | cut --delimiter=' ' --fields=1)
              lean_binary_blake3=$(b3sum "$lean_binary" | cut --delimiter=' ' --fields=1)
              lake_binary_blake3=$(b3sum "$lake_binary" | cut --delimiter=' ' --fields=1)
              leantar_binary_blake3=$(b3sum "$leantar_binary" | cut --delimiter=' ' --fields=1)
              network_sandbox_blake3=$(b3sum "$network_sandbox_binary" | cut --delimiter=' ' --fields=1)
              provider_dir="$out/share/octet/aeneas-lean"
              mkdir -p "$out/bin" "$provider_dir"
              ln -s "$aeneas_source" "$provider_dir/aeneas-source"
              ln -s "$charon_source" "$provider_dir/charon-source"
              ln -s "$aeneas_binary" "$provider_dir/aeneas"
              ln -s "$charon_binary" "$provider_dir/charon"
              ln -s "$lean_binary" "$provider_dir/lean"
              ln -s "$lake_binary" "$provider_dir/lake"
              ln -s "$leantar_binary" "$provider_dir/leantar"
              ln -s "$network_sandbox_binary" "$provider_dir/bwrap"
              cp -r "$aeneas_source/backends/lean" "$provider_dir/aeneas-lean-library"
              chmod -R u+w "$provider_dir/aeneas-lean-library"
              cat > "$provider_dir/aeneas-lean-library/Aeneas.lean" <<'EOF'
              import Aeneas.Data
              import Aeneas.Do
              import Aeneas.Std.Scalar
              EOF
              mathlib_library="$provider_dir/aeneas-lean-library/mathlib"
              cp -r "$mathlib_source" "$mathlib_library"
              chmod -R u+w "$mathlib_library"
              find "$mathlib_library" -type l -delete
              mathlib_packages="$mathlib_library/.lake/packages"
              mkdir -p "$mathlib_packages"
              cp -r "$mathlib_dependencies/." "$mathlib_packages/"
              chmod -R u+w "$mathlib_packages"
              find "$mathlib_packages" -type l -delete
              test -z "$(find "$mathlib_library" -type l -print -quit)"
              sed -i '/require mathlib from git/,+1c\require mathlib from "./mathlib"' "$provider_dir/aeneas-lean-library/lakefile.lean"
              rm -f "$provider_dir/aeneas-lean-library/lake-manifest.json"
              dependency_overrides=$(
                for dependency in "$mathlib_packages"/*; do
                  dependency_name=$(basename "$dependency")
                  jq --null-input \
                    --arg name "$dependency_name" \
                    --arg dir "aeneas-lean-library/mathlib/.lake/packages/$dependency_name" \
                    '{ type: "path", name: $name, inherited: false, dir: $dir }'
                done | jq --slurp '.'
              )
              jq --null-input \
                --argjson dependencies "$dependency_overrides" \
                '{
                  schemaVersion: "1.2.0",
                  packages: ([
                    {
                      type: "path",
                      name: "aeneas",
                      inherited: false,
                      dir: "aeneas-lean-library"
                    },
                    {
                      type: "path",
                      name: "mathlib",
                      inherited: false,
                      dir: "aeneas-lean-library/mathlib"
                    }
                  ] + $dependencies)
                }' > "$provider_dir/lake-overrides.json"
              jq '{
                version: .schemaVersion,
                fixedToolchain: true,
                name: "octet_aeneas_fixture",
                lakeDir: ".lake",
                packagesDir: ".lake/packages",
                packages: .packages
              }' "$provider_dir/lake-overrides.json" > "$provider_dir/lake-manifest.json"
              mathlib_dependency_overrides=$(
                for dependency in "$mathlib_packages"/*; do
                  dependency_name=$(basename "$dependency")
                  jq --null-input \
                    --arg name "$dependency_name" \
                    --arg dir "aeneas-lean-library/mathlib/.lake/packages/$dependency_name" \
                    '{ type: "path", name: $name, inherited: false, dir: $dir }'
                done | jq --slurp '.'
              )
              jq --null-input \
                --argjson dependencies "$mathlib_dependency_overrides" \
                '{ schemaVersion: "1.2.0", packages: $dependencies }' \
                > "$mathlib_library/.lake/package-overrides.json"
              cp "$charon_pin" "$provider_dir/charon-pin"
              cp "$profile_path" "$provider_dir/profile.json"

              cat > "$provider_dir/identity.json" <<EOF
              {
                "schema_version": "octet-aeneas-lean-provider-closure/v1",
                "status": "packaged",
                "aeneas_revision": "$expected_aeneas_revision",
                "charon_revision": "$expected_charon_revision",
                "charon_pin_blake3": "$charon_pin_blake3",
                "profile_blake3": "$profile_blake3",
                "aeneas_binary_blake3": "$aeneas_binary_blake3",
                "charon_binary_blake3": "$charon_binary_blake3",
                "lean_binary_blake3": "$lean_binary_blake3",
                "lake_binary_blake3": "$lake_binary_blake3",
                "leantar_binary_blake3": "$leantar_binary_blake3",
                "network_sandbox_blake3": "$network_sandbox_blake3",
                "mathlib_lake_hash": "$mathlib_lake_hash",
                "mathlib_lake_dependencies": "$mathlib_dependencies",
                "aeneas_source": "$provider_dir/aeneas-source",
                "charon_source": "$provider_dir/charon-source",
                "aeneas_binary": "$provider_dir/aeneas",
                "charon_binary": "$provider_dir/charon",
                "lean_binary": "$provider_dir/lean",
                "lake_binary": "$provider_dir/lake",
                "leantar_binary": "$provider_dir/leantar",
                "network_sandbox": "$provider_dir/bwrap",
                "aeneas_lean_library": "$provider_dir/aeneas-lean-library",
                "mathlib_source": "$mathlib_library",
                "lake_overrides": "$provider_dir/lake-overrides.json",
                "lake_manifest": "$provider_dir/lake-manifest.json",
                "charon_pin": "$provider_dir/charon-pin",
                "profile": "$provider_dir/profile.json"
              }
              EOF

              cat > "$out/bin/octet-aeneas-lean" <<EOF
              #!${pkgs.runtimeShell}
              set -eu
              revision_length=$revision_length
              expected_charon_revision="$expected_charon_revision"
              identity_path="$provider_dir/identity.json"
              provider_dir="$provider_dir"

              check_charon_pin() {
                pin_file="\$1"
                actual_revision="\$(grep --only-matching --extended-regexp "[0-9a-f]{\$revision_length}" "\$pin_file" | head --lines=1 || true)"
                if [ "\$actual_revision" != "\$expected_charon_revision" ]; then
                  echo "Aeneas Charon revision mismatch: \$actual_revision != \$expected_charon_revision" >&2
                  return 1
                fi
              }

              check_charon_pin "\$provider_dir/charon-pin"
              if [ "\$#" -eq 1 ] && [ "\$1" = "--identity" ]; then
                cat "\$identity_path"
                exit 0
              fi
              if [ "\$#" -eq 3 ] && [ "\$1" = "--check-charon-pin" ]; then
                check_charon_pin "\$2"
                printf 'charon-pin: valid\\n'
                exit 0
              fi
              if [ "\$#" -eq 1 ] && [ "\$1" = "--tool-identities" ]; then
                jq --sort-keys '{aeneas_binary, charon_binary, lean_binary, lake_binary, leantar_binary, network_sandbox, aeneas_binary_blake3, charon_binary_blake3, lean_binary_blake3, lake_binary_blake3, leantar_binary_blake3, network_sandbox_blake3}' "\$identity_path"
                exit 0
              fi
              echo "Aeneas Lean execution requires the cargo-octet execution shell" >&2
              exit 2
              EOF
              chmod +x "$out/bin/octet-aeneas-lean"
            '';

        aeneasLeanProviderClosureCheck =
          pkgs.runCommand "octet-aeneas-lean-provider-closure-check"
            {
              nativeBuildInputs = with pkgs; [
                coreutils
                gnugrep
                jq
              ];
            }
            ''
              set -eu
              identity="${aeneasLeanProviderClosure}/share/octet/aeneas-lean/identity.json"
              mathlib_library="${aeneasLeanProviderClosure}/share/octet/aeneas-lean/aeneas-lean-library/mathlib"
              expected_charon_revision="${aeneasCharonRevision}"
              test -d "$mathlib_library"
              test -x "${aeneasLeanProviderClosure}/share/octet/aeneas-lean/leantar"
              test -z "$(find "$mathlib_library" -type l -print -quit)"
              test ! -e "${aeneasLeanProviderClosure}/share/octet/aeneas-lean/aeneas-lean-library/lake-manifest.json"
              grep --fixed-strings 'require mathlib from "./mathlib"' \
                "${aeneasLeanProviderClosure}/share/octet/aeneas-lean/aeneas-lean-library/lakefile.lean" >/dev/null
              staged_prelude="${aeneasLeanProviderClosure}/share/octet/aeneas-lean/aeneas-lean-library/Aeneas.lean"
              staged_prelude_import_count=${toString stagedAeneasPreludeImportCount}
              test "$(grep --count '^import ' "$staged_prelude")" -eq "$staged_prelude_import_count"
              grep --fixed-strings 'import Aeneas.Data' "$staged_prelude" >/dev/null
              grep --fixed-strings 'import Aeneas.Do' "$staged_prelude" >/dev/null
              grep --fixed-strings 'import Aeneas.Std.Scalar' "$staged_prelude" >/dev/null
              if grep --extended-regexp '^import Aeneas\.(Command|Extract|Std|Tactic)$' "$staged_prelude" >/dev/null; then
                echo "Aeneas provider closure unexpectedly includes an unsupported umbrella import" >&2
                exit 1
              fi
              jq --exit-status \
                '.schemaVersion == "1.2.0"
                  and ([.packages[].type] | all(. == "path"))
                  and ([.packages[].name] | index("aeneas") != null)
                  and ([.packages[].name] | index("mathlib") != null)
                  and ([.packages[].name] | index("batteries") != null)' \
                "${aeneasLeanProviderClosure}/share/octet/aeneas-lean/lake-overrides.json" >/dev/null
              jq --exit-status \
                '.version == "1.2.0"
                  and .fixedToolchain == true
                  and .name == "octet_aeneas_fixture"
                  and ([.packages[].type] | all(. == "path"))
                  and ([.packages[].name] | index("aeneas") != null)' \
                "${aeneasLeanProviderClosure}/share/octet/aeneas-lean/lake-manifest.json" >/dev/null
              jq --exit-status \
                '.schemaVersion == "1.2.0"
                  and ([.packages[].type] | all(. == "path"))
                  and ([.packages[].name] | index("batteries") != null)' \
                "$mathlib_library/.lake/package-overrides.json" >/dev/null
              jq --exit-status \
                --arg aeneas_revision "${aeneasRevision}" \
                --arg charon_revision "$expected_charon_revision" \
                '.schema_version == "octet-aeneas-lean-provider-closure/v1" and .status == "packaged" and .aeneas_revision == $aeneas_revision and .charon_revision == $charon_revision and (.charon_pin_blake3 | length) == 64 and (.profile_blake3 | length) == 64 and (.aeneas_binary_blake3 | length) == 64 and (.charon_binary_blake3 | length) == 64 and (.lean_binary_blake3 | length) == 64 and (.lake_binary_blake3 | length) == 64 and (.leantar_binary_blake3 | length) == 64 and (.aeneas_binary | length) > 0 and (.charon_binary | length) > 0 and (.lean_binary | length) > 0 and (.lake_binary | length) > 0 and (.leantar_binary | length) > 0 and (.network_sandbox_blake3 | length) == 64 and (.network_sandbox | length) > 0 and (.mathlib_lake_hash | length) > 0 and (.mathlib_lake_dependencies | length) > 0 and (.mathlib_source | length) > 0 and (.lake_overrides | length) > 0 and (.lake_manifest | length) > 0' \
                "$identity" >/dev/null
              "${aeneasLeanProviderClosure}/bin/octet-aeneas-lean" --identity > "$TMPDIR/identity.json"
              cmp "$identity" "$TMPDIR/identity.json"
              "${aeneasLeanProviderClosure}/bin/octet-aeneas-lean" --tool-identities > "$TMPDIR/tool-identities.json"
              jq --exit-status '([.aeneas_binary, .charon_binary, .lean_binary, .lake_binary, .leantar_binary, .network_sandbox] | all(length > 0)) and ([.aeneas_binary_blake3, .charon_binary_blake3, .lean_binary_blake3, .lake_binary_blake3, .leantar_binary_blake3, .network_sandbox_blake3] | all(length == 64))' "$TMPDIR/tool-identities.json" >/dev/null

              positive_pin="$TMPDIR/charon-pin-positive"
              printf '# reviewed pin\\n%s\\n' "$expected_charon_revision" > "$positive_pin"
              "${aeneasLeanProviderClosure}/bin/octet-aeneas-lean" --check-charon-pin "$positive_pin" "$expected_charon_revision" \
                | grep --fixed-strings 'charon-pin: valid' >/dev/null

              negative_pin="$TMPDIR/charon-pin-negative"
              printf '# drifted pin\\n%s\\n' '0000000000000000000000000000000000000000' > "$negative_pin"
              set +e
              "${aeneasLeanProviderClosure}/bin/octet-aeneas-lean" --check-charon-pin "$negative_pin" "$expected_charon_revision" > "$TMPDIR/negative.out" 2>&1
              negative_status="$?"
              set -e
              test "$negative_status" -eq 1
              grep --fixed-strings 'Aeneas Charon revision mismatch' "$TMPDIR/negative.out" >/dev/null
              mkdir -p "$out"
              cp "$TMPDIR/identity.json" "$out/identity.json"
              cp "$TMPDIR/negative.out" "$out/negative.out"
            '';

        octetProductionVerus = pkgs.stdenvNoCC.mkDerivation {
          pname = "octet-production-verus";
          version = productionVerusVersion;
          src = pkgs.fetchurl {
            url = productionVerusReleaseUrl;
            hash = productionVerusReleaseSha256;
          };
          nativeBuildInputs = with pkgs; [
            autoPatchelfHook
            b3sum
            makeWrapper
            unzip
          ];
          buildInputs = with pkgs; [
            stdenv.cc.cc.lib
            zlib
            productionVerusRustToolchain
          ];
          unpackPhase = ''
            unzip "$src"
          '';
          installPhase = ''
            runHook preInstall
            mkdir -p "$out/libexec/verus" "$out/bin" "$out/share"
            cp -R verus-x86-linux/. "$out/libexec/verus/"
            chmod -R u+w "$out/libexec/verus"
            find "$out/libexec/verus" -type f \( -name verus -o -name rust_verify -o -name z3 \) -exec chmod +x {} +

            rustup_shim="$out/libexec/verus/rustup"
            cat > "$rustup_shim" <<'EOF'
            #!${pkgs.runtimeShell}
            set -eu
            case "''${1:-}" in
              --version|-V)
                printf 'rustup 1.28.2 (octet shim)\n'
                ;;
              toolchain)
                if [ "''${2:-}" = "list" ]; then
                  printf '${productionVerusToolchainLabel} (default)\n'
                else
                  echo "octet rustup shim: unsupported toolchain ''${2:-}" >&2
                  exit 1
                fi
                ;;
              run)
                requested="''${2:-}"
                shift 2
                if [ "''${1:-}" = "--" ]; then
                  shift
                fi
                case "$requested" in
                  ${productionVerusToolchainLabel}|${productionVerusRustVersion}) ;;
                  *) echo "octet rustup shim: unsupported toolchain $requested" >&2; exit 1 ;;
                esac
                export LD_LIBRARY_PATH="${productionVerusRustcLibPath}:''${LD_LIBRARY_PATH:-}"
                exec "$@"
                ;;
              which)
                case "''${2:-}" in
                  rustc) printf '%s/bin/rustc\n' "${productionVerusRustToolchain}" ;;
                  cargo) printf '%s/bin/cargo\n' "${productionVerusRustToolchain}" ;;
                  *) echo "octet rustup shim: unsupported which ''${2:-}" >&2; exit 1 ;;
                esac
                ;;
              show)
                if [ "''${2:-}" = "active-toolchain" ]; then
                  printf '${productionVerusToolchainLabel} (default)\n'
                else
                  printf '${productionVerusToolchainLabel}\n'
                fi
                ;;
              *) echo "octet rustup shim: unsupported: $*" >&2; exit 1 ;;
            esac
            EOF
            chmod +x "$rustup_shim"

            makeWrapper "$out/libexec/verus/verus" "$out/bin/verus" \
              --prefix PATH : "$out/libexec/verus" \
              --prefix LD_LIBRARY_PATH : "${productionVerusRustcLibPath}"

            verus_binary_blake3="$(b3sum "$out/libexec/verus/verus")"
            verus_binary_blake3="''${verus_binary_blake3%% *}"
            solver_blake3="$(b3sum "$out/libexec/verus/z3")"
            solver_blake3="''${solver_blake3%% *}"
            toolchain_blake3="$(b3sum "${productionVerusRustToolchain}/bin/rustc")"
            toolchain_blake3="''${toolchain_blake3%% *}"

            cat > "$out/share/identity.txt" <<EOF
            proof-verifier-mode: production
            proof-verifier: verus@${productionVerusVersion}
            proof-verifier-source-revision: ${productionVerusSourceRevision}
            proof-verifier-release-sha256: ${productionVerusReleaseSha256Hex}
            proof-verifier-binary: verus
            proof-verifier-binary-blake3: $verus_binary_blake3
            proof-toolchain: ${productionVerusToolchainLabel}
            proof-toolchain-blake3: $toolchain_blake3
            proof-solver: z3@verus-${productionVerusVersion}
            proof-solver-blake3: $solver_blake3
            trust-boundary: production Verus binary release verifies the declared proof target only
            EOF

            production_wrapper="$out/bin/octet-production-verus"
            cat > "$production_wrapper" <<EOF
            #!${pkgs.runtimeShell}
            set -eu
            if [ "\''${1:-}" = "--identity" ]; then
              cat "$out/share/identity.txt"
              exit 0
            fi
            exec "$out/bin/verus" "\$@"
            EOF
            chmod +x "$production_wrapper"
            runHook postInstall
          '';
          passthru = {
            inherit
              productionVerusReleaseSha256Hex
              productionVerusSourceRevision
              productionVerusToolchainLabel
              productionVerusVersion
              ;
          };
        };

        octetVerusfmt =
          if verusfmtAsset == null then
            pkgs.runCommand "octet-verusfmt-unsupported-${system}" { } ''
              mkdir -p "$out/bin" "$out/share"
              cat > "$out/share/identity.txt" <<EOF
              formatter-profile: ${verusToolchainProfile.profile_id}
              formatter: verusfmt@${verusToolchainProfile.formatter.version}
              formatter-target: ${system}
              formatter-status: unsupported
              trust-boundary: formatter availability and formatting are not proof evidence
              EOF
              cat > "$out/bin/verusfmt" <<'EOF'
              #!${pkgs.runtimeShell}
              echo "verusfmt: no pinned release asset for this target" >&2
              exit ${toString verusfmtPolicyExitCode}
              EOF
              cat > "$out/bin/octet-verusfmt" <<EOF
              #!${pkgs.runtimeShell}
              if [ "\$#" -eq 1 ] && [ "\$1" = "--identity" ]; then
                cat "$out/share/identity.txt"
                exit 0
              fi
              exec "$out/bin/verusfmt" "\$@"
              EOF
              chmod +x "$out/bin/verusfmt" "$out/bin/octet-verusfmt"
            ''
          else
            pkgs.stdenvNoCC.mkDerivation {
              pname = "octet-verusfmt";
              version = verusToolchainProfile.formatter.version;
              src = pkgs.fetchurl {
                url = "https://github.com/verus-lang/verusfmt/releases/download/${verusToolchainProfile.formatter.release_tag}/${verusfmtAsset.file_name}";
                hash = verusfmtAsset.nix_sri;
              };
              nativeBuildInputs = with pkgs; [
                autoPatchelfHook
                b3sum
                gnutar
                xz
              ];
              buildInputs = [ pkgs.stdenv.cc.cc.lib ];
              dontUnpack = true;
              installPhase = ''
                runHook preInstall
                release_dir="$out/libexec/verusfmt-release"
                mkdir -p "$release_dir" "$out/bin" "$out/share"
                tar --extract --xz --file "$src" --strip-components=1 --directory "$release_dir"
                mv "$release_dir/verusfmt" "$release_dir/verusfmt-real"
                chmod +x "$release_dir/verusfmt-real"

                cat > "$out/bin/verusfmt" <<'EOF'
                #!${pkgs.runtimeShell}
                set -eu
                if [ "$#" -gt 0 ] && [ "$1" = "--update" ]; then
                  echo "verusfmt self-update is disabled; update the Octet Nickel profile and Nix hash instead" >&2
                  exit ${toString verusfmtPolicyExitCode}
                fi
                exec "@REAL_VERUSFMT@" "$@"
                EOF
                substituteInPlace "$out/bin/verusfmt" \
                  --replace-fail "@REAL_VERUSFMT@" "$release_dir/verusfmt-real"
                chmod +x "$out/bin/verusfmt"

                formatter_blake3="$(b3sum "$release_dir/verusfmt-real")"
                formatter_blake3="''${formatter_blake3%% *}"
                cat > "$out/share/identity.txt" <<EOF
                formatter-profile: ${verusToolchainProfile.profile_id}
                formatter: verusfmt@${verusToolchainProfile.formatter.version}
                formatter-source-revision: ${verusToolchainProfile.formatter.source_revision}
                formatter-target: ${system}
                formatter-release-sha256: ${verusfmtAsset.sha256_hex}
                formatter-binary-blake3: $formatter_blake3
                formatter-self-update: disabled-by-wrapper
                trust-boundary: formatting proves selected source shape only, not proof success or correctness
                EOF

                cat > "$out/bin/octet-verusfmt" <<EOF
                #!${pkgs.runtimeShell}
                set -eu
                if [ "\$#" -eq 1 ] && [ "\$1" = "--identity" ]; then
                  cat "$out/share/identity.txt"
                  exit 0
                fi
                exec "$out/bin/verusfmt" "\$@"
                EOF
                chmod +x "$out/bin/octet-verusfmt"
                runHook postInstall
              '';
            };

        octetVerusToolchainProfile =
          pkgs.runCommand "octet-verus-toolchain-profile"
            {
              nativeBuildInputs = with pkgs; [
                b3sum
                diffutils
                nickel
              ];
            }
            ''
              profile_dir=${./standards/verus-toolchain}
              mkdir -p "$out/share/octet/verus-toolchain"
              cd "$profile_dir"
              nickel export --format json profile.ncl > "$TMPDIR/profile.json"
              diff --unified generated/profile.json "$TMPDIR/profile.json"
              cp contracts.ncl "$out/share/octet/verus-toolchain/contracts.ncl"
              cp profile.ncl "$out/share/octet/verus-toolchain/profile.ncl"
              cp "$TMPDIR/profile.json" "$out/share/octet/verus-toolchain/profile.json"
              profile_blake3="$(b3sum "$TMPDIR/profile.json")"
              profile_blake3="''${profile_blake3%% *}"
              printf '%s\n' "$profile_blake3" > "$out/share/octet/verus-toolchain/profile.blake3"
            '';

        verusToolchainProfileCheck =
          pkgs.runCommand "octet-verus-toolchain-profile-check"
            {
              nativeBuildInputs =
                with pkgs;
                [
                  diffutils
                  findutils
                  nickel
                ]
                ++ [ octetVerusfmt ];
            }
            ''
              profile_dir=${./standards/verus-toolchain}
              cd "$profile_dir"
              nickel export --format json profile.ncl > "$TMPDIR/profile.json"
              diff --unified generated/profile.json "$TMPDIR/profile.json"
              nickel export --format json fixtures/valid/current_profile.ncl > /dev/null
              for invalid_profile in fixtures/invalid/*.ncl; do
                if nickel export --format json "$invalid_profile" > /dev/null 2>&1; then
                  echo "invalid Verus toolchain profile unexpectedly passed: $invalid_profile" >&2
                  exit 1
                fi
              done
              ${pkgs.lib.optionalString (verusfmtAsset != null) ''
                verusfmt --check --verus-only ${./standards/verus-toolchain/fixtures/valid/formatted.rs}
                if verusfmt --check --verus-only ${./standards/verus-toolchain/fixtures/invalid/unformatted.rs} > /dev/null 2>&1; then
                  echo "unformatted Verus fixture unexpectedly passed" >&2
                  exit 1
                fi
                while IFS= read -r verus_source; do
                  verusfmt --check --verus-only "$verus_source"
                done < <(find ${./verus} -type f -name '*.rs' -print | LC_ALL=C sort)
                if verusfmt --update > /dev/null 2>&1; then
                  echo "pinned verusfmt wrapper unexpectedly allowed self-update" >&2
                  exit 1
                fi
                octet-verusfmt --identity | grep -F "formatter: verusfmt@${verusToolchainProfile.formatter.version}" > /dev/null
              ''}
              mkdir -p "$out"
            '';

        octetProductionVerusNixClosureManifest =
          pkgs.runCommand "octet-production-verus-nix-closure-manifest"
            {
              nativeBuildInputs = [
                pkgs.b3sum
                pkgs.python3
              ];
              exportReferencesGraph = [
                "octet-production-verus-closure"
                octetProductionVerus
              ];
            }
            ''
              mkdir -p "$out"
              cp octet-production-verus-closure "$out/octet-production-verus-closure.graph"
              python3 - "$out/octet-production-verus-closure.graph" "$out/octet-production-verus-closure.graph.canonical" <<'PY'
              import json
              import sys

              source, dest = sys.argv[1:]
              records = []
              with open(source, "r", encoding="utf-8") as handle:
                  lines = [line.rstrip("\n") for line in handle]
              index = 0
              while index < len(lines):
                  path = lines[index]
                  index += 1
                  if not path:
                      continue
                  deriver = lines[index]
                  index += 1
                  refs_count = int(lines[index])
                  index += 1
                  refs = sorted(lines[index:index + refs_count])
                  index += refs_count
                  records.append({"path": path, "deriver": deriver, "refs": refs})
              records.sort(key=lambda record: record["path"])
              with open(dest, "w", encoding="utf-8") as handle:
                  for record in records:
                      handle.write(json.dumps(record, sort_keys=True, separators=(",", ":")))
                      handle.write("\n")
              PY
              graph_digest=$(b3sum "$out/octet-production-verus-closure.graph.canonical" | cut -d' ' -f1)
              printf '%s\n' "$graph_digest" > "$out/octet-production-verus-closure.graph.blake3"
              cat > "$out/proof-nix-closure.txt" <<EOF
              proof-verifier-nix-output: ${octetProductionVerus}
              proof-verifier-nix-closure: exportReferencesGraph:octet-production-verus-closure
              proof-verifier-nix-closure-graph-blake3: $graph_digest
              EOF
            '';

        uiDriverVendorDir = craneLib.vendorCargoDeps {
          src = uiDriverSrc;
          cargoLock = ./tests/support/ui-driver/Cargo.lock;
        };

        uiDriver = pkgs.stdenv.mkDerivation {
          pname = "octet-ui-driver";
          version = "0.1.0";
          src = uiDriverSrc;
          nativeBuildInputs = checkNativeBuildInputs ++ [ rustToolchain ];
          inherit buildInputs;
          LD_LIBRARY_PATH = rustToolchainLibPath;
          RUSTUP_TOOLCHAIN = rustupToolchain;
          dontConfigure = true;

          buildPhase = ''
            runHook preBuild
            export PATH="${dylintShimPath}:$PATH"
            export CARGO_HOME="$PWD/.cargo-home"
            mkdir -p "$CARGO_HOME"
            cp "${uiDriverVendorDir}/config.toml" "$CARGO_HOME/config.toml"
            cargo build --release --locked --offline --manifest-path ui-driver/Cargo.toml
            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall
            mkdir -p "$out/bin"
            driver_bin=""
            for candidate in \
              target/release/octet-ui-driver \
              target/release/octet_ui_driver \
              ui-driver/target/release/octet-ui-driver \
              ui-driver/target/release/octet_ui_driver
            do
              if [ -f "$candidate" ]; then
                driver_bin="$candidate"
                break
              fi
            done
            if [ -z "$driver_bin" ]; then
              echo "missing Octet UI driver binary" >&2
              find . -maxdepth 4 \( -name 'octet-ui-driver' -o -name 'octet_ui_driver' \) -print >&2 || true
              exit 1
            fi
            cp "$driver_bin" "$out/bin/octet-ui-driver"
            runHook postInstall
          '';
        };

        octetUiLibrary = pkgs.runCommand "octet-ui-library" { } ''
          mkdir -p "$out/lib"
          ln -s "${octet}/lib/${octetLibraryName}" "$out/lib/${octetUiLibraryName}"
        '';

        # Dylint driver binary — used by mkConsumerCheck to run lints.
        # Patches the upstream driver source to add a binary target, then
        # builds offline with vendored deps.
        dylintDriverSrc = pkgs.runCommand "dylint-driver-src" { } ''
                    cp -r ${dylint-src} $out
                    chmod -R u+w $out
                    cat > $out/driver/src/main.rs <<'MAIN'
          #![feature(rustc_private)]
          fn main() {
              let args: Vec<_> = std::env::args_os().collect();
              dylint_driver::dylint_driver(&args).unwrap_or_else(|e| {
                  eprintln!("dylint-driver: {e:#}");
                  std::process::exit(1);
              });
          }
          MAIN
        '';

        dylintDriverVendorDir = craneLib.vendorCargoDeps {
          cargoLock = "${dylint-src}/driver/Cargo.lock";
        };

        dylintDriver = pkgs.stdenv.mkDerivation {
          pname = "dylint-driver";
          version = "5.0.0";
          src = dylintDriverSrc;
          nativeBuildInputs = checkNativeBuildInputs ++ [ rustToolchain ];
          buildInputs =
            buildInputs
            ++ (with pkgs; [
              curl
              libgit2
            ]);
          LD_LIBRARY_PATH = rustToolchainLibPath;
          # Setting RUSTUP_TOOLCHAIN to "nightly-<triple>" (without date)
          # makes the build.rs skip cloning the clippy repo for extra symbols
          # while still parsing as a valid toolchain string.
          RUSTUP_TOOLCHAIN = "nightly-${hostTriple}";
          LIBGIT2_SYS_USE_PKG_CONFIG = "1";
          dontConfigure = true;

          buildPhase = ''
            runHook preBuild
            export PATH="${dylintShimPath}:$PATH"
            export CARGO_HOME="$PWD/.cargo-home"
            mkdir -p "$CARGO_HOME"
            cp "${dylintDriverVendorDir}/config.toml" "$CARGO_HOME/config.toml"
            cd driver
            cargo build --release --offline
            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall
            mkdir -p "$out/bin"
            driver_bin=$(find .. -name 'dylint_driver' -type f -executable | head -1)
            if [ -z "$driver_bin" ]; then
              echo "dylint_driver binary not found" >&2
              exit 1
            fi
            cp "$driver_bin" "$out/bin/dylint-driver"
            runHook postInstall
          '';
        };

        cargoOctet = pkgs.symlinkJoin {
          name = "cargo-octet-0.1.0";
          paths = [ cargoOctetUnwrapped ];
          nativeBuildInputs = [ pkgs.makeWrapper ];
          postBuild = ''
            wrapProgram "$out/bin/cargo-octet" \
              --prefix PATH : "${cargoOctetRuntimePath}/bin" \
              --prefix LD_LIBRARY_PATH : "${rustToolchainLibPath}" \
              --set TIGERSTYLE_TOOLCHAIN "${rustupToolchain}" \
              --set TIGERSTYLE_DRIVER_PATH "${dylintDriver}/bin/dylint-driver" \
              --set TIGERSTYLE_LINT_LIB "${octetUiLibrary}/lib/${octetUiLibraryName}" \
              --set OCTET_COVERAGE_CARGO "${cargo-shim}/bin/cargo" \
              --set OCTET_COVERAGE_PATH "${cargoOctetRuntimePath}/bin" \
              --set OCTET_COVERAGE_CARGO_CONFIG "${cargoVendorDir}/config.toml" \
              --set OCTET_COVERAGE_LD_LIBRARY_PATH "${rustToolchainLibPath}" \
              --set OCTET_MUTATION_CARGO "${cargo-shim}/bin/cargo" \
              --set OCTET_MUTATION_PATH "${cargoOctetRuntimePath}/bin" \
              --set OCTET_MUTATION_CARGO_CONFIG "${cargoVendorDir}/config.toml" \
              --set OCTET_MUTATION_LD_LIBRARY_PATH "${rustToolchainLibPath}" \
              --set OCTET_HEGEL_CARGO "${cargo-shim}/bin/cargo" \
              --set OCTET_HEGEL_PATH "${cargoOctetRuntimePath}/bin" \
              --set OCTET_HEGEL_CARGO_CONFIG "${cargoVendorDir}/config.toml" \
              --set OCTET_HEGEL_LD_LIBRARY_PATH "${rustToolchainLibPath}"
            wrapProgram "$out/bin/cargo-slotcar" \
              --prefix PATH : "${cargoOctetRuntimePath}/bin" \
              --prefix LD_LIBRARY_PATH : "${rustToolchainLibPath}" \
              --set TIGERSTYLE_TOOLCHAIN "${rustupToolchain}" \
              --set TIGERSTYLE_DRIVER_PATH "${dylintDriver}/bin/dylint-driver" \
              --set TIGERSTYLE_LINT_LIB "${octetUiLibrary}/lib/${octetUiLibraryName}" \
              --set OCTET_COVERAGE_CARGO "${cargo-shim}/bin/cargo" \
              --set OCTET_COVERAGE_PATH "${cargoOctetRuntimePath}/bin" \
              --set OCTET_COVERAGE_CARGO_CONFIG "${cargoVendorDir}/config.toml" \
              --set OCTET_COVERAGE_LD_LIBRARY_PATH "${rustToolchainLibPath}" \
              --set OCTET_MUTATION_CARGO "${cargo-shim}/bin/cargo" \
              --set OCTET_MUTATION_PATH "${cargoOctetRuntimePath}/bin" \
              --set OCTET_MUTATION_CARGO_CONFIG "${cargoVendorDir}/config.toml" \
              --set OCTET_MUTATION_LD_LIBRARY_PATH "${rustToolchainLibPath}" \
              --set OCTET_HEGEL_CARGO "${cargo-shim}/bin/cargo" \
              --set OCTET_HEGEL_PATH "${cargoOctetRuntimePath}/bin" \
              --set OCTET_HEGEL_CARGO_CONFIG "${cargoVendorDir}/config.toml" \
              --set OCTET_HEGEL_LD_LIBRARY_PATH "${rustToolchainLibPath}"
            wrapProgram "$out/bin/cargo-tigerstyle" \
              --prefix PATH : "${cargoOctetRuntimePath}/bin" \
              --prefix LD_LIBRARY_PATH : "${rustToolchainLibPath}" \
              --set TIGERSTYLE_TOOLCHAIN "${rustupToolchain}" \
              --set TIGERSTYLE_DRIVER_PATH "${dylintDriver}/bin/dylint-driver" \
              --set TIGERSTYLE_LINT_LIB "${octetUiLibrary}/lib/${octetUiLibraryName}" \
              --set OCTET_COVERAGE_CARGO "${cargo-shim}/bin/cargo" \
              --set OCTET_COVERAGE_PATH "${cargoOctetRuntimePath}/bin" \
              --set OCTET_COVERAGE_CARGO_CONFIG "${cargoVendorDir}/config.toml" \
              --set OCTET_COVERAGE_LD_LIBRARY_PATH "${rustToolchainLibPath}" \
              --set OCTET_MUTATION_CARGO "${cargo-shim}/bin/cargo" \
              --set OCTET_MUTATION_PATH "${cargoOctetRuntimePath}/bin" \
              --set OCTET_MUTATION_CARGO_CONFIG "${cargoVendorDir}/config.toml" \
              --set OCTET_MUTATION_LD_LIBRARY_PATH "${rustToolchainLibPath}" \
              --set OCTET_HEGEL_CARGO "${cargo-shim}/bin/cargo" \
              --set OCTET_HEGEL_PATH "${cargoOctetRuntimePath}/bin" \
              --set OCTET_HEGEL_CARGO_CONFIG "${cargoVendorDir}/config.toml" \
              --set OCTET_HEGEL_LD_LIBRARY_PATH "${rustToolchainLibPath}"
          '';
        };

        policyExport = pkgs.writeShellApplication {
          name = "octet-policy-export";
          runtimeInputs = with pkgs; [ nickel ];
          text = ''
            exec nickel export --format json standards/policies/registry.ncl
          '';
        };

        proofRailManifestExport = pkgs.writeShellApplication {
          name = "octet-proof-rail-manifest-export";
          runtimeInputs = with pkgs; [ nickel ];
          text = ''
            exec nickel export --format json standards/proof-rails/manifest.ncl
          '';
        };

        trellisProofImportsExport = pkgs.writeShellApplication {
          name = "octet-trellis-proof-imports-export";
          runtimeInputs = with pkgs; [ nickel ];
          text = ''
            exec nickel export --format json standards/proof-imports/trellis.ncl
          '';
        };

        policyArtifacts = pkgs.writeShellApplication {
          name = "octet-policy-artifacts";
          runtimeInputs = with pkgs; [
            nickel
            octetStandards
          ];
          text = ''
            if [ "$#" -ne 1 ]; then
              echo "usage: octet-policy-artifacts OUT_DIR" >&2
              exit 2
            fi
            cd ${self}
            registry_json=$(mktemp)
            nickel export --format json standards/policies/registry.ncl > "$registry_json"
            exec octet-standards export-artifacts --registry "$registry_json" --out-dir "$1"
          '';
        };

        policyValidate = pkgs.writeShellApplication {
          name = "octet-policy-validate";
          runtimeInputs = with pkgs; [
            gawk
            jq
            nickel
          ];
          text = ''
            nickel typecheck standards/policies/registry.ncl
            nickel export --format json standards/policies/registry.ncl \
              | jq --exit-status '.policies | length > 0' >/dev/null
            nickel export --format json standards/policies/registry.ncl \
              | jq --exit-status '.seed_manifests | length == 3' >/dev/null
            nickel export --format json standards/policies/registry.ncl \
              | jq --exit-status 'any(.policies[]; .target_enforcement_mode == "structural" and (.mechanisms | length > 0))' >/dev/null
            nickel export --format json standards/policies/registry.ncl \
              | jq --exit-status 'any(.policies[]; .target_enforcement_mode == "verified" and (.proof_rails | length > 0))' >/dev/null

            while IFS=$'\t' read -r policy_id source_path source_heading; do
              if [ ! -f "$source_path" ]; then
                echo "policy $policy_id references missing source path: $source_path" >&2
                exit 1
              fi
              if ! awk -v heading="$source_heading" '
                /^=+ / {
                  current = $0
                  sub(/^=+ /, "", current)
                  if (current == heading) {
                    found = 1
                  }
                }
                END { exit found ? 0 : 1 }
              ' "$source_path"; then
                echo "policy $policy_id references missing source heading: $source_heading" >&2
                exit 1
              fi
            done < <(nickel export --format json standards/policies/registry.ncl \
              | jq --raw-output '.policies[] | [.policy_id, .source_path, .source_heading] | @tsv')

            for file in standards/policies/fixtures/valid/*.ncl; do
              nickel export --format json "$file" >/dev/null
            done

            for file in standards/policies/fixtures/invalid/*.ncl; do
              if nickel export --format json "$file" >/dev/null 2>&1; then
                echo "invalid policy fixture passed: $file" >&2
                exit 1
              fi
            done

            for file in \
              standards/nickel-exports/fixtures/valid/*.ncl \
              standards/policy-packages/fixtures/valid/*.ncl \
              standards/external-schemas/fixtures/valid/*.ncl; do
              nickel export --format json "$file" >/dev/null
            done

            for file in \
              standards/nickel-exports/fixtures/invalid/*.ncl \
              standards/policy-packages/fixtures/invalid/*.ncl \
              standards/external-schemas/fixtures/invalid/*.ncl; do
              if nickel export --format json "$file" >/dev/null 2>&1; then
                echo "invalid generated-contract/package fixture passed: $file" >&2
                exit 1
              fi
            done
          '';
        };

        policyRegistryCheck =
          pkgs.runCommand "octet-policy-registry-check"
            {
              nativeBuildInputs = with pkgs; [
                gawk
                jq
                nickel
              ];
            }
            ''
              cd ${./.}
              nickel typecheck standards/policies/registry.ncl
              nickel export --format json standards/policies/registry.ncl \
                | jq --exit-status '.policies | length > 0' >/dev/null
              nickel export --format json standards/policies/registry.ncl \
                | jq --exit-status '.seed_manifests | length == 3' >/dev/null
              nickel export --format json standards/policies/registry.ncl \
                | jq --exit-status 'any(.policies[]; .target_enforcement_mode == "structural" and (.mechanisms | length > 0))' >/dev/null
              nickel export --format json standards/policies/registry.ncl \
                | jq --exit-status 'any(.policies[]; .target_enforcement_mode == "verified" and (.proof_rails | length > 0))' >/dev/null

              while IFS=$'\t' read -r policy_id source_path source_heading; do
                if [ ! -f "$source_path" ]; then
                  echo "policy $policy_id references missing source path: $source_path" >&2
                  exit 1
                fi
                if ! awk -v heading="$source_heading" '
                  /^=+ / {
                    current = $0
                    sub(/^=+ /, "", current)
                    if (current == heading) {
                      found = 1
                    }
                  }
                  END { exit found ? 0 : 1 }
                ' "$source_path"; then
                  echo "policy $policy_id references missing source heading: $source_heading" >&2
                  exit 1
                fi
              done < <(nickel export --format json standards/policies/registry.ncl \
                | jq --raw-output '.policies[] | [.policy_id, .source_path, .source_heading] | @tsv')

              for file in standards/policies/fixtures/valid/*.ncl; do
                nickel export --format json "$file" >/dev/null
              done

              for file in standards/policies/fixtures/invalid/*.ncl; do
                if nickel export --format json "$file" >/dev/null 2>&1; then
                  echo "invalid policy fixture passed: $file" >&2
                  exit 1
                fi
              done

              for file in \
                standards/nickel-exports/fixtures/valid/*.ncl \
                standards/policy-packages/fixtures/valid/*.ncl \
                standards/external-schemas/fixtures/valid/*.ncl; do
                nickel export --format json "$file" >/dev/null
              done

              for file in \
                standards/nickel-exports/fixtures/invalid/*.ncl \
                standards/policy-packages/fixtures/invalid/*.ncl \
                standards/external-schemas/fixtures/invalid/*.ncl; do
                if nickel export --format json "$file" >/dev/null 2>&1; then
                  echo "invalid generated-contract/package fixture passed: $file" >&2
                  exit 1
                fi
              done

              touch "$out"
            '';

        astGrepStructuralRailCheck =
          pkgs.runCommand "octet-ast-grep-structural-rail-check"
            {
              nativeBuildInputs = with pkgs; [
                ast-grep
                diffutils
                jq
                nickel
              ];
            }
            ''
              cd ${./.}
              generated_profile=$(mktemp)
              nickel typecheck standards/ast-grep/profile.ncl
              nickel export --format json standards/ast-grep/profile.ncl > "$generated_profile"
              diff -u standards/ast-grep/generated/profile.json "$generated_profile"

              expected_version=$(jq -r '.tool.version' standards/ast-grep/generated/profile.json)
              test "$(ast-grep --version)" = "ast-grep $expected_version"
              ast-grep test --config standards/ast-grep/sgconfig.yml

              ast-grep scan \
                --rule standards/ast-grep/rules/unsafe-block-review.yml \
                --json=compact \
                standards/ast-grep/fixtures/positive > "$TMPDIR/positive.json"
              test "$(jq 'length' "$TMPDIR/positive.json")" -gt 0

              ast-grep scan \
                --rule standards/ast-grep/rules/unsafe-block-review.yml \
                --json=compact \
                standards/ast-grep/fixtures/negative > "$TMPDIR/negative.json"
              test "$(jq 'length' "$TMPDIR/negative.json")" -eq 0
              touch "$out"
            '';

        wasmArtifactRailCheck =
          pkgs.runCommand "octet-wasm-artifact-rail-check"
            {
              nativeBuildInputs = with pkgs; [
                b3sum
                diffutils
                jq
                nickel
              ];
            }
            ''
              cd ${./.}
              generated_profiles=$(mktemp)
              generated_corpus=$(mktemp)
              generated_spacewasm_projection=$(mktemp)
              generated_spacewasm_corpus=$(mktemp)
              nickel typecheck standards/wasm-artifact/profiles.ncl
              nickel export --format json standards/wasm-artifact/profiles.ncl > "$generated_profiles"
              diff -u standards/wasm-artifact/generated/profiles.json "$generated_profiles"
              nickel typecheck standards/wasm-artifact/fixtures/corpus.ncl
              nickel export --format json standards/wasm-artifact/fixtures/corpus.ncl > "$generated_corpus"
              diff -u standards/wasm-artifact/fixtures/corpus.json "$generated_corpus"
              jq -e '[.cases[] | select(.expected_issue_class == "none")] | length > 0' "$generated_corpus" >/dev/null
              jq -e '[.cases[] | select(.expected_issue_class != "none")] | length > 0' "$generated_corpus" >/dev/null

              nickel typecheck standards/wasm-artifact/spacewasm/support-projection.ncl
              nickel export --format json standards/wasm-artifact/spacewasm/support-projection.ncl > "$generated_spacewasm_projection"
              diff -u standards/wasm-artifact/spacewasm/support-projection.json "$generated_spacewasm_projection"
              nickel typecheck standards/wasm-artifact/spacewasm/corpus.ncl
              nickel export --format json standards/wasm-artifact/spacewasm/corpus.ncl > "$generated_spacewasm_corpus"
              diff -u standards/wasm-artifact/spacewasm/corpus.json "$generated_spacewasm_corpus"
              jq -e '.profiles[] | select(.profile_id == "spacewasm-mvp") | .target_runtime.support_projection.source_revision == "e24cf09355a90497148eb5029fdb8e3400bd63e3"' "$generated_profiles" >/dev/null
              jq -e '[.cases[] | select(.generator.tool == "wasm-smith")] | length > 0' "$generated_spacewasm_corpus" >/dev/null
              jq -e '[.cases[] | select(.generator.tool == "wasm-mutate")] | length > 0' "$generated_spacewasm_corpus" >/dev/null
              while IFS=$'\t' read -r relative_path expected_digest; do
                observed_digest=$(b3sum "standards/wasm-artifact/spacewasm/$relative_path" | cut -d ' ' -f 1)
                test "$observed_digest" = "$expected_digest"
              done < <(jq -r '.cases[] | [.source, .artifact_blake3] | @tsv' "$generated_spacewasm_corpus")
              touch "$out"
            '';

        projectArchitecturePolicy =
          pkgs.runCommand "octet-project-architecture-policy-v1"
            {
              nativeBuildInputs = with pkgs; [ jq ];
            }
            ''
              policy_root="$out/share/octet/project-architecture-policy-v1"
              mkdir -p "$policy_root/templates" "$policy_root/octet-self"
              cp ${./templates/octet-architecture.ncl} "$policy_root/templates/octet-architecture.ncl"
              cp ${./templates/octet-architecture.contracts.ncl} "$policy_root/templates/octet-architecture.contracts.ncl"
              cp ${./templates/octet-architecture-export-spec.ncl} "$policy_root/templates/octet-architecture-export-spec.ncl"
              cp ${./standards/project-architecture/octet-self-architecture.ncl} "$policy_root/octet-self/octet-architecture.ncl"
              cp ${./standards/project-architecture/octet-self-architecture.contracts.ncl} "$policy_root/octet-self/octet-architecture.contracts.ncl"
              cp ${./standards/project-architecture/octet-self-architecture.json} "$policy_root/octet-self/octet-architecture.json"
              cp ${./standards/project-architecture/octet-self-architecture.manifest.json} "$policy_root/octet-self/octet-architecture.manifest.json"
              jq --raw-output '.exports[0].output.blake3' \
                "$policy_root/octet-self/octet-architecture.manifest.json" > "$policy_root/octet-self/octet-architecture.blake3"
            '';

        projectArchitecturePolicyCheck =
          pkgs.runCommand "octet-project-architecture-policy-check"
            {
              nativeBuildInputs = with pkgs; [
                coreutils
                diffutils
                nickel
                octetStandards
              ];
            }
            ''
              cd ${./.}
              policy_dir=standards/project-architecture
              actual_policy="$TMPDIR/octet-self-architecture.json"
              actual_spec="$TMPDIR/octet-self-architecture-export-spec.json"
              actual_manifest="$TMPDIR/octet-self-architecture.manifest.json"

              cmp "$policy_dir/octet-self-architecture.contracts.ncl" templates/octet-architecture.contracts.ncl
              nickel typecheck "$policy_dir/octet-self-architecture.contracts.ncl"
              nickel typecheck standards/compiler-architecture-ir/contracts/compiler-architecture-ir.generated.ncl
              nickel typecheck templates/octet-architecture.ncl
              nickel export --format json "$policy_dir/octet-self-architecture.ncl" > "$actual_policy"
              cmp "$actual_policy" "$policy_dir/octet-self-architecture.json"

              for file in "$policy_dir"/fixtures/valid/*.ncl; do
                nickel export --format json "$file" >/dev/null
              done
              for file in "$policy_dir"/fixtures/invalid/*.ncl; do
                if nickel export --format json "$file" >/dev/null 2>&1; then
                  echo "invalid project-architecture fixture passed: $file" >&2
                  exit 1
                fi
              done

              nickel export --format json "$policy_dir/export-spec.ncl" > "$actual_spec"
              octet-standards nickel-export-manifest \
                --spec "$actual_spec" \
                --output "$actual_policy" \
                --manifest-out "$actual_manifest" \
                --nickel-identity "$(nickel --version)"
              cmp "$actual_manifest" "$policy_dir/octet-self-architecture.manifest.json"
              touch "$out"
            '';

        assuranceProfileOnixOSLifecycle =
          pkgs.runCommand "octet-assurance-profile-onixos-lifecycle-proof-v1"
            {
              nativeBuildInputs = with pkgs; [ jq ];
            }
            ''
              profile_root="$out/share/octet/assurance-profiles/onixos-lifecycle-proof-v1"
              mkdir -p "$profile_root/generated"
              cp ${./standards/policies/assurance-profiles/contracts.ncl} "$profile_root/contracts.ncl"
              cp ${./standards/policies/assurance-profiles/export-spec.ncl} "$profile_root/export-spec.ncl"
              cp ${./standards/policies/assurance-profiles/onixos-lifecycle-proof.ncl} "$profile_root/profile.ncl"
              cp ${./standards/policies/assurance-profiles/generated/onixos-lifecycle-proof.json} "$profile_root/generated/profile.json"
              cp ${./standards/policies/assurance-profiles/generated/onixos-lifecycle-proof.manifest.json} "$profile_root/generated/profile.manifest.json"
              jq --raw-output '.exports[] | select(.family_id == "octet.assurance-profile.onixos-lifecycle-proof") | .output.blake3' \
                "$profile_root/generated/profile.manifest.json" > "$profile_root/generated/profile.blake3"
            '';

        assuranceProfileOnixOSLifecycleCheck =
          pkgs.runCommand "octet-assurance-profile-onixos-lifecycle-check"
            {
              nativeBuildInputs = with pkgs; [
                coreutils
                diffutils
                jq
                nickel
                octetStandards
              ];
            }
            ''
              cd ${./.}
              profile_dir=standards/policies/assurance-profiles
              generated_dir="$profile_dir/generated"
              actual_profile="$TMPDIR/onixos-lifecycle-proof.json"
              repeat_profile="$TMPDIR/onixos-lifecycle-proof.repeat.json"
              drift_profile="$TMPDIR/onixos-lifecycle-proof.drift.json"
              actual_spec="$TMPDIR/export-spec.json"
              actual_manifest="$TMPDIR/onixos-lifecycle-proof.manifest.json"
              repeat_manifest="$TMPDIR/onixos-lifecycle-proof.repeat.manifest.json"
              drift_manifest="$TMPDIR/onixos-lifecycle-proof.drift.manifest.json"

              nickel typecheck "$profile_dir/contracts.ncl"
              nickel typecheck "$profile_dir/onixos-lifecycle-proof.ncl"
              nickel export --format json "$profile_dir/onixos-lifecycle-proof.ncl" > "$actual_profile"
              cmp "$actual_profile" "$generated_dir/onixos-lifecycle-proof.json"

              for file in "$profile_dir"/fixtures/valid/*.ncl; do
                nickel export --format json "$file" >/dev/null
              done
              for file in "$profile_dir"/fixtures/invalid/*.ncl; do
                if nickel export --format json "$file" >/dev/null 2>&1; then
                  echo "invalid assurance-profile fixture passed: $file" >&2
                  exit 1
                fi
              done

              nickel export --format json "$profile_dir/export-spec.ncl" > "$actual_spec"
              octet-standards nickel-export-manifest \
                --spec "$actual_spec" \
                --output "$actual_profile" \
                --manifest-out "$actual_manifest" \
                --nickel-identity "$(nickel --version)"
              cmp "$actual_manifest" "$generated_dir/onixos-lifecycle-proof.manifest.json"

              nickel export --format json "$profile_dir/onixos-lifecycle-proof.ncl" > "$repeat_profile"
              octet-standards nickel-export-manifest \
                --spec "$actual_spec" \
                --output "$repeat_profile" \
                --manifest-out "$repeat_manifest" \
                --nickel-identity "$(nickel --version)"
              cmp "$actual_profile" "$repeat_profile"
              cmp "$actual_manifest" "$repeat_manifest"

              cp "$actual_profile" "$drift_profile"
              printf '\n' >> "$drift_profile"
              octet-standards nickel-export-manifest \
                --spec "$actual_spec" \
                --output "$drift_profile" \
                --manifest-out "$drift_manifest" \
                --nickel-identity "$(nickel --version)"
              if cmp --silent "$actual_manifest" "$drift_manifest"; then
                echo "drifted assurance-profile output retained its prior manifest" >&2
                exit 1
              fi
              expected_output_blake3=$(jq --raw-output '.exports[0].output.blake3' "$actual_manifest")
              drift_output_blake3=$(jq --raw-output '.exports[0].output.blake3' "$drift_manifest")
              test "$expected_output_blake3" != "$drift_output_blake3"

              jq --exit-status '
                .profile_id == "onixos-lifecycle-proof-v1"
                and (.mechanisms | map(.assurance_plane) | sort) == ["construction", "effect-control", "provenance-readiness", "semantic-design", "verification"]
                and (.required_evidence | map(.evidence_id) | sort) == ["cairn-proof-chain-readiness", "canonical-fsm-identity", "kani-physical-switch-proof", "octet-core-conformance", "rust-fsm-conformance", "valence-proof-linkage", "verus-lifecycle-proof"]
                and all(.mechanisms[]; (.non_claims | length) > 0)
                and all(.required_evidence[]; (.non_claims | length) > 0)
                and (.non_claims | length) > 0
              ' "$actual_profile" >/dev/null

              packaged_root=${assuranceProfileOnixOSLifecycle}/share/octet/assurance-profiles/onixos-lifecycle-proof-v1
              test -s "$packaged_root/contracts.ncl"
              test -s "$packaged_root/profile.ncl"
              test -s "$packaged_root/generated/profile.json"
              test -s "$packaged_root/generated/profile.manifest.json"
              expected_blake3=$(jq --raw-output '.exports[0].output.blake3' "$actual_manifest")
              test "$expected_blake3" = "$(cat "$packaged_root/generated/profile.blake3")"
              mkdir -p "$out"
            '';

        dynamicEvidenceProfilesCheck =
          pkgs.runCommand "octet-dynamic-evidence-profiles-check"
            {
              nativeBuildInputs = with pkgs; [
                b3sum
                coreutils
                diffutils
                jq
                nickel
              ];
            }
            ''
              cd ${./.}
              generated=$(mktemp)
              expected_profile_identity=7ad698ddebaaac2640a51e5f58871ac026b4b32653fb95bf971ec491560b48d6
              expected_profile_count=1
              expected_max_time_seconds=60
              expected_max_runs=100000
              expected_seed=1

              nickel typecheck templates/dynamic-evidence-profiles.ncl
              nickel export --format json templates/dynamic-evidence-profiles.ncl > "$generated"
              diff -u dynamic-evidence-profiles.json "$generated"
              test "$(b3sum "$generated" | cut -d ' ' -f 1)" = "$expected_profile_identity"
              test "$(jq '.cargo_fuzz_profiles | length' "$generated")" = "$expected_profile_count"
              jq --exit-status \
                --argjson max_time "$expected_max_time_seconds" \
                --argjson max_runs "$expected_max_runs" \
                --argjson seed "$expected_seed" \
                '.cargo_fuzz_profiles[0]
                  | .name == "cargo-fuzz-libfuzzer"
                  and .engine == "libfuzzer"
                  and .target == "octet-profile-intake"
                  and .max_total_time_seconds == $max_time
                  and .max_runs == $max_runs
                  and .seed == $seed
                  and .evidence_tier == "transcript_assertion"' \
                "$generated"
              mkdir -p "$out"
            '';

        sarifProfilesCheck =
          pkgs.runCommand "octet-sarif-profiles-check"
            {
              nativeBuildInputs = with pkgs; [
                b3sum
                diffutils
                jq
                nickel
              ];
            }
            ''
              cd ${./.}
              generated=$(mktemp)
              expected_identity=81a482fe3b807b973898bd7454e73857605f6c5e48e5092d1d6e47b3fe7a9ce7
              expected_profiles=2
              nickel typecheck standards/sarif/profiles.ncl
              nickel export --format json standards/sarif/profiles.ncl > "$generated"
              diff -u standards/sarif/profiles.json "$generated"
              test "$(b3sum "$generated" | cut -d ' ' -f 1)" = "$expected_identity"
              test "$(jq '.profiles | length' "$generated")" = "$expected_profiles"
              jq --exit-status 'all(.profiles[]; .supported_version == "2.1.0" and .workspace_relative_only and .require_rule_id and .evidence_tier == "static_lint")' "$generated"
              mkdir -p "$out"
            '';

        coverageProfilesCheck =
          pkgs.runCommand "octet-coverage-profiles-check"
            {
              nativeBuildInputs = with pkgs; [
                b3sum
                cargo-llvm-cov
                diffutils
                jq
                nickel
              ];
            }
            ''
              cd ${./.}
              generated=$(mktemp)
              expected_identity=e221f5d35528e6979bb44ab58cc5e486109799cf99fb80fc4d85ab77a7082634
              nickel typecheck standards/coverage/profiles.ncl
              nickel export --format json standards/coverage/profiles.ncl > "$generated"
              diff -u standards/coverage/profiles.json "$generated"
              test "$(b3sum "$generated" | cut -d ' ' -f 1)" = "$expected_identity"
              test "$(cargo-llvm-cov llvm-cov --version)" = "cargo-llvm-cov 0.8.3"
              jq --exit-status '
                .schema_version == "octet-coverage-profiles/v1"
                and (.profiles | length) == 1
                and .profiles[0].profile_id == "cargo-llvm-cov-octet-lib"
                and .profiles[0].provider.version == "0.8.3"
                and .profiles[0].evidence_tier == "transcript_assertion"
                and .profiles[0].denominator_policy == "require_equal"
              ' "$generated" >/dev/null
              mkdir -p "$out"
            '';

        benchmarkProfilesCheck =
          pkgs.runCommand "octet-benchmark-profiles-check"
            {
              nativeBuildInputs = with pkgs; [
                b3sum
                diffutils
                jq
                nickel
              ];
            }
            ''
              cd ${./.}
              generated=$(mktemp)
              expected_identity=64e4b8163b3c89d8143d4f6ff07230d33b2b2a9b7bcab35bf45325027a20cd47
              expected_profiles=1
              expected_workloads=4
              expected_corpora=2
              expected_environment_facts=9
              nickel typecheck standards/benchmark/profiles.ncl
              nickel export --format json standards/benchmark/profiles.ncl > "$generated"
              diff -u standards/benchmark/profiles.json "$generated"
              test "$(b3sum "$generated" | cut -d ' ' -f 1)" = "$expected_identity"
              jq --exit-status \
                --argjson expected_profiles "$expected_profiles" \
                --argjson expected_workloads "$expected_workloads" \
                --argjson expected_corpora "$expected_corpora" \
                --argjson expected_environment_facts "$expected_environment_facts" '
                  .schema_version == "octet-benchmark-profiles/v1"
                  and (.profiles | length) == $expected_profiles
                  and .profiles[0].profile_id == "octet-metadata-benchmark"
                  and (.profiles[0].workloads | length) == $expected_workloads
                  and (.profiles[0].signal_corpus | length) == $expected_corpora
                  and ([.profiles[0].signal_corpus[].source_kind] | sort) == (["cross_project", "synthetic"] | sort)
                  and ([.profiles[0].workloads[].mode] | sort) == (["cold", "incremental", "no_change", "warm"] | sort)
                  and all(.profiles[0].workloads[]; (.required_environment_facts | length) == $expected_environment_facts)
                  and .profiles[0].evidence_tier == "transcript_assertion"
                ' "$generated" >/dev/null
              nickel export --format json standards/evidence/fixtures/valid/benchmark_evidence_details_v1.ncl >/dev/null
              if nickel export --format json standards/evidence/fixtures/invalid/benchmark_evidence_bad_profile.ncl >/dev/null 2>&1; then
                echo "invalid benchmark evidence fixture passed" >&2
                exit 1
              fi
              mkdir -p "$out"
            '';

        hegelProfilesCheck =
          pkgs.runCommand "octet-hegel-profiles-check"
            {
              nativeBuildInputs = with pkgs; [
                b3sum
                diffutils
                jq
                nickel
              ];
            }
            ''
              cd ${./.}
              generated_profiles=$(mktemp)
              generated_laws=$(mktemp)
              expected_profile_identity=8739e90ca0707af37eae8553c3b792ec747b0668cad8c4e7e6b0b7c7a5e00e17
              expected_law_identity=33710ac60efb4a7edca25de746d9740d75233b372d0016f54a2e138ec1a038eb
              nickel typecheck standards/hegel/profiles.ncl
              nickel typecheck standards/hegel/laws.ncl
              nickel export --format json standards/hegel/profiles.ncl > "$generated_profiles"
              nickel export --format json standards/hegel/laws.ncl > "$generated_laws"
              diff -u standards/hegel/profiles.json "$generated_profiles"
              diff -u standards/hegel/laws.json "$generated_laws"
              test "$(b3sum "$generated_profiles" | cut -d ' ' -f 1)" = "$expected_profile_identity"
              test "$(b3sum "$generated_laws" | cut -d ' ' -f 1)" = "$expected_law_identity"
              jq --exit-status '
                .schema_version == "octet-hegel-producer-profiles/v1"
                and (.profiles | length) == 1
                and .profiles[0].profile_id == "octet-pure-core-laws-v1"
                and .profiles[0].producer.hegel_version == "0.14.27"
                and .profiles[0].producer.backend == "native"
                and .profiles[0].settings.case_budget == 64
                and .profiles[0].settings.derandomized
                and .profiles[0].evidence_tier == "transcript_assertion"
                and .profiles[0].verification_role == "recorded_only"
                and (.profiles[0].command_argv[0:2] == ["cargo", "test"])
              ' "$generated_profiles" >/dev/null
              jq --exit-status '
                .schema_version == "octet-hegel-law-catalog/v1"
                and (.laws | length) == 1
                and .laws[0].effect_free
                and .laws[0].case_budget == 64
                and (.laws[0].positive_property | length) > 0
                and (.laws[0].negative_property | length) > 0
                and (.laws[0].generated_input_benefit | length) > 0
              ' "$generated_laws" >/dev/null
              nickel export --format json standards/evidence/fixtures/valid/hegel_hardened_details_v1.ncl >/dev/null
              if nickel export --format json standards/evidence/fixtures/invalid/hegel_hardened_wrong_profile.ncl >/dev/null 2>&1; then
                echo "invalid Hegel evidence fixture passed" >&2
                exit 1
              fi
              mkdir -p "$out"
            '';

        miraiPilotCheck =
          pkgs.runCommand "octet-mirai-semantic-pilot-check"
            {
              nativeBuildInputs = with pkgs; [
                b3sum
                diffutils
                jq
                nickel
              ];
            }
            ''
              cd ${./.}
              generated_policy=$(mktemp)
              generated_corpus=$(mktemp)
              expected_policy_identity=cd4120f1df6d9715a615b87bbce4cbf41cdb3e0abd0a40dfd3ae175dc085b1ca
              expected_corpus_identity=1dfb3e7ddeb1bc470d0e17e0a0b1c6b2895895643fa7c98a7f529a20ee99b015
              expected_comparisons=8
              nickel typecheck standards/mirai/pilot.ncl
              nickel typecheck standards/mirai/corpus.ncl
              nickel export --format json standards/mirai/pilot.ncl > "$generated_policy"
              nickel export --format json standards/mirai/corpus.ncl > "$generated_corpus"
              diff -u standards/mirai/pilot.json "$generated_policy"
              diff -u standards/mirai/corpus.json "$generated_corpus"
              test "$(b3sum "$generated_policy" | cut -d ' ' -f 1)" = "$expected_policy_identity"
              test "$(b3sum "$generated_corpus" | cut -d ' ' -f 1)" = "$expected_corpus_identity"
              test "$(readlink ${miraiSourceProvider}/source)" = "${miraiSource}"
              jq --exit-status '
                .schema_version == "octet-mirai-source-provider/v1"
                and .revision == "${miraiRevision}"
                and .rust_toolchain == "nightly-2025-01-10"
                and (.execution_admitted | not)
                and .blocker_code == "stable-machine-output-unavailable"
              ' ${miraiSourceProvider}/provider.json >/dev/null
              jq --exit-status --argjson expected_comparisons "$expected_comparisons" '
                .schema_version == "octet-mirai-semantic-pilot-decision/v1"
                and .decision == "blocked"
                and (.execution_attempted | not)
                and .reproducible_decision
                and .blocker_code == "stable-machine-output-unavailable"
                and (.comparisons | length) == $expected_comparisons
                and all(.comparisons[]; .outcome == "unknown" and (.parent_identities | length) == 2)
              ' standards/mirai/decision.json >/dev/null
              jq -r '.profile.corpus[] | [.source_identity, (.bytes | tostring), .path] | @tsv' "$generated_policy" |
                while IFS=$'\t' read -r identity bytes path; do
                  test "b3:$(b3sum "$path" | cut -d ' ' -f 1)" = "$identity"
                  test "$(wc -c < "$path")" = "$bytes"
                done
              nickel export --format json standards/evidence/fixtures/valid/mirai_pilot_decision_v1.ncl >/dev/null
              if nickel export --format json standards/evidence/fixtures/invalid/mirai_pilot_wrong_decision.ncl >/dev/null 2>&1; then
                echo "invalid MIRAI pilot decision fixture passed" >&2
                exit 1
              fi
              mkdir -p "$out"
            '';

        mutationProfilesCheck =
          pkgs.runCommand "octet-mutation-profiles-check"
            {
              nativeBuildInputs = with pkgs; [
                b3sum
                cargo-mutants
                diffutils
                jq
                nickel
              ];
            }
            ''
              cd ${./.}
              generated=$(mktemp)
              expected_identity=a40fc0726aea38932f42d6096a84846778e832472767b3c3129de4f7e69d9176
              nickel typecheck standards/mutation/profiles.ncl
              nickel export --format json standards/mutation/profiles.ncl > "$generated"
              diff -u standards/mutation/profiles.json "$generated"
              test "$(b3sum "$generated" | cut -d ' ' -f 1)" = "$expected_identity"
              test "$(cargo-mutants mutants --version)" = "cargo-mutants 27.0.0"
              jq --exit-status '
                .schema_version == "octet-mutation-profiles/v1"
                and (.profiles | length) == 1
                and .profiles[0].profile_id == "cargo-mutants-octet-metadata"
                and .profiles[0].provider.version == "27.0.0"
                and .profiles[0].evidence_tier == "transcript_assertion"
                and .profiles[0].retain_logs
                and .profiles[0].retain_diffs
              ' "$generated" >/dev/null
              mkdir -p "$out"
            '';

        strictProfileCatalogCheck =
          pkgs.runCommand "octet-strict-profile-catalog-check"
            {
              nativeBuildInputs = with pkgs; [ nickel ];
            }
            ''
              cd ${./.}
              generated_catalog=$(mktemp)
              nickel typecheck standards/strict-profiles/profiles.ncl
              nickel export --format json standards/strict-profiles/profiles.ncl > "$generated_catalog"
              cmp "$generated_catalog" standards/strict-profiles/generated/strict-rust-profiles.json

              for file in standards/strict-profiles/fixtures/valid/*.ncl; do
                nickel export --format json "$file" >/dev/null
              done
              for file in standards/strict-profiles/fixtures/invalid/*.ncl; do
                if nickel export --format json "$file" >/dev/null 2>&1; then
                  echo "invalid strict-profile fixture passed: $file" >&2
                  exit 1
                fi
              done
              touch "$out"
            '';

        nominalDomainCatalogCheck =
          pkgs.runCommand "octet-nominal-domain-catalog-check"
            {
              nativeBuildInputs = with pkgs; [ nickel ];
            }
            ''
              cd ${./.}
              generated_catalog=$(mktemp)
              nickel typecheck standards/nominal-domains/catalog.ncl
              nickel export --format json standards/nominal-domains/catalog.ncl > "$generated_catalog"
              cmp "$generated_catalog" standards/nominal-domains/generated/catalog.json

              for file in standards/nominal-domains/fixtures/valid/*.ncl; do
                nickel export --format json "$file" >/dev/null
              done
              for file in standards/nominal-domains/fixtures/invalid/*.ncl; do
                if nickel export --format json "$file" >/dev/null 2>&1; then
                  echo "invalid nominal-domain fixture passed: $file" >&2
                  exit 1
                fi
              done
              touch "$out"
            '';

        clippyRestrictionCatalog = builtins.fromJSON (
          builtins.readFile ./standards/clippy-restrictions/generated/profiles.json
        );
        productionSafetyClippyProfile = pkgs.lib.findFirst (
          profile: profile.profile_id == "production-safety"
        ) (throw "missing production-safety Clippy restriction profile") clippyRestrictionCatalog.profiles;
        productionSafetyClippyLevelFlag =
          if productionSafetyClippyProfile.lint_level == "deny" then
            "-D"
          else if productionSafetyClippyProfile.lint_level == "warn" then
            "-W"
          else
            throw "unsupported production-safety Clippy lint level";
        productionSafetyClippyArgs = pkgs.lib.concatMapStringsSep " " (
          lint: "${productionSafetyClippyLevelFlag} clippy::${lint}"
        ) productionSafetyClippyProfile.lints;

        clippyRestrictionProfilesCheck =
          pkgs.runCommand "octet-clippy-restriction-profiles-check"
            {
              nativeBuildInputs = with pkgs; [ nickel ];
            }
            ''
              cd ${./.}
              generated_catalog=$(mktemp)
              nickel typecheck standards/clippy-restrictions/profiles.ncl
              nickel export --format json standards/clippy-restrictions/profiles.ncl > "$generated_catalog"
              cmp "$generated_catalog" standards/clippy-restrictions/generated/profiles.json

              for file in standards/clippy-restrictions/fixtures/valid/*.ncl; do
                nickel export --format json "$file" >/dev/null
              done
              for file in standards/clippy-restrictions/fixtures/invalid/*.ncl; do
                if nickel export --format json "$file" >/dev/null 2>&1; then
                  echo "invalid Clippy restriction profile fixture passed: $file" >&2
                  exit 1
                fi
              done
              touch "$out"
            '';

        effectClosureCatalogCheck =
          pkgs.runCommand "octet-effect-closure-catalog-check"
            {
              nativeBuildInputs = with pkgs; [ nickel ];
            }
            ''
              cd ${./.}
              generated_catalog=$(mktemp)
              nickel typecheck standards/effect-closure/catalog.ncl
              nickel export --format json standards/effect-closure/catalog.ncl > "$generated_catalog"
              cmp "$generated_catalog" standards/effect-closure/generated/effect-closure-catalog.json

              for file in standards/effect-closure/fixtures/valid/*.ncl; do
                nickel export --format json "$file" >/dev/null
              done
              for file in standards/effect-closure/fixtures/invalid/*.ncl; do
                if nickel export --format json "$file" >/dev/null 2>&1; then
                  echo "invalid effect-closure fixture passed: $file" >&2
                  exit 1
                fi
              done
              touch "$out"
            '';

        downstreamProcMacroCheck =
          pkgs.runCommand "octet-downstream-proc-macro-check"
            {
              nativeBuildInputs = [ cargoOctet ];
            }
            ''
              cp -R ${./tests/fixtures/downstream-proc-macro-consumer} fixture
              chmod -R u+w fixture
              cd fixture
              export CARGO_HOME="$PWD/.cargo-home"
              mkdir -p "$CARGO_HOME"
              cp "${workspaceVendorDir}/config.toml" "$CARGO_HOME/config.toml"

              cargo-octet check \
                --profile robust \
                --function-address-path src/lib.rs \
                --artifact-dir artifacts
              cargo-octet artifact verify --artifact-dir artifacts --output-format json > verification.json
              grep -q '"profile_name": "robust"' artifacts/status.json
              test -s artifacts/function-addresses.json
              grep -q '"status": "valid"' verification.json
              touch "$out"
            '';

        proofRailManifestCheck =
          pkgs.runCommand "octet-proof-rail-manifest-check"
            {
              nativeBuildInputs = with pkgs; [
                b3sum
                jq
                nickel
              ];
            }
            ''
              cd ${./.}
              manifest_json=$(mktemp)
              aeneas_profile_json=$(mktemp)
              consumer_profile_json=$(mktemp)
              valence_adapter_fixture_json=$(mktemp)
              compatibility_matrix_json=$(mktemp)
              trellis_import_json=$(mktemp)
              nickel export --format json standards/proof-rails/manifest.ncl > "$manifest_json"
              jq --exit-status '
                .schema_version == "octet-proof-rail-manifest/v1"
                and .rails.aeneas_fixture.approved_theorem_statement_blake3 == "${approvedAeneasTheoremStatementBlake3}"
                and .rails.aeneas_external_consumer.theorem_id == "aeneas.consumer.increment-matches-model"
                and .rails.aeneas_external_consumer.source_paths == ["src/lib.rs"]
                and .rails.aeneas_external_consumer.proof_paths == ["Theorem.lean"]
              ' "$manifest_json" >/dev/null

              nickel typecheck standards/proof-rails/aeneas-lean/contracts.ncl
              nickel typecheck standards/proof-rails/aeneas-lean/profile.ncl
              nickel export --format json standards/proof-rails/aeneas-lean/profile.ncl > "$aeneas_profile_json"
              cmp "$aeneas_profile_json" standards/proof-rails/aeneas-lean/generated/profile.json
              nickel typecheck standards/proof-rails/aeneas-lean/consumer-contracts.ncl
              nickel typecheck standards/proof-rails/aeneas-lean/consumer-profile.ncl
              nickel export --format json standards/proof-rails/aeneas-lean/consumer-profile.ncl > "$consumer_profile_json"
              cmp "$consumer_profile_json" standards/proof-rails/aeneas-lean/generated/consumer-profile.json
              expected_consumer_identity=62bb3c01a2037eef1410e35d3902d73fb5e7439e8856d721d0823e14c0af4879
              test "$(b3sum "$consumer_profile_json" | cut -d ' ' -f 1)" = "$expected_consumer_identity"
              jq --exit-status '
                .schema_version == "octet-aeneas-consumer-profile/v1"
                and .profile_id == "octet-external-aeneas-consumer-v1"
                and .provider.profile_identity == "b3:cd11b49d1acede2c6a91fa26f0b7a6b4b28db4007b4601a65b3269706a0ce625"
                and .provider.compatibility_matrix_identity == "b3:403656d75de41b955cd2493ef289ba17308da65a3d214babc1ea606aeae248f5"
                and (.source.selected_items | length) == 1
                and (.theorems | length) == 1
                and .outcomes == ["supported", "blocked", "failed"]
                and (.provider.network_allowed | not)
              ' "$consumer_profile_json" >/dev/null
              nickel typecheck standards/proof-rails/aeneas-lean/valence-adapter-fixture.ncl
              nickel export --format json standards/proof-rails/aeneas-lean/valence-adapter-fixture.ncl > "$valence_adapter_fixture_json"
              cmp "$valence_adapter_fixture_json" standards/proof-rails/aeneas-lean/generated/valence-adapter-fixture.json
              valence_stage_count=6
              jq --exit-status \
                --argjson valence_stage_count "$valence_stage_count" \
                ' .schema_version == "octet-aeneas-lean-valence-adapter/v1"
                  and .adapter_target == "valence.stack-provenance.adapter-input"
                  and .provider_id == "aeneas-lean"
                  and .octet_export.artifact_verification_status == "valid"
                  and .octet_export.proof_receipt_status == "passed"
                  and .octet_export.exported_after_verification == true
                  and (.octet_export.artifact_verification_receipt | length) > 0
                  and (.octet_export.proof_receipt_id | length) > 0
                  and (.stages | length) == $valence_stage_count
                  and ([.stages[].stage_kind] | index("rust_source") != null)
                  and ([.stages[].stage_kind] | index("charon_extraction") != null)
                  and ([.stages[].stage_kind] | index("aeneas_translation") != null)
                  and ([.stages[].stage_kind] | index("generated_lean") != null)
                  and ([.stages[].stage_kind] | index("theorem") != null)
                  and ([.stages[].stage_kind] | index("lean_check") != null)
                  and .property_candidate.evidence_tier == "formal_proof"
                  and .property_candidate.owner == "octet"
                  and (.non_claims | index("does not prove Aeneas or Charon translation soundness") != null)
                  and (.non_claims | index("does not prove Lean or verifier soundness") != null)
                  and (.non_claims | index("does not prove whole-program correctness") != null)
                ' "$valence_adapter_fixture_json" >/dev/null
              jq --exit-status '
                .schema_version == "octet-aeneas-lean-provider-profile/v1"
                and .provider_id == "aeneas-lean"
                and .aeneas_release == "0.1.0"
                and .aeneas_revision == "bc2eb6f941369fa3c695e30598b5758b11e4f0de"
                and .charon_revision == "cb50ff16b9f1066b8a97dc06da704de2da2fa41c"
                and .lean_toolchain == "leanprover/lean4:v4.31.0"
                and .semantic_reference.revision == "8a9b8bb3823ceceb47026837d23baea0934dfb27"
                and .semantic_reference.license == "Apache-2.0"
                and .semantic_reference.source_sentinel == "README.md:Mechanized LLBC"
                and .semantic_reference.recorded_only == true
                and .extraction_policy.has_errors == false
                and .extraction_policy.skip_borrowck == false
                and (.non_claims | index("does not prove Aeneas or Charon translation soundness") != null)
                and (.non_claims | index("does not prove Lean or verifier soundness") != null)
                and (.non_claims | index("does not prove whole-program correctness") != null)
              ' "$aeneas_profile_json" >/dev/null
              nickel export --format json standards/proof-rails/aeneas-lean/fixtures/valid/current_profile.ncl >/dev/null
              nickel typecheck standards/proof-rails/aeneas-lean/compatibility-matrix.ncl
              nickel export --format json standards/proof-rails/aeneas-lean/compatibility-matrix.ncl > "$compatibility_matrix_json"
              cmp "$compatibility_matrix_json" standards/proof-rails/aeneas-lean/generated/compatibility-matrix.json
              upgrade_dimension_count=7
              canary_count=13
              positive_canary_count=2
              rejection_canary_count=11
              jq --exit-status \
                --argjson upgrade_dimension_count "$upgrade_dimension_count" \
                --argjson canary_count "$canary_count" \
                --argjson positive_canary_count "$positive_canary_count" \
                --argjson rejection_canary_count "$rejection_canary_count" \
                ' .schema_version == "octet-aeneas-lean-compatibility-matrix/v1"
                  and .provider_id == "aeneas-lean"
                  and (.upgrade_dimensions | length) == $upgrade_dimension_count
                  and (.canaries | length) == $canary_count
                  and ([.canaries[] | select(.expected == "pass")] | length) == $positive_canary_count
                  and ([.canaries[] | select(.expected == "reject")] | length) == $rejection_canary_count
                  and ([.canaries[].category] | index("false-vacuous") != null)
                  and ([.canaries[].category] | index("partial-output") != null)
                  and ([.canaries[].category] | index("borrow-check-bypass") != null)
                  and ([.canaries[].category] | index("opacity-model") != null)
                  and ([.canaries[].category] | index("target") != null)
                  and ([.canaries[].category] | index("freshness") != null)
                  and ([.canaries[].category] | index("unsupported-feature") != null)
                  and ([.canaries[] | select(.execution == "live")] | length) == 3
                  and ([.canaries[] | select(.execution == "pure")] | length) == 10
                ' "$compatibility_matrix_json" >/dev/null
              while IFS= read -r test_function; do
                if ! grep --fixed-strings "fn $test_function" crates/octet-standards/src/formal_proof.rs >/dev/null \
                  && ! grep --fixed-strings "fn $test_function" cargo-octet/src/aeneas_execution.rs >/dev/null; then
                  echo "Aeneas compatibility canary test function is missing: $test_function" >&2
                  exit 1
                fi
              done < <(jq --raw-output '.canaries[].test_function' "$compatibility_matrix_json")
              for file in standards/proof-rails/aeneas-lean/fixtures/invalid/*.ncl; do
                if nickel export --format json "$file" >/dev/null 2>&1; then
                  echo "invalid Aeneas Lean provider profile fixture passed: $file" >&2
                  exit 1
                fi
              done
              jq --exit-status '.rails.addition_bound.command_argv[0] == "octet-verus"' "$manifest_json" >/dev/null
              jq --exit-status '.rails.addition_bound.verifier_mode == "fixture"' "$manifest_json" >/dev/null
              jq --exit-status '.rails.addition_bound.receipt_paths | length > 0' "$manifest_json" >/dev/null
              jq --exit-status '.rails.addition_bound_production.command_argv[0] == "octet-production-verus"' "$manifest_json" >/dev/null
              jq --exit-status '.rails.addition_bound_production.verifier_mode == "production"' "$manifest_json" >/dev/null
              jq --exit-status '.rails.addition_bound_production.pass_signal == "verification results:: 1 verified, 0 errors"' "$manifest_json" >/dev/null
              jq --exit-status '.rails.ordering_reflexive_production.command_argv == ["octet-production-verus", "verus/proofs/ordering_reflexive.rs"]' "$manifest_json" >/dev/null
              jq --exit-status '.rails.ordering_reflexive_production.source_paths == ["src/verified/ordering.rs"]' "$manifest_json" >/dev/null
              jq --exit-status '.rails.ordering_reflexive_production.verifier_mode == "production"' "$manifest_json" >/dev/null
              jq --exit-status '.rails.addition_bound_production_negative.command_argv == ["octet-production-verus", "verus/proofs/addition_bound_invalid.rs"]' "$manifest_json" >/dev/null
              jq --exit-status '.rails.addition_bound_production_negative.source_paths == ["verus/failure-fixtures/addition_bound_invalid_source.rs"]' "$manifest_json" >/dev/null
              jq --exit-status '.rails.addition_bound_production_negative.verifier_mode == "production"' "$manifest_json" >/dev/null
              production_graph_digest=$(cat ${octetProductionVerusNixClosureManifest}/octet-production-verus-closure.graph.blake3)
              jq --exit-status --arg digest "$production_graph_digest" \
                '.rails.addition_bound_production.receipt_verifier.verifier_nix_closure_graph_digest == $digest' \
                "$manifest_json" >/dev/null
              jq --exit-status --arg digest "$production_graph_digest" \
                '.rails.ordering_reflexive_production.receipt_verifier.verifier_nix_closure_graph_digest == $digest' \
                "$manifest_json" >/dev/null
              jq --exit-status --arg digest "$production_graph_digest" \
                '.rails.addition_bound_production_negative.receipt_verifier.verifier_nix_closure_graph_digest == $digest' \
                "$manifest_json" >/dev/null

              nickel export --format json standards/proof-imports/trellis.ncl > "$trellis_import_json"
              cmp "$trellis_import_json" standards/proof-imports/generated/trellis-proof-imports.json
              jq --exit-status '.schema_version == "octet-imported-proof-bundles/v1"' "$trellis_import_json" >/dev/null
              jq --exit-status '.bundles[0].repository_label == "../trellis"' "$trellis_import_json" >/dev/null
              jq --exit-status '.bundles[0].hash_algorithm == "blake3"' "$trellis_import_json" >/dev/null
              jq --exit-status '.bundles[0].intended_evidence_tier == "manual_review_required"' "$trellis_import_json" >/dev/null

              for file in standards/proof-rails/fixtures/valid/*.ncl; do
                nickel export --format json "$file" >/dev/null
              done
              for file in standards/proof-rails/fixtures/invalid/*.ncl; do
                if nickel export --format json "$file" >/dev/null 2>&1; then
                  echo "invalid proof-rail manifest fixture passed: $file" >&2
                  exit 1
                fi
              done

              touch "$out"
            '';

        aeneasConsumerContractCheck = pkgs.runCommand "octet-aeneas-consumer-contract-check" { } ''
          test -e ${proofRailManifestCheck}
          test -s ${./standards/proof-rails/aeneas-lean/generated/consumer-profile.json}
          test -s ${./standards/proof-rails/aeneas-lean/fixtures/external-consumer/src/lib.rs}
          test -s ${./standards/proof-rails/aeneas-lean/fixtures/external-consumer/Theorem.lean}
          test -s ${./standards/proof-rails/aeneas-lean/fixtures/external-consumer/expected/llbc.llbc}
          test -s ${./standards/proof-rails/aeneas-lean/fixtures/external-consumer/expected/translation.json}
          test -s ${./standards/proof-rails/aeneas-lean/fixtures/external-consumer/expected/Llbc.lean}
          test -s ${./standards/proof-rails/aeneas-lean/fixtures/external-consumer/expected/formal-proof-receipt.json}
          touch "$out"
        '';

        productionVerusNegativeProofCheck = pkgs.runCommand "octet-production-verus-negative-proof" { } ''
          set +e
          ${octetProductionVerus}/bin/octet-production-verus ${./verus/proofs/addition_bound_invalid.rs} > stdout.txt 2> stderr.txt
          proof_status="$?"
          set -e
          if [ "$proof_status" -eq 0 ]; then
            echo "invalid production Verus proof unexpectedly passed" >&2
            exit 1
          fi
          if grep -F "verification results:: 1 verified, 0 errors" stdout.txt >/dev/null; then
            echo "invalid production Verus proof printed the success pass signal" >&2
            exit 1
          fi
          mkdir -p "$out"
          cp stdout.txt stderr.txt "$out/"
        '';

        productionVerusNegativeProveCheck =
          pkgs.runCommand "octet-production-verus-negative-prove"
            {
              nativeBuildInputs = with pkgs; [
                gnugrep
                jq
              ];
            }
            ''
              artifact_dir=$(mktemp -d)
              policy_dir=$(mktemp -d)
              repo="$TMPDIR/source"
              cp -r ${src} "$repo"
              chmod -R u+w "$repo"
              ${policyArtifacts}/bin/octet-policy-artifacts "$policy_dir"
              cd "$repo"
              set +e
              ${cargoOctet}/bin/cargo-octet prove \
                --policy-artifacts-dir "$policy_dir" \
                --source verus/failure-fixtures/addition_bound_invalid_source.rs \
                --artifact-dir "$artifact_dir" \
                --output-format json > prove.json 2> prove.err
              prove_status="$?"
              set -e
              if [ "$prove_status" -ne 1 ]; then
                echo "expected cargo octet prove to exit 1 for the negative production rail, got $prove_status" >&2
                exit 1
              fi
              jq --exit-status '.status == "failed"' "$artifact_dir/status.json" >/dev/null
              grep -F 'verifier-failure: proof command `octet-production-verus verus/proofs/addition_bound_invalid.rs` exited without success' "$artifact_dir/summary.txt" >/dev/null
              receipt="$artifact_dir/proofs/octet.verus_proof_rails.addition-bound-production-negative-rail-0.receipt"
              grep -F '"status": "failed"' "$receipt" >/dev/null
              grep -F '"stdout_artifact"' "$receipt" >/dev/null
              grep -F '"stderr_artifact"' "$receipt" >/dev/null
              if grep -F '"evidence_tier":"formal_proof"' "$artifact_dir/provenance.jsonl" >/dev/null; then
                echo "failed production Verus rail emitted formal_proof provenance" >&2
                exit 1
              fi
              mkdir -p "$out"
              cp -r "$artifact_dir" "$out/artifacts"
              cp prove.json prove.err "$out/"
            '';

        productionVerusMultiRailProveCheck =
          pkgs.runCommand "octet-production-verus-multi-rail-prove"
            {
              nativeBuildInputs = with pkgs; [ jq ];
            }
            ''
              expected_total_proof_count=3
              expected_production_proof_count=2
              artifact_dir=$(mktemp -d)
              policy_dir=$(mktemp -d)
              repo="$TMPDIR/source"
              cp -r ${src} "$repo"
              chmod -R u+w "$repo"
              ${policyArtifacts}/bin/octet-policy-artifacts "$policy_dir"
              cd "$repo"
              ${cargoOctet}/bin/cargo-octet prove \
                --policy-artifacts-dir "$policy_dir" \
                --source src/verified \
                --artifact-dir "$artifact_dir" \
                --output-format json > prove.json
              jq --exit-status --argjson expected "$expected_total_proof_count" \
                '.status == "passed" and (.proofs | length) == $expected' prove.json >/dev/null
              jq --exit-status --argjson expected "$expected_production_proof_count" '
                ([.proofs[] | select(.verifier_mode == "production" and .status == "passed")] | length) == $expected
              ' prove.json >/dev/null
              jq --exit-status '
                any(.proofs[]; .policy_id == "octet.verus_proof_rails.addition-bound-production" and .receipt == "proofs/octet.verus_proof_rails.addition-bound-production-rail-0.receipt")
                and any(.proofs[]; .policy_id == "octet.verus_proof_rails.ordering-reflexive-production" and .receipt == "proofs/octet.verus_proof_rails.ordering-reflexive-production-rail-0.receipt")
              ' prove.json >/dev/null
              jq --slurp --exit-status --argjson expected "$expected_production_proof_count" '
                ([.[] | select(.evidence_tier == "formal_proof" and (.artifact_locator | test("addition-bound-production|ordering-reflexive-production")))] | length) == $expected
              ' "$artifact_dir/provenance.jsonl" >/dev/null
              ${cargoOctet}/bin/cargo-octet artifact verify --artifact-dir "$artifact_dir" --output-format json > verify.json
              jq --exit-status '.status == "valid" and (.diagnostics | length) == 0' verify.json >/dev/null
              mkdir -p "$out"
              cp -r "$artifact_dir" "$out/artifacts"
              cp prove.json verify.json "$out/"
            '';

        standardsFixtureCheck =
          pkgs.runCommand "octet-standards-fixtures"
            {
              nativeBuildInputs = with pkgs; [
                jq
                nickel
                octetStandards
              ];
            }
            ''
              cd ${./.}
              artifact_dir=$(mktemp -d)
              registry_json=$(mktemp)
              compliant_json=$(mktemp)
              noncompliant_json=$(mktemp)
              advisory_json=$(mktemp)
              markdown_report=$(mktemp)

              nickel export --format json standards/policies/registry.ncl > "$registry_json"
              octet-standards export-artifacts --registry "$registry_json" --out-dir "$artifact_dir"
              test -s "$artifact_dir/octet-policies.json"
              test -s "$artifact_dir/octet-check-config.toml"
              test -s "$artifact_dir/octet-seed-manifests.json"
              test -s "$artifact_dir/octet-proof-bundles.json"
              test -s "$artifact_dir/octet-config-receipt-families.json"
              test -s "$artifact_dir/octet-artifact-manifest.json"
              jq --exit-status '.schema_version == "octet-artifact-manifest/v1"' "$artifact_dir/octet-artifact-manifest.json" >/dev/null
              jq --exit-status '(.artifacts | length) == 5' "$artifact_dir/octet-artifact-manifest.json" >/dev/null
              jq --exit-status 'any(.artifacts[]; .path == "octet-policies.json" and (.blake3 | length) == 64 and .bytes > 0)' "$artifact_dir/octet-artifact-manifest.json" >/dev/null
              jq --exit-status 'any(.artifacts[]; .path == "octet-config-receipt-families.json" and (.blake3 | length) == 64 and .bytes > 0)' "$artifact_dir/octet-artifact-manifest.json" >/dev/null
              jq --exit-status 'any(.artifacts[]; .path == "octet-check-config.toml" and (.blake3 | length) == 64 and .bytes > 0)' "$artifact_dir/octet-artifact-manifest.json" >/dev/null
              jq --exit-status '.schema_version == "octet-proof-bundle/v1"' "$artifact_dir/octet-proof-bundles.json" >/dev/null
              jq --exit-status 'any(.proof_bundles[]; .policy_id == "tigerstyle.verification.deterministic-effect-handlers")' "$artifact_dir/octet-proof-bundles.json" >/dev/null
              jq --exit-status 'any(.proof_bundles[]; .proof_rail.receipt_verifier.verifier_nix_closure_graph_paths[] == "proof-nix-closure.graph")' "$artifact_dir/octet-proof-bundles.json" >/dev/null
              grep --fixed-strings "structural_policy_ids" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "bounded_queue_forbidden_apis" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "tokio::sync::mpsc::unbounded_channel" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "bounded_queue_capacity_witnesses" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "bounded_queue_backpressure_behaviors" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "capability_forbidden_ambient_apis" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "std::process::" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "capability_core_source_scopes" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "capability_shell_constructor_scopes" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "capability_approved_names" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "type_state_source_scopes" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "type_state_raw_alias_patterns" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "type_state_raw_transition_patterns" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "type_state_required_state_markers" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "type_state_required_transition_methods" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "send_raw(" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_backends" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "local-command-receipt" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_commands" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "bash proof-command.sh" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_command_argv" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "octet-verus" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_paths" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proofs/octet.verus_proof_rails.addition-bound-rail-0.receipt" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_verifier_modes" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "fixture" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_pass_signals" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof-result: pass deterministic-effect-handlers" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_bindings" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "blake3-command-and-source" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_schemas" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "tigerstyle-proof-receipt/v1" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_runners" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "octet-standards" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_environments" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "local-source-root" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_requires" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "trust-boundary" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_verifiers" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "octet-standards@proof-receipt/v1" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_verifier_digests" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "9fb0cd3ff1b2898eba4c6f71479f93ae3cdc80f869cd8ae5e6355235ec5f9aef" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_verifier_binaries" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_verifier_binary_digests" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "ca07415846ec49c45311f7b889ba9be3f316e180ad51eab64b0903c5dcd27821" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_verifier_build_closures" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "nix-octet-standards-build-closure/v1" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_verifier_build_closure_digests" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "d482e4bbbb8e9e1a102319c19c55327cc6944fbb9abbd67122e1f9f88a9109ee" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_verifier_build_closure_paths" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof-build-closure.txt" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_verifier_nix_derivations" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "octet-standards.drv" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_verifier_nix_derivation_digests" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "395cf66a51543cf89028075deef9cf172bac902fd777f0ba0bead91aa68afec4" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_verifier_nix_closures" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "nix-store-recursive-closure/v1" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_verifier_nix_closure_digests" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "b1501c0c0d38a17b4a5ddd298136b1c168a499237b872cf02fc72f2650a002f2" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_verifier_nix_closure_paths" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof-nix-closure.txt" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_verifier_nix_closure_graph_digests" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "0ed5ac9f1899c34eeda9035c5a165eefeb0beb254a1e36e354e458ae97d4943b" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_verifier_nix_closure_graph_paths" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof-nix-closure.graph" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_toolchains" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "rust-toolchain.toml" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "proof_rail_receipt_toolchain_digests" "$artifact_dir/octet-check-config.toml" >/dev/null
              grep --fixed-strings "50692525e10589e3a1060a0ed0adc77f695ad5617121b572e549ef9855ceb7a8" "$artifact_dir/octet-check-config.toml" >/dev/null

              octet-standards check \
                --policy-artifacts-dir "$artifact_dir" \
                --source standards/check-fixtures/suppressions/compliant \
                --family devtools \
                --output json > "$compliant_json"
              jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null
              jq --exit-status '.active_suppressions | length == 1' "$compliant_json" >/dev/null

              octet-standards check \
                --policy-artifacts-dir "$artifact_dir" \
                --source standards/check-fixtures/architecture/compliant \
                --source standards/check-fixtures/capability/clock-handler.txt \
                --family architecture \
                --output json > "$compliant_json"
              jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null
              jq --exit-status 'any(.policy_evaluations[]; .policy_id == "tigerstyle.architecture.fcis-crate-boundary" and .target_enforcement_mode == "structural" and .failure_type == "crate-boundary")' "$compliant_json" >/dev/null

              octet-standards check \
                --policy-artifacts-dir "$artifact_dir" \
                --source standards/check-fixtures/embedded/compliant \
                --family embedded \
                --output json > "$compliant_json"
              jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null

              octet-standards check \
                --policy-artifacts-dir "$artifact_dir" \
                --source standards/check-fixtures/architecture/compliant/typestate-protocol \
                --family idioms \
                --output json > "$compliant_json"
              jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null
              jq --exit-status 'all(.diagnostics[]; .policy_id == "tigerstyle.idioms.nominal-type-boundaries")' "$compliant_json" >/dev/null
              jq --exit-status 'any(.policy_evaluations[]; .policy_id == "tigerstyle.idioms.nominal-type-boundaries")' "$compliant_json" >/dev/null

              octet-standards check \
                --policy-artifacts-dir "$artifact_dir" \
                --source standards/check-fixtures/verification/compliant \
                --family verification \
                --output json > "$compliant_json"
              jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null
              jq --exit-status 'any(.policy_evaluations[]; .policy_id == "tigerstyle.verification.deterministic-effect-handlers" and .current_enforcement_mode == "verified" and .failure_type == "verified-proof-rail")' "$compliant_json" >/dev/null

              octet-standards check \
                --policy-artifacts-dir "$artifact_dir" \
                --source standards/check-fixtures/verification/compliant \
                --family verification \
                --execute-proof-rails \
                --output json > "$compliant_json"
              jq --exit-status '.diagnostics | length == 0' "$compliant_json" >/dev/null

              octet-standards check \
                --policy-artifacts-dir "$artifact_dir" \
                --source standards/check-fixtures/architecture/advisory \
                --source standards/check-fixtures/capability/clock-handler.txt \
                --family architecture \
                --output json > "$advisory_json"
              jq --exit-status '.diagnostics | length == 1' "$advisory_json" >/dev/null
              jq --exit-status '.diagnostics[0].policy_id == "tigerstyle.architecture.public-api-budget"' "$advisory_json" >/dev/null
              jq --exit-status '.diagnostics[0].severity == "warning"' "$advisory_json" >/dev/null

              if octet-standards check \
                --policy-artifacts-dir "$artifact_dir" \
                --source standards/check-fixtures/suppressions/noncompliant \
                --family devtools \
                --output json > "$noncompliant_json"; then
                echo "noncompliant suppression fixtures unexpectedly passed" >&2
                exit 1
              fi
              jq --exit-status '.diagnostics | length >= 1' "$noncompliant_json" >/dev/null

              if octet-standards check \
                --policy-artifacts-dir "$artifact_dir" \
                --source standards/check-fixtures/architecture/noncompliant \
                --source standards/check-fixtures/capability/clock-handler.txt \
                --family architecture \
                --output json > "$noncompliant_json"; then
                echo "noncompliant architecture fixtures unexpectedly passed" >&2
                exit 1
              fi
              jq --exit-status '.diagnostics | length >= 4' "$noncompliant_json" >/dev/null
              jq --exit-status 'any(.diagnostics[]; .policy_id == "tigerstyle.architecture.external-domain-leakage")' "$noncompliant_json" >/dev/null
              jq --exit-status 'any(.diagnostics[]; .policy_id == "tigerstyle.architecture.root-crate-thinness" and (.path | contains("thick-root")))' "$noncompliant_json" >/dev/null
              jq --exit-status 'any(.diagnostics[]; .policy_id == "tigerstyle.architecture.fcis-crate-boundary" and (.message | contains("must depend on core crate `tigerstyle-core`")))' "$noncompliant_json" >/dev/null
              jq --exit-status 'any(.diagnostics[]; .policy_id == "tigerstyle.architecture.effect-capability-boundaries")' "$noncompliant_json" >/dev/null
              jq --exit-status 'all(.diagnostics[]; .policy_id != "tigerstyle.idioms.nominal-type-boundaries")' "$noncompliant_json" >/dev/null
              jq --exit-status 'any(.policy_evaluations[]; .target_enforcement_mode == "structural")' "$noncompliant_json" >/dev/null
              jq --exit-status 'all(.diagnostics[]; ((.path | contains("thick-root")) | not) or .policy_id != "tigerstyle.architecture.public-api-budget")' "$noncompliant_json" >/dev/null

              if octet-standards check \
                --policy-artifacts-dir "$artifact_dir" \
                --source standards/check-fixtures/architecture/noncompliant/typestate-raw \
                --family idioms \
                --output json > "$noncompliant_json"; then
                echo "noncompliant type-state fixtures unexpectedly passed" >&2
                exit 1
              fi
              jq --exit-status '.diagnostics | length >= 1' "$noncompliant_json" >/dev/null
              jq --exit-status 'all(.diagnostics[]; .policy_id == "tigerstyle.idioms.nominal-type-boundaries")' "$noncompliant_json" >/dev/null
              jq --exit-status 'any(.diagnostics[]; .message | contains("type ConnectionState = bool"))' "$noncompliant_json" >/dev/null

              if octet-standards check \
                --policy-artifacts-dir "$artifact_dir" \
                --source standards/check-fixtures/verification/noncompliant \
                --family verification \
                --output json > "$noncompliant_json"; then
                echo "noncompliant proof-rail fixtures unexpectedly passed" >&2
                exit 1
              fi
              jq --exit-status '.diagnostics | length >= 1' "$noncompliant_json" >/dev/null
              jq --exit-status 'all(.diagnostics[]; .policy_id == "tigerstyle.verification.deterministic-effect-handlers")' "$noncompliant_json" >/dev/null
              jq --exit-status 'any(.diagnostics[]; .message | contains("missing pass signal"))' "$noncompliant_json" >/dev/null

              if octet-standards check \
                --policy-artifacts-dir "$artifact_dir" \
                --source standards/check-fixtures/verification/noncompliant \
                --family verification \
                --execute-proof-rails \
                --output json > "$noncompliant_json"; then
                echo "noncompliant proof-command fixtures unexpectedly passed" >&2
                exit 1
              fi
              jq --exit-status '.diagnostics | length >= 1' "$noncompliant_json" >/dev/null
              jq --exit-status 'any(.diagnostics[]; .message | contains("missing pass signal"))' "$noncompliant_json" >/dev/null

              if octet-standards check \
                --policy-artifacts-dir "$artifact_dir" \
                --source standards/check-fixtures/embedded/noncompliant \
                --family embedded \
                --output json > "$noncompliant_json"; then
                echo "noncompliant embedded fixtures unexpectedly passed" >&2
                exit 1
              fi
              jq --exit-status '.diagnostics | length >= 2' "$noncompliant_json" >/dev/null
              jq --exit-status 'any(.diagnostics[]; .policy_id == "tigerstyle.embedded.bounded-queues" and (.message | contains("forbidden unbounded API")))' "$noncompliant_json" >/dev/null
              jq --exit-status 'any(.diagnostics[]; .policy_id == "tigerstyle.embedded.bounded-queues" and (.message | contains("missing capacity or backpressure witness")))' "$noncompliant_json" >/dev/null

              octet-standards check \
                --policy-artifacts-dir "$artifact_dir" \
                --source standards/check-fixtures/suppressions/compliant \
                --family devtools \
                --output markdown > "$markdown_report"
              grep --fixed-strings "# Tiger Style Standards Report" "$markdown_report" >/dev/null
              grep --fixed-strings "## Active Suppressions" "$markdown_report" >/dev/null

              touch "$out"
            '';

        policyArtifactConsumerFixtureCheck = mkPolicyArtifactConsumerCheck {
          inherit system;
          src = ./standards/check-fixtures/embedded/compliant;
          family = "embedded";
          expectedDiagnostics = 0;
          requiredPolicyIds = [ "tigerstyle.embedded.bounded-queues" ];
        };
      in
      {
        packages = {
          nickel-1-17-0 = nickel;
          default = octet;
          octet = octet;
          slotcar = octet;
          tigerstyle = octet;
          cargo-octet = cargoOctet;
          cargo-llvm-cov-provider = pkgs.cargo-llvm-cov;
          cargo-mutants-provider = pkgs.cargo-mutants;
          mirai-source-provider = miraiSourceProvider;
          cargo-slotcar = cargoOctet;
          cargo-tigerstyle = cargoOctet;
          fingerprint-docs-downstream-smoke = fingerprintDocsDownstreamSmoke;
          cairn = cairn.packages.${system}.cairn;
          dylint-driver = dylintDriver;
          octet-standards = octetStandards;
          slotcar-standards = octetStandards;
          tigerstyle-standards = octetStandards;
          octet-standards-nix-closure-manifest = octetStandardsNixClosureManifest;
          octet-verus = octetVerus;
          octet-production-verus = octetProductionVerus;
          octet-aeneas-lean-provider-closure = aeneasLeanProviderClosure;
          octet-production-verus-nix-closure-manifest = octetProductionVerusNixClosureManifest;
          octet-verusfmt = octetVerusfmt;
          octet-verus-toolchain-profile = octetVerusToolchainProfile;
          octet-project-architecture-policy = projectArchitecturePolicy;
          octet-assurance-profile-onixos-lifecycle = assuranceProfileOnixOSLifecycle;
          slotcar-standards-nix-closure-manifest = octetStandardsNixClosureManifest;
          octet-policy-export = policyExport;
          tigerstyle-policy-export = policyExport;
          octet-proof-rail-manifest-export = proofRailManifestExport;
          octet-trellis-proof-imports-export = trellisProofImportsExport;
          octet-policy-artifacts = policyArtifacts;
          tigerstyle-policy-artifacts = policyArtifacts;
          octet-policy-validate = policyValidate;
          tigerstyle-policy-validate = policyValidate;
        };

        apps = {
          cargo-octet = flake-utils.lib.mkApp {
            drv = cargoOctet;
            exePath = "/bin/cargo-octet";
          };

          cargo-slotcar = flake-utils.lib.mkApp {
            drv = cargoOctet;
            exePath = "/bin/cargo-slotcar";
          };
          cargo-tigerstyle = flake-utils.lib.mkApp {
            drv = cargoOctet;
            exePath = "/bin/cargo-tigerstyle";
          };

          fingerprint-docs-downstream-smoke = flake-utils.lib.mkApp {
            drv = fingerprintDocsDownstreamSmoke;
            exePath = "/bin/octet-fingerprint-docs-downstream-smoke";
          };

          cairn = flake-utils.lib.mkApp {
            drv = cairn.packages.${system}.cairn;
            exePath = "/bin/cairn";
          };

          octet-standards = flake-utils.lib.mkApp {
            drv = octetStandards;
            exePath = "/bin/octet-standards";
          };

          slotcar-standards = flake-utils.lib.mkApp {
            drv = octetStandards;
            exePath = "/bin/slotcar-standards";
          };

          tigerstyle-standards = flake-utils.lib.mkApp {
            drv = octetStandards;
            exePath = "/bin/tigerstyle-standards";
          };

          octet-policy-export = flake-utils.lib.mkApp {
            drv = policyExport;
            exePath = "/bin/octet-policy-export";
          };

          tigerstyle-policy-export = flake-utils.lib.mkApp {
            drv = policyExport;
            exePath = "/bin/octet-policy-export";
          };

          octet-proof-rail-manifest-export = flake-utils.lib.mkApp {
            drv = proofRailManifestExport;
            exePath = "/bin/octet-proof-rail-manifest-export";
          };

          octet-trellis-proof-imports-export = flake-utils.lib.mkApp {
            drv = trellisProofImportsExport;
            exePath = "/bin/octet-trellis-proof-imports-export";
          };

          octet-policy-artifacts = flake-utils.lib.mkApp {
            drv = policyArtifacts;
            exePath = "/bin/octet-policy-artifacts";
          };

          tigerstyle-policy-artifacts = flake-utils.lib.mkApp {
            drv = policyArtifacts;
            exePath = "/bin/octet-policy-artifacts";
          };

          octet-policy-validate = flake-utils.lib.mkApp {
            drv = policyValidate;
            exePath = "/bin/octet-policy-validate";
          };

          octet-verus = flake-utils.lib.mkApp {
            drv = octetVerus;
            exePath = "/bin/octet-verus";
          };

          octet-production-verus = flake-utils.lib.mkApp {
            drv = octetProductionVerus;
            exePath = "/bin/octet-production-verus";
          };

          octet-aeneas-lean-provider = flake-utils.lib.mkApp {
            drv = aeneasLeanProviderClosure;
            exePath = "/bin/octet-aeneas-lean";
          };

          octet-verusfmt = flake-utils.lib.mkApp {
            drv = octetVerusfmt;
            exePath = "/bin/octet-verusfmt";
          };

          tigerstyle-policy-validate = flake-utils.lib.mkApp {
            drv = policyValidate;
            exePath = "/bin/octet-policy-validate";
          };
        };

        checks =
          let
            mkFingerprintDocsCheck =
              pname: testFilter:
              craneLib.buildPackage {
                inherit
                  src
                  cargoArtifacts
                  buildInputs
                  cargoVendorDir
                  ;
                cargoToml = ./Cargo.toml;
                inherit pname;
                version = "0.1.0";
                nativeBuildInputs = checkNativeBuildInputs;
                cargoExtraArgs = "-p cargo-octet";
                doNotPostBuildInstallCargoBinaries = true;
                buildPhaseCargoCommand = ":";
                cargoTestExtraArgs = "--test fingerprint_consumer ${testFilter} -- --nocapture";
                installPhaseCommand = ''
                  mkdir -p "$out"
                '';
                LD_LIBRARY_PATH = rustToolchainLibPath;
                RUSTUP_TOOLCHAIN = rustupToolchain;
                preCheck = ''
                  export PATH="${dylintShimPath}:$PATH"
                  export CARGO_OCTET_BIN="${cargoOctetUnwrapped}/bin/cargo-octet"
                '';
              };
            cargoOctetSelfCheck =
              pkgs.runCommand "octet-cargo-octet-self-check"
                {
                  nativeBuildInputs = with pkgs; [
                    coreutils
                    gnugrep
                    jq
                  ];
                }
                ''
                  scratch="$(mktemp -d)"
                  source_dir="$scratch/source"
                  cargo_home="$scratch/cargo-home"
                  positive_stdout="$scratch/positive.stdout"
                  positive_stderr="$scratch/positive.stderr"
                  positive_artifacts="$scratch/positive-artifacts"
                  structured_stdout="$scratch/structured.stdout"
                  structured_stderr="$scratch/structured.stderr"
                  structured_artifacts="$scratch/structured-artifacts"
                  drift_dir="$scratch/drift"
                  drift_stdout="$scratch/drift.stdout"
                  drift_stderr="$scratch/drift.stderr"
                  drift_artifacts="$scratch/drift-artifacts"
                  malformed_dir="$scratch/malformed"
                  negative_stdout="$scratch/negative.stdout"
                  negative_stderr="$scratch/negative.stderr"
                  child_failure_dir="$scratch/child-failure"
                  child_failure_stdout="$scratch/child-failure.stdout"
                  child_failure_stderr="$scratch/child-failure.stderr"
                  sentinel_bin="$scratch/sentinel-bin"
                  sentinel_marker="$scratch/cargo-invoked"
                  baseline_relative="standards/self-check-warning-budget/octet-baseline.json"
                  accepted_warning_total=2660
                  accepted_identity_total=1813
                  lint_failure_exit=1
                  integration_failure_exit=2
                  sentinel_exit=99
                  malformed_scope_entry=7
                  diagnostic_tail_lines=80
                  fixture_edition=2024
                  fixture_version=0.0.0

                  cp -r ${src} "$source_dir"
                  chmod -R u+w "$source_dir"
                  mkdir -p "$cargo_home" "$scratch/home"
                  cp "${cargoVendorDir}/config.toml" "$cargo_home/config.toml"
                  baseline="$source_dir/$baseline_relative"
                  test -x "${cargoOctetRuntimePath}/bin/clang"
                  test -x "${cargoOctetRuntimePath}/bin/mold"
                  grep -F 'linker = "clang"' "$source_dir/.cargo/config.toml" >/dev/null
                  grep -F -- '-fuse-ld=mold' "$source_dir/.cargo/config.toml" >/dev/null
                  baseline_hash="$(jq --raw-output '.baseline_hash' "$baseline")"
                  jq --exit-status --argjson expected "$accepted_identity_total" '
                    .schema_version == 1
                    and .finding_schema_version == 2
                    and (.findings | length) == $expected
                    and ([.findings[].stable_id] | length) == ([.findings[].stable_id] | unique | length)
                    and all(.findings[]; (.file | startswith("/") | not))
                    and all(.findings[]; .lint != "negated_predicate")
                  ' "$baseline" >/dev/null

                  set +e
                  (
                    cd "$source_dir"
                    export CARGO_HOME="$cargo_home"
                    export CARGO_TARGET_DIR="$scratch/target"
                    export HOME="$scratch/home"
                    ${cargoOctet}/bin/cargo-octet check \
                      --artifact-dir "$positive_artifacts" \
                      --baseline "$baseline" \
                      > "$positive_stdout" 2> "$positive_stderr"
                  )
                  positive_status="$?"
                  set -e
                  # r[impl octet.runner_self_check.failure_diagnostics]
                  # r[verify octet.runner_self_check.failure_diagnostics]
                  if test "$positive_status" -ne 0; then
                    printf 'canonical self-check failed with status %s\n' "$positive_status" >&2
                    if test -f "$positive_artifacts/status.json" && test -f "$positive_artifacts/results.jsonl"; then
                      jq --raw-output '
                        .baseline
                        | "accepted=\(.accepted_total) current=\(.current_total) accepted_current=\(.accepted_current) new=\(.new_total) stale=\(.stale_total)",
                          (.stale_finding_ids[] | "stale_id=\(.)")
                      ' "$positive_artifacts/status.json" >&2
                      jq --raw-output \
                        --slurpfile status "$positive_artifacts/status.json" '
                          select(.stable_id as $id | ($status[0].baseline.new_finding_ids | index($id)))
                          | "new_id=\(.stable_id) lint=\(.lint) file=\(.file):\(.line):\(.column) message=\(.message)"
                        ' "$positive_artifacts/results.jsonl" >&2
                    else
                      grep -E '^--- octet summary ---$|^(Status|Findings|Warnings|Errors|Baseline hash|Baseline):' \
                        "$positive_stderr" >&2 \
                        || tail -n "$diagnostic_tail_lines" "$positive_stderr" >&2
                    fi
                    exit "$positive_status"
                  fi
                  grep -F -- '--- octet summary ---' "$positive_stderr" >/dev/null
                  grep -E '^Status: (clean|warning-only)$' "$positive_stderr" >/dev/null
                  grep -F 'Errors: 0' "$positive_stderr" >/dev/null
                  grep -F '    Finished checking' "$positive_stderr" >/dev/null

                  # r[verify octet.runner_self_check.inventory.paths]
                  # r[verify octet.runner_self_check.warning_budget.baseline]
                  # r[verify octet.runner_self_check.warning_budget.semantic_refresh]
                  set +e
                  (
                    cd "$source_dir"
                    export CARGO_HOME="$cargo_home"
                    export CARGO_TARGET_DIR="$scratch/target"
                    export HOME="$scratch/home"
                    ${cargoOctet}/bin/cargo-octet check \
                      --output-format json \
                      --artifact-dir "$structured_artifacts" \
                      --baseline "$baseline" \
                      > "$structured_stdout" 2> "$structured_stderr"
                  )
                  structured_status="$?"
                  set -e
                  if test "$structured_status" -ne 0; then
                    printf 'structured self-check failed with status %s\n' "$structured_status" >&2
                    if test -f "$structured_artifacts/status.json"; then
                      jq --raw-output '
                        .baseline
                        | "accepted=\(.accepted_total) current=\(.current_total) accepted_current=\(.accepted_current) new=\(.new_total) stale=\(.stale_total)"
                      ' "$structured_artifacts/status.json" >&2
                    else
                      tail -n "$diagnostic_tail_lines" "$structured_stderr" >&2
                    fi
                    exit "$structured_status"
                  fi
                  test "$(wc -l < "$structured_stdout")" -eq "$accepted_warning_total"
                  jq --exit-status \
                    --arg baseline_hash "$baseline_hash" \
                    --argjson warning_total "$accepted_warning_total" \
                    --argjson identity_total "$accepted_identity_total" '
                      .status == "warning-only"
                      and .exit_code == 0
                      and .total_findings == $warning_total
                      and .warning_findings == $warning_total
                      and .error_findings == 0
                      and .baseline.baseline_hash == $baseline_hash
                      and .baseline.accepted_total == $identity_total
                      and .baseline.current_total == $identity_total
                      and .baseline.accepted_current == $identity_total
                      and .baseline.new_total == 0
                      and .baseline.stale_total == 0
                    ' "$structured_artifacts/status.json" >/dev/null
                  for crate_name in octet_lint_metadata octet octet_standards cargo_octet; do
                    grep -F "\"crate_name\":\"$crate_name\"" "$structured_stdout" >/dev/null
                  done

                  # r[verify octet.runner_self_check.warning_budget.regression]
                  # r[verify octet.runner_self_check.warning_budget.reduction.regression]
                  # r[verify octet.runner_self_check.warning_budget.reduction.public_unsafe_abi.regression]
                  # r[verify octet.runner_self_check.warning_budget.reduction.standards_resource.regression]
                  # r[verify octet.runner_self_check.warning_budget.reduction.standards_report.regression]
                  # r[verify octet.runner_self_check.warning_budget.reduction.standards_policy_evaluation.regression]
                  cp -r "$source_dir" "$drift_dir"
                  test -z "$(grep -F 'has_no_budget' "$source_dir/cargo-octet/src/baseline.rs" || true)"
                  substituteInPlace "$drift_dir/cargo-octet/src/baseline.rs" \
                    --replace-fail \
                    'fn unique_findings(findings: &[Finding]) -> Vec<BaselineFinding<'"'"'_>> {' \
                    'fn unique_findings(findings: &[Finding]) -> Vec<BaselineFinding<'"'"'_>> {
                      let has_no_budget = findings.is_empty();
                      if has_no_budget {
                          return Vec::new();
                      }'
                  set +e
                  (
                    cd "$drift_dir"
                    export CARGO_HOME="$cargo_home"
                    export CARGO_TARGET_DIR="$scratch/drift-target"
                    export HOME="$scratch/home"
                    ${cargoOctet}/bin/cargo-octet check \
                      --output-format json \
                      --artifact-dir "$drift_artifacts" \
                      --baseline "$drift_dir/$baseline_relative" \
                      > "$drift_stdout" 2> "$drift_stderr"
                  )
                  drift_status="$?"
                  set -e
                  test "$drift_status" -eq "$lint_failure_exit"
                  test "$(grep --count '"lint":"negated_predicate"' "$drift_stdout")" -eq 1
                  jq --exit-status \
                    --argjson warning_total "$accepted_warning_total" \
                    --argjson identity_total "$accepted_identity_total" '
                      .status == "lint-failure"
                      and .exit_code == 1
                      and .total_findings == ($warning_total + 1)
                      and .baseline.accepted_total == $identity_total
                      and .baseline.accepted_current == $identity_total
                      and .baseline.current_total == ($identity_total + 1)
                      and .baseline.new_total == 1
                      and .baseline.stale_total == 0
                      and (.baseline.new_finding_ids | length) == 1
                    ' "$drift_artifacts/status.json" >/dev/null

                  mkdir -p "$malformed_dir/src" "$child_failure_dir/src" "$sentinel_bin"
                  cat > "$malformed_dir/Cargo.toml" <<EOF
                  [package]
                  name = "malformed-octet-metadata"
                  version = "$fixture_version"
                  edition = "$fixture_edition"

                  [workspace]

                  [workspace.metadata.octet]
                  default_scope = ["--workspace", $malformed_scope_entry]
                  cargo_check_args = ["--lib"]
                  EOF
                  printf 'pub fn checked() {}\n' > "$malformed_dir/src/lib.rs"
                  cat > "$sentinel_bin/cargo" <<EOF
                  #!${pkgs.runtimeShell}
                  touch "$sentinel_marker"
                  exit $sentinel_exit
                  EOF
                  chmod +x "$sentinel_bin/cargo"

                  set +e
                  (
                    cd "$malformed_dir"
                    PATH="$sentinel_bin:${pkgs.coreutils}/bin" \
                      ${cargoOctetUnwrapped}/bin/cargo-octet check > "$negative_stdout" 2> "$negative_stderr"
                  )
                  negative_status="$?"
                  set -e
                  test "$negative_status" -eq "$integration_failure_exit"
                  grep -F 'error: `default_scope` must contain only strings in' "$negative_stderr" >/dev/null
                  test ! -e "$sentinel_marker"

                  cat > "$child_failure_dir/Cargo.toml" <<EOF
                  [package]
                  name = "octet-child-failure"
                  version = "$fixture_version"
                  edition = "$fixture_edition"

                  [workspace]

                  [workspace.metadata.octet]
                  default_scope = ["--workspace"]
                  cargo_check_args = ["--lib"]
                  EOF
                  printf 'pub fn checked() {}\n' > "$child_failure_dir/src/lib.rs"
                  set +e
                  (
                    cd "$child_failure_dir"
                    PATH="$sentinel_bin:${pkgs.coreutils}/bin" \
                      TIGERSTYLE_TOOLCHAIN="${rustupToolchain}" \
                      TIGERSTYLE_DRIVER_PATH="${dylintDriver}/bin/dylint-driver" \
                      TIGERSTYLE_LINT_LIB="${octetUiLibrary}/lib/${octetUiLibraryName}" \
                      ${cargoOctetUnwrapped}/bin/cargo-octet check \
                        > "$child_failure_stdout" 2> "$child_failure_stderr"
                  )
                  child_failure_status="$?"
                  set -e
                  test "$child_failure_status" -eq "$integration_failure_exit"
                  test -e "$sentinel_marker"
                  grep -F 'Status: integration-failure' "$child_failure_stderr" >/dev/null

                  mkdir -p "$out"
                  cp \
                    "$positive_stdout" "$positive_stderr" \
                    "$structured_stdout" "$structured_stderr" \
                    "$drift_stdout" "$drift_stderr" \
                    "$negative_stdout" "$negative_stderr" \
                    "$child_failure_stdout" "$child_failure_stderr" \
                    "$out/"
                  cp -r "$positive_artifacts" "$structured_artifacts" "$drift_artifacts" "$out/"
                '';
            fingerprintLifecycleDocs = mkFingerprintDocsCheck "octet-fingerprint-lifecycle-docs" "fingerprint_lifecycle_docs_fixture_matches_transcript";
            fingerprintHumanSummaryDocs = mkFingerprintDocsCheck "octet-fingerprint-human-summary-docs" "fingerprint_human_summary_docs_fixture_matches_transcript";
            fingerprintWarningSummaryDocs = mkFingerprintDocsCheck "octet-fingerprint-warning-summary-docs" "fingerprint_warning_summary_docs_fixture_matches_transcript";
            fingerprintJsonOutputDocs = mkFingerprintDocsCheck "octet-fingerprint-json-output-docs" "fingerprint_json_output_docs_fixture_matches_transcript";
            fingerprintSourceScannerDocs = mkFingerprintDocsCheck "octet-fingerprint-source-scanner-docs" "fingerprint_source_scanner_docs_fixture_matches_transcript";
            releaseOperatorSmokeDocs = mkFingerprintDocsCheck "octet-release-operator-smoke-docs" "release_operator_smoke_transcript_and_help_cover_operator_surfaces";
            objectCorpusGate = mkFingerprintDocsCheck "octet-object-corpus-gate" "object_internal_corpus";
            objectCorpusCompatibilityGate = mkFingerprintDocsCheck "octet-object-corpus-compatibility-gate" "object_corpus_compatibility";
            objectCorpusCaveatTaxonomyGate = mkFingerprintDocsCheck "octet-object-corpus-caveat-taxonomy-gate" "caveat_taxonomy";
            objectCorpusFixtureFreshnessGate = mkFingerprintDocsCheck "octet-object-corpus-fixture-freshness" "matches_fresh";
            supplementCompatibilityGate = mkFingerprintDocsCheck "octet-supplement-compatibility-gate" "supplement_compatibility";
            valenceEvidenceIndexGate = pkgs.runCommand "octet-valence-evidence-index" { } ''
              cd ${src}
              ${cargoOctetUnwrapped}/bin/cargo-octet evidence-index check --page docs/current-head-valence-evidence.md
              mkdir -p "$out"
            '';
            evidenceCodebaseIndexGate =
              pkgs.runCommand "octet-evidence-codebase-index"
                {
                  nativeBuildInputs = with pkgs; [
                    coreutils
                    diffutils
                    gnugrep
                    jq
                  ];
                }
                ''
                  scratch="$(mktemp -d)"
                  fixture="$scratch/fixture"
                  repo=${src}
                  mkdir -p "$fixture"

                  grep -F 'Octet-owned review index' "$repo/docs/evidence-codebase-index.md" >/dev/null
                  grep -F 'consumed from the exact standalone `valence-core::evidence_codebase_index` dependency' "$repo/docs/evidence-codebase-index.md" >/dev/null
                  grep -F 'non-publishable, non-workspace package `octet-valence-legacy-fixture`' "$repo/docs/valence-core-cutover.md" >/dev/null
                  grep -F 'MUST NOT be selected by Octet or downstream consumers' "$repo/crates/valence-core/README.md" >/dev/null
                  grep -F 'MUST NOT be selected as canonical `valence-core`' "$repo/crates/valence-core/src/lib.rs" >/dev/null

                  cat > "$fixture/findings.jsonl" <<'EOF'
                  {"version":2,"index":"F1","stable_id":"finding-stable-id","semantic_id":"semantic-fixture","identity_status":"computed","normalized_snippet_hash":"snippet-fixture","config_hash":"config-fixture","cache_status":"disabled","lint":"ambient_clock","group":"tigerstyle::purity","severity":"warning","file":"src/lib.rs","line":1,"column":1,"message":"r[octet.evidence_codebase_index.query.lookup]","principle":"Functional Core","effect_family":"time","evidence_tier":"static_lint","crate_name":"fixture"}
                  EOF
                  cat > "$fixture/proof-cards.json" <<'EOF'
                  {"schema_version":"octet-proof-cards/v1","cards":[{"proof_or_property_id":"octet.policy.capability-boundary#rail-0","source_requirement_or_policy_id":"r[octet.evidence_codebase_index.query.lookup]","policy_id":"octet.policy.capability-boundary","rail_index":0,"checked_rust_source_scope":"src/lib.rs","source_paths":["src/lib.rs"],"proof_target_paths":["proof.rs"],"verifier_mode":"fixture","verifier_identity_summary":"fixture","solver_identity_summary":"fixture","toolchain_identity_summary":"fixture","receipt_path":"proofs/policy.receipt","artifact_blake3_identities":[{"artifact":"proofs/policy.receipt","blake3":"proof-hash"}],"evidence_tier":"formal_proof","evidence_origin":"octet-rerun","status":"passed","trust_boundaries":[],"non_claims":["not whole-program correctness"],"artifact_verifier_command":"cargo octet artifact verify --artifact-dir artifacts/octet","provenance_artifact":"provenance.jsonl","diagnostics":[]}]}
                  EOF
                  cat > "$fixture/capsule.json" <<'EOF'
                  {"schema_version":1,"capsule_hash":"capsule-hash-fixture","capsule_kind":"octet-replay-capsule","manifest":{"command_argv":["cargo","octet","check"],"run_kind":"check","octet_binary_identity":"cargo-octet","config_hash":"config-fixture","input_hashes":["input-hash"],"output_hashes":["finding-stable-id"],"expected_exit_code":0},"replay_contract":{},"redaction_status":"normalized"}
                  EOF
                  cat > "$fixture/transcript.md" <<'EOF'
                  $ cargo octet check
                  => pass
                  EOF
                  cat > "$fixture/provenance.jsonl" <<'EOF'
                  {"schema_version":"octet-provenance-record/v1","source_id":"octet.transcript.assertion","checker_id":"octet-transcript","evidence_tier":"transcript_assertion","artifact_locator":"missing-transcript.md","blake3":"missing-hash","checked_scope":"transcript","parent_artifacts":["capsule-hash-fixture"]}
                  EOF
                  cat > "$fixture/fingerprint-review.json" <<'EOF'
                  {"schema_version":1,"tool_name":"cargo-octet","tool_version":"0.1.0","mode":"check","input_paths":["src/lib.rs"],"manifest_hash":"manifest-fixture","records":[{"stable_id":"fingerprint-stable-id","path":"src/lib.rs","name":"checked","status":"match","computed_hash":"fingerprint-hash","normalized_body_hash":"body-hash","semantic_id":"semantic-fixture","dependency_set_hash":"dependency-hash","dependency_identity_status":"source-scanner-token-set","tool_version":"0.1.0","review":"r[octet.evidence_codebase_index.query.lookup]","evidence":"octet.policy.capability-boundary","requires_review":false,"macro_warning":false}]}
                  EOF
                  cat > "$fixture/evidence-log-manifest.json" <<'EOF'
                  {"schema_version":"octet-evidence-log-manifest/v1","logs":[{"artifact_locator":"run.txt.gz","blake3":"log-hash","bytes":10,"replay_command":"cargo octet check","checked_scope":"fixture","capture_source":"test","retention":"compressed"}]}
                  EOF
                  cat > "$fixture/evidence-rail-receipt.json" <<'EOF'
                  {"schema_version":"octet-evidence-rail-receipt/v1","rail_id":"octet.policy.capability-boundary","checker_id":"cargo-deny","mode":"import","command_argv":["cargo","deny","check"],"workspace_scope":".","checked_rule_families":["licenses"],"status":"passed","finding_counts":{},"bounded_log_identity":"rail-log-hash","imported_input_identity":"rail-input-hash","non_claims":["not correctness"],"diagnostics":[],"findings":[{"family":"ambient_clock","severity":"warning","message":"fixture","file":"src/lib.rs"}],"parent_artifacts":["capsule-hash-fixture"]}
                  EOF

                  ${cargoOctetUnwrapped}/bin/cargo-octet evidence-index codebase build --artifact-dir "$fixture" --output "$scratch/index-a.json"
                  ${cargoOctetUnwrapped}/bin/cargo-octet evidence-index codebase build --artifact-dir "$fixture" --output "$scratch/index-b.json"
                  cmp "$scratch/index-a.json" "$scratch/index-b.json"
                  jq --exit-status '.schema_version == "octet-evidence-codebase-index/v1" and .record_count >= 8' "$scratch/index-a.json" >/dev/null
                  jq --exit-status '.boundary | contains("Octet evidence codebase index proves artifact identity and linkage only") and contains("standalone Valence release ownership")' "$scratch/index-a.json" >/dev/null
                  jq --exit-status 'all(.issues[]?; .severity == "caveat")' "$scratch/index-a.json" >/dev/null
                  ${cargoOctetUnwrapped}/bin/cargo-octet evidence-index codebase query --index "$scratch/index-a.json" --identity 'r[octet.evidence_codebase_index.query.lookup]' > "$scratch/query.json"
                  jq --exit-status '.match_count >= 4 and any(.records[]; .lint_ids | index("ambient_clock")) and any(.records[]; .policy_ids | index("octet.policy.capability-boundary")) and any(.records[]; .stable_id == "capsule-hash-fixture")' "$scratch/query.json" >/dev/null
                  jq --exit-status '.boundary | contains("Octet evidence codebase index proves artifact identity and linkage only")' "$scratch/query.json" >/dev/null
                  ${cargoOctetUnwrapped}/bin/cargo-octet evidence-index codebase html --index "$scratch/index-a.json" --output "$scratch/index.html"
                  grep -F 'Octet evidence codebase index proves artifact identity and linkage only' "$scratch/index.html" >/dev/null
                  grep -F 'href="#octet-finding-' "$scratch/index.html" >/dev/null
                  for overclaim in \
                    'proves whole-program correctness' \
                    'proves semantic equivalence' \
                    'complete dependency closure proof' \
                    'distributed execution correctness guarantee' \
                    'remote code mobility guarantee' \
                    'formal proof for non-proof tiers' \
                    'claims standalone Valence release ownership' \
                    'Valence owns the evidence codebase index release'; do
                    if grep -F "$overclaim" "$scratch/index.html" >/dev/null; then
                      echo "evidence codebase index HTML overclaimed: $overclaim" >&2
                      exit 1
                    fi
                  done

                  printf '{' > "$scratch/malformed.json"
                  set +e
                  ${cargoOctetUnwrapped}/bin/cargo-octet evidence-index codebase build --artifact "$scratch/malformed.json" --output "$scratch/malformed-index.json"
                  malformed_status="$?"
                  set -e
                  test "$malformed_status" -eq 1
                  jq --exit-status 'any(.issues[]; .severity == "error" and .code == "malformed_json")' "$scratch/malformed-index.json" >/dev/null

                  mkdir -p "$out"
                '';
            structuredMigrationSessionsGate =
              pkgs.runCommand "octet-structured-migration-sessions"
                {
                  nativeBuildInputs = with pkgs; [
                    coreutils
                    diffutils
                    gnugrep
                    jq
                  ];
                }
                ''
                  scratch="$(mktemp -d)"
                  session="$scratch/session.json"
                  cp ${src}/templates/structured-migration-session.json "$session"

                  ${cargoOctetUnwrapped}/bin/cargo-octet migrate session plan --session "$session" --output-format json > "$scratch/plan-a.json"
                  ${cargoOctetUnwrapped}/bin/cargo-octet migrate session plan --session "$session" --output-format json > "$scratch/plan-b.json"
                  cmp "$scratch/plan-a.json" "$scratch/plan-b.json"
                  jq --exit-status '.schema_version == "octet-migration-plan-report/v1" and .session_id == "rename-cargo-slotcar" and (.compatibility_aliases | length == 1) and (.archive_surfaces | length == 1)' "$scratch/plan-a.json" >/dev/null
                  jq --exit-status '.non_claim_boundary | contains("do not prove behavioral correctness") and contains("semantic equivalence")' "$scratch/plan-a.json" >/dev/null

                  ${cargoOctetUnwrapped}/bin/cargo-octet migrate session status --session "$session" --output-format json > "$scratch/status.json"
                  jq --exit-status '.schema_version == "octet-migration-status-report/v1" and .compatibility_alias_count == 1 and .unresolved_todo_count == 1' "$scratch/status.json" >/dev/null
                  ${cargoOctetUnwrapped}/bin/cargo-octet migrate session verify --session "$session" --output-format json > "$scratch/verify.json"
                  jq --exit-status '.valid == true and (.checked_archive_surfaces | index("cairn/archive/2026-01-01-legacy/tasks.md"))' "$scratch/verify.json" >/dev/null

                  ${cargoOctetUnwrapped}/bin/cargo-octet migrate session apply --session "$session" --operation-manifest "$scratch/dry-run-manifest.json" --dry-run --output-format json > "$scratch/dry-run.json"
                  ${cargoOctetUnwrapped}/bin/cargo-octet migrate session apply --session "$session" --operation-manifest "$scratch/apply-manifest-a.json" --output-format json > "$scratch/apply-a.json"
                  ${cargoOctetUnwrapped}/bin/cargo-octet migrate session apply --session "$session" --operation-manifest "$scratch/apply-manifest-b.json" --output-format json > "$scratch/apply-b.json"
                  cmp "$scratch/apply-manifest-a.json" "$scratch/apply-manifest-b.json"
                  jq --exit-status '.pre_write_manifest == true and .dry_run == false and (.operations | length == 1) and (.diagnostics | length == 0)' "$scratch/apply-manifest-a.json" >/dev/null
                  jq --exit-status --slurp '.[0].operations == .[1].operations' "$scratch/dry-run-manifest.json" "$scratch/apply-manifest-a.json" >/dev/null

                  jq '.compatibility[1].coverage = [] | .compatibility[1] |= del(.removal_boundary)' "$session" > "$scratch/missing-alias.json"
                  set +e
                  ${cargoOctetUnwrapped}/bin/cargo-octet migrate session verify --session "$scratch/missing-alias.json" --output-format json > "$scratch/missing-alias-verify.json"
                  missing_alias_status="$?"
                  set -e
                  test "$missing_alias_status" -eq 1
                  jq --exit-status 'any(.diagnostics[]; contains("compatibility.coverage missing")) and any(.diagnostics[]; contains("compatibility.removal_boundary missing"))' "$scratch/missing-alias-verify.json" >/dev/null

                  grep -F 'Structured migration sessions prove reviewed name-pointer and artifact linkage only' ${src}/docs/structured-migration-sessions.md >/dev/null
                  if grep -F 'proves semantic equivalence' ${src}/docs/structured-migration-sessions.md >/dev/null; then
                    echo "structured migration sessions doc overclaimed semantic equivalence" >&2
                    exit 1
                  fi
                  mkdir -p "$out"
                '';
            thirSupplementInventoryGate = mkFingerprintDocsCheck "octet-thir-supplement-inventory-gate" "thir_inventory";
            mirSupplementFixtureGate = mkFingerprintDocsCheck "octet-mir-supplement-fixture-gate" "mir_fixture";
            mirSupplementInventoryGate = mkFingerprintDocsCheck "octet-mir-supplement-inventory-gate" "mir_fixture";
            freshBuildObjectCorpusDeterminism =
              pkgs.runCommand "octet-fresh-build-object-corpus-determinism"
                {
                  nativeBuildInputs = [
                    pkgs.coreutils
                    pkgs.diffutils
                    pkgs.jq
                  ];
                }
                ''
                  repo=${src}
                  scratch="$(mktemp -d)"
                  trap 'rm -rf "$scratch"' EXIT
                  fixture="$repo/cargo-octet/tests/fixtures/object-corpus/expanded-internal-corpus-receipt.json"
                  mapfile -t sources < <(jq -r '.source_paths[]' "$fixture")

                  cd "$repo"
                  ${cargoOctetFreshBuildA}/bin/cargo-octet object corpus receipt --output "$scratch/fresh-a.json" "''${sources[@]}"
                  ${cargoOctetFreshBuildB}/bin/cargo-octet object corpus receipt --output "$scratch/fresh-b.json" "''${sources[@]}"

                  cmp "$scratch/fresh-a.json" "$scratch/fresh-b.json"
                  cmp "$scratch/fresh-a.json" "$fixture"
                  for field in schema schema_version object_count source_paths object_set_hash caveat_counts pure_cache_blocked_count boundary; do
                    jq -S ".$field" "$scratch/fresh-a.json" > "$scratch/a-$field.json"
                    jq -S ".$field" "$scratch/fresh-b.json" > "$scratch/b-$field.json"
                    jq -S ".$field" "$fixture" > "$scratch/fixture-$field.json"
                    cmp "$scratch/a-$field.json" "$scratch/b-$field.json"
                    cmp "$scratch/a-$field.json" "$scratch/fixture-$field.json"
                  done
                  jq -e '.boundary.claims_correctness == false and .boundary.claims_semantic_equivalence == false' "$scratch/fresh-a.json" > /dev/null
                  jq -e '.boundary.detail | contains("do not prove behavioral correctness") and contains("semantic equivalence")' "$scratch/fresh-a.json" > /dev/null

                  echo "fresh_build.receipt_producers=2"
                  echo "fresh_build.byte_stability: PASS"
                  echo "fresh_build.fixture_parity: PASS"
                  echo "fresh_build.compared_fields_stable: PASS"
                  echo "fresh_build.boundary_claims_false: PASS"
                  echo "fresh_build.scope=bounded-local-fresh-build-evidence-replay"
                  mkdir -p "$out"
                '';
            valenceObjectEvidenceGate = pkgs.runCommand "octet-valence-object-evidence" { } ''
              test -d ${objectCorpusGate}
              test -d ${objectCorpusCompatibilityGate}
              test -d ${objectCorpusCaveatTaxonomyGate}
              test -d ${objectCorpusFixtureFreshnessGate}
              test -d ${supplementCompatibilityGate}
              test -d ${valenceEvidenceIndexGate}
              test -d ${valenceEvidenceReleaseGate}
              test -d ${valenceCorePackageReadiness}
              test -d ${thirSupplementInventoryGate}
              test -d ${mirSupplementFixtureGate}
              test -d ${mirSupplementInventoryGate}
              test -d ${freshBuildObjectCorpusDeterminism}

              echo "valence_object_evidence.object_corpus_gate: PASS"
              echo "valence_object_evidence.compatibility_gate: PASS"
              echo "valence_object_evidence.caveat_taxonomy_gate: PASS"
              echo "valence_object_evidence.object_corpus_fixture_freshness_gate: PASS"
              echo "valence_object_evidence.supplement_compatibility_gate: PASS"
              echo "valence_object_evidence.evidence_index_gate: PASS"
              echo "valence_object_evidence.evidence_release_gate: PASS"
              echo "valence_object_evidence.valence_core_package_readiness: PASS"
              echo "valence_object_evidence.thir_supplement_inventory_gate: PASS"
              echo "valence_object_evidence.mir_supplement_fixture_gate: PASS"
              echo "valence_object_evidence.mir_supplement_inventory_gate: PASS"
              echo "valence_object_evidence.fresh_build_determinism_gate: PASS"
              echo "valence_object_evidence.scope=bounded-local-selected-corpus-evidence-replay"
              echo "valence_object_evidence.non_claims=correctness,semantic_equivalence,exhaustive_rust_coverage,cross_machine_determinism,cross_toolchain_determinism"
              mkdir -p "$out"
            '';
            valenceEvidenceReleaseGate =
              pkgs.runCommand "octet-valence-evidence-release"
                {
                  nativeBuildInputs = [
                    pkgs.b3sum
                    pkgs.coreutils
                    pkgs.jq
                  ];
                }
                ''
                  repo=${src}
                  bundle="$out/bundle"
                  mkdir -p "$bundle"

                  test -d ${valenceEvidenceIndexGate}
                  cd "$repo"
                  ${cargoOctetUnwrapped}/bin/cargo-octet evidence-index check --page docs/current-head-valence-evidence.md \
                    | tee "$bundle/evidence-index-check.txt"
                  grep -F "valence_evidence_index.valid: PASS" "$bundle/evidence-index-check.txt" >/dev/null
                  grep -F "valence_evidence_index.issue_count=0" "$bundle/evidence-index-check.txt" >/dev/null
                  cp docs/current-head-valence-evidence.md "$bundle/current-head-valence-evidence.md"

                  page_hash="$(b3sum "$bundle/current-head-valence-evidence.md" | cut -d ' ' -f1)"
                  check_hash="$(b3sum "$bundle/evidence-index-check.txt" | cut -d ' ' -f1)"
                  cat > "$bundle/valence-evidence-release-index.json" <<EOF
                  {
                    "schema": "octet.valence-evidence-release.v1",
                    "scope": "bounded-local-selected-corpus-evidence-replay",
                    "artifacts": [
                      {
                        "path": "current-head-valence-evidence.md",
                        "role": "valence_current_head_evidence",
                        "blake3": "$page_hash"
                      },
                      {
                        "path": "evidence-index-check.txt",
                        "role": "valence_evidence_index_check",
                        "blake3": "$check_hash"
                      }
                    ],
                    "required_gates": [
                      "valence-object-evidence",
                      "valence-evidence-index"
                    ],
                    "non_claims": [
                      "correctness",
                      "semantic_equivalence",
                      "exhaustive_rust_coverage",
                      "cross_machine_determinism",
                      "cross_toolchain_determinism",
                      "general_hir_thir_mir_extraction"
                    ]
                  }
                  EOF

                  jq --exit-status '.schema == "octet.valence-evidence-release.v1"' \
                    "$bundle/valence-evidence-release-index.json" >/dev/null
                  jq --exit-status '.scope == "bounded-local-selected-corpus-evidence-replay"' \
                    "$bundle/valence-evidence-release-index.json" >/dev/null
                  jq --exit-status '.artifacts | length == 2' \
                    "$bundle/valence-evidence-release-index.json" >/dev/null
                  jq --exit-status '(.non_claims | index("correctness")) and (.non_claims | index("semantic_equivalence")) and (.non_claims | index("general_hir_thir_mir_extraction"))' \
                    "$bundle/valence-evidence-release-index.json" >/dev/null
                  jq -r '.artifacts[] | [.path, .blake3] | @tsv' \
                    "$bundle/valence-evidence-release-index.json" \
                    | while IFS=$'\t' read -r artifact_path expected_hash; do
                        actual_hash="$(b3sum "$bundle/$artifact_path" | cut -d ' ' -f1)"
                        test "$actual_hash" = "$expected_hash"
                      done

                  echo "valence_evidence_release.bundle_index: PASS"
                  echo "valence_evidence_release.artifact_hashes: PASS"
                  echo "valence_evidence_release.evidence_index_check: PASS"
                  echo "valence_evidence_release.scope=bounded-local-selected-corpus-evidence-replay"
                  echo "valence_evidence_release.non_claims=correctness,semantic_equivalence,exhaustive_rust_coverage,cross_machine_determinism,cross_toolchain_determinism,general_hir_thir_mir_extraction"
                '';
            valenceCorePackageReadiness =
              pkgs.runCommand "octet-standalone-valence-cutover"
                {
                  nativeBuildInputs = [
                    pkgs.b3sum
                    pkgs.coreutils
                    pkgs.diffutils
                    pkgs.gnugrep
                    pkgs.jq
                    nickel
                  ];
                }
                ''
                  export LC_ALL=C

                  repo=${src}
                  authored="$repo/standards/valence-core-cutover/manifest.ncl"
                  generated="$repo/standards/valence-core-cutover/generated-manifest.json"
                  legacy_manifest="$repo/crates/valence-core/Cargo.toml"
                  lockfile="$repo/Cargo.lock"
                  workflow="$repo/.github/workflows/nix-flake-check.yml"
                  expected_revision=${valenceRevision}
                  expected_source='${valenceCargoSource}'

                  validate_manifest() {
                    checked_manifest="$1"
                    jq --exit-status \
                      --arg revision "$expected_revision" \
                      --arg source "$expected_source" \
                      '.schema_version == "octet-valence-core-cutover-manifest/v2"
                       and .standalone_source.revision == $revision
                       and .standalone_source.cargo_source == $source
                       and .standalone_source.hardened_profile_archive == "cairn/archive/2026-07-12-harden-preserves-integrity-boundaries"
                       and (.modules | length == 3)
                       and all(.modules[]; .compatibility_disposition == "legacy-fixture")
                       and all(.modules[]; (.non_claims | join(" ") | contains("does not prove behavioral correctness")))' \
                      "$checked_manifest" >/dev/null
                  }

                  nickel export --format json "$authored" > "$TMPDIR/generated.json"
                  jq --sort-keys . "$generated" > "$TMPDIR/expected.sorted.json"
                  jq --sort-keys . "$TMPDIR/generated.json" > "$TMPDIR/actual.sorted.json"
                  cmp "$TMPDIR/expected.sorted.json" "$TMPDIR/actual.sorted.json"
                  validate_manifest "$generated"
                  test "$expected_revision" = "${valenceInputRevision}"

                  jq '.standalone_source.revision = "wrong-revision"' "$generated" > "$TMPDIR/wrong-revision.json"
                  if validate_manifest "$TMPDIR/wrong-revision.json"; then
                    echo 'negative fixture unexpectedly accepted wrong Valence revision' >&2
                    exit 1
                  fi

                  jq '.modules[0].non_claims = ["cutover proves compatibility"]' "$generated" > "$TMPDIR/weakened-non-claim.json"
                  if validate_manifest "$TMPDIR/weakened-non-claim.json"; then
                    echo 'negative fixture unexpectedly accepted weakened non-claim' >&2
                    exit 1
                  fi

                  test -d ${valence}/cairn/archive/2026-07-12-harden-preserves-integrity-boundaries
                  test -s ${valence}/crates/valence-core/src/function_object.rs
                  test -s ${valence}/crates/valence-core/src/evidence_index.rs
                  test -s ${valence}/crates/valence-core/src/evidence_codebase_index.rs

                  function_object_b3="b3:$(b3sum ${valence}/crates/valence-core/src/function_object.rs | cut -d' ' -f1)"
                  evidence_index_b3="b3:$(b3sum ${valence}/crates/valence-core/src/evidence_index.rs | cut -d' ' -f1)"
                  evidence_codebase_index_b3="b3:$(b3sum ${valence}/crates/valence-core/src/evidence_codebase_index.rs | cut -d' ' -f1)"
                  test "$function_object_b3" = "$(jq -r '.modules[] | select(.module_or_schema == "function_object") | .standalone_source_b3' "$generated")"
                  test "$evidence_index_b3" = "$(jq -r '.modules[] | select(.module_or_schema == "evidence_index") | .standalone_source_b3' "$generated")"
                  test "$evidence_codebase_index_b3" = "$(jq -r '.modules[] | select(.module_or_schema == "evidence_codebase_index") | .standalone_source_b3' "$generated")"

                  grep -F 'name = "octet-valence-legacy-fixture"' "$legacy_manifest" >/dev/null
                  grep -F 'publish = false' "$legacy_manifest" >/dev/null
                  grep -F 'exclude = ["crates/valence-core"]' "$repo/Cargo.toml" >/dev/null
                  if grep -F 'valence-core = { path = "../crates/valence-core"' "$repo/cargo-octet/Cargo.toml" >/dev/null; then
                    echo 'canonical dependency unexpectedly falls back to hosted path' >&2
                    exit 1
                  fi
                  grep -F "valence-core = { git = \"https://github.com/OnixResearch/valence-public.git\", rev = \"$expected_revision\"" "$repo/cargo-octet/Cargo.toml" >/dev/null
                  test "$(grep -Fc "source = \"$expected_source\"" "$lockfile")" -eq 1
                  test "$(grep -Fc 'name = "valence-core"' "$lockfile")" -eq 1
                  grep -F 'pub(crate) use valence_core::' "$repo/cargo-octet/src/valence_core_shim.rs" >/dev/null
                  grep -Fx '      - run: nix flake check' "$workflow" >/dev/null

                  echo "standalone_valence_cutover.manifest_freshness: PASS"
                  echo "standalone_valence_cutover.exact_source: PASS"
                  echo "standalone_valence_cutover.hardened_profile: PASS"
                  echo "standalone_valence_cutover.unique_package_identity: PASS"
                  echo "standalone_valence_cutover.positive_and_negative_fixtures: PASS"
                  echo "standalone_valence_cutover.non_claims=behavioral_correctness,semantic_equivalence,pure_cache_eligibility,external_consumer_compatibility,cross_toolchain_determinism,release_eligibility"
                  mkdir -p "$out"
                '';
            nickelExportCutoverReadiness =
              pkgs.runCommand "octet-nickel-export-cutover"
                {
                  nativeBuildInputs = [
                    pkgs.b3sum
                    pkgs.coreutils
                    pkgs.diffutils
                    pkgs.gnugrep
                    pkgs.jq
                    nickel
                  ];
                }
                ''
                  export LC_ALL=C

                  repo=${src}
                  authored="$repo/standards/nickel-export-cutover/manifest.ncl"
                  generated="$repo/standards/nickel-export-cutover/generated-manifest.json"
                  migration_authored="$repo/standards/nickel-export-cutover/migration-profiles.ncl"
                  migration_generated="$repo/standards/nickel-export-cutover/generated-migration-profiles.json"
                  crate_manifest="$repo/crates/octet-standards/Cargo.toml"
                  facade="$repo/crates/octet-standards/src/nickel_export.rs"
                  lockfile="$repo/Cargo.lock"
                  expected_revision=${nickelExportRevision}
                  expected_source='${nickelExportCargoSource}'
                  expected_package_count=2
                  expected_selected_count=1

                  validate_manifest() {
                    checked_manifest="$1"
                    jq --exit-status \
                      --arg revision "$expected_revision" \
                      --arg source "$expected_source" \
                      --arg input_revision "${nickelExportInputRevision}" \
                      --arg local_override "diagnostic-only" \
                      --arg canonical_owner "nickel-export-core" \
                      --argjson package_count "$expected_package_count" \
                      --argjson selected_count "$expected_selected_count" \
                      '.schema_version == "octet-nickel-export-cutover-manifest/v1"
                       and .standalone_source.revision == $revision
                       and .standalone_source.revision == $input_revision
                       and .standalone_source.cargo_source == $source
                       and (.standalone_source.local_override_policy | contains($local_override))
                       and (.standalone_source.local_override_policy | contains("MUST NOT"))
                       and (.packages | length == $package_count)
                       and ([.packages[] | select(.selected_by_octet)] | length == $selected_count)
                       and .compatibility_facade.canonical_owner == $canonical_owner
                       and .compatibility_facade.disposition == "delegating legacy projection only"
                       and (.non_claims | join(" ") | contains("does not prove evaluator correctness"))' \
                      "$checked_manifest" >/dev/null
                  }

                  nickel export --format json "$authored" > "$TMPDIR/generated.json"
                  jq --sort-keys . "$generated" > "$TMPDIR/expected.sorted.json"
                  jq --sort-keys . "$TMPDIR/generated.json" > "$TMPDIR/actual.sorted.json"
                  cmp "$TMPDIR/expected.sorted.json" "$TMPDIR/actual.sorted.json"
                  validate_manifest "$generated"
                  nickel export --format json "$migration_authored" > "$TMPDIR/migration-generated.json"
                  jq --sort-keys . "$migration_generated" > "$TMPDIR/migration-expected.sorted.json"
                  jq --sort-keys . "$TMPDIR/migration-generated.json" > "$TMPDIR/migration-actual.sorted.json"
                  cmp "$TMPDIR/migration-expected.sorted.json" "$TMPDIR/migration-actual.sorted.json"
                  jq --exit-status \
                    --arg revision "$expected_revision" \
                    '.schema_version == "octet-nickel-export-consumer-migrations/v1"
                     and .standalone_revision == $revision
                     and (.profiles | length == 4)
                     and ([.profiles[].consumer] | unique | length == 4)
                     and ([.profiles[].handoff_commit] | unique | length == 4)
                     and all(.profiles[]; (.handoff_commit | test("^[0-9a-f]{40}$"))
                       and .handoff_state == "consumer-owned package committed locally; publication remains consumer-owned"
                       and (.retained_authority | length > 0)
                       and (.dual_run_validation | length > 0)
                       and (.non_claim | contains("does not prove evaluator equivalence")))' \
                    "$migration_generated" >/dev/null

                  jq '.standalone_source.revision = "floating-main"' "$generated" > "$TMPDIR/floating.json"
                  if validate_manifest "$TMPDIR/floating.json"; then
                    echo 'negative fixture unexpectedly accepted floating Nickel Export revision' >&2
                    exit 1
                  fi
                  jq '.standalone_source.local_override_policy = "local path override accepted"' "$generated" > "$TMPDIR/local-override.json"
                  if validate_manifest "$TMPDIR/local-override.json"; then
                    echo 'negative fixture unexpectedly accepted local release override' >&2
                    exit 1
                  fi
                  jq '.non_claims = ["cutover proves evaluator correctness"]' "$generated" > "$TMPDIR/weakened-non-claim.json"
                  if validate_manifest "$TMPDIR/weakened-non-claim.json"; then
                    echo 'negative fixture unexpectedly accepted weakened non-claim' >&2
                    exit 1
                  fi

                  test -s ${nickel-export}/cairn/specs/deterministic-nickel-export/spec.md
                  core_b3="b3:$(b3sum ${nickel-export}/crates/nickel-export-core/src/lib.rs | cut -d' ' -f1)"
                  shell_b3="b3:$(b3sum ${nickel-export}/crates/nickel-export/src/lib.rs | cut -d' ' -f1)"
                  test "$core_b3" = "$(jq -r '.packages[] | select(.name == "nickel-export-core") | .source_b3' "$generated")"
                  test "$shell_b3" = "$(jq -r '.packages[] | select(.name == "nickel-export") | .source_b3' "$generated")"

                  grep -F "nickel-export-core = { git = \"https://github.com/OnixResearch/nickel-export\", rev = \"$expected_revision\"" "$crate_manifest" >/dev/null
                  if grep -F 'nickel-export-core = { path =' "$crate_manifest" >/dev/null; then
                    echo 'canonical Nickel Export dependency unexpectedly uses a local path' >&2
                    exit 1
                  fi
                  test "$(grep -Fc "source = \"$expected_source\"" "$lockfile")" -eq "$expected_selected_count"
                  test "$(grep -Fc 'name = "nickel-export-core"' "$lockfile")" -eq "$expected_selected_count"
                  if grep -F 'name = "nickel-export"' "$lockfile" >/dev/null; then
                    echo 'ordinary Octet graph unexpectedly includes the external evaluator shell' >&2
                    exit 1
                  fi

                  grep -F 'use nickel_export_core as canonical;' "$facade" >/dev/null
                  grep -F 'canonical::build_receipt' "$facade" >/dev/null
                  grep -F 'canonical::build_manifest' "$facade" >/dev/null
                  grep -F 'canonical::project_octet_manifest' "$facade" >/dev/null
                  if grep -E 'SECRET_MARKERS|blake3::hash|artifact_text_identity|canonical::ExportReceipt[[:space:]]*\{' "$facade" >/dev/null; then
                    echo 'Octet compatibility facade contains forked canonical export logic' >&2
                    exit 1
                  fi

                  echo "nickel_export_cutover.manifest_freshness: PASS"
                  echo "nickel_export_cutover.exact_source: PASS"
                  echo "nickel_export_cutover.unique_package_identity: PASS"
                  echo "nickel_export_cutover.delegating_facade: PASS"
                  echo "nickel_export_cutover.migration_profiles: PASS"
                  echo "nickel_export_cutover.runtime_shell_absent: PASS"
                  mkdir -p "$out"
                '';
            roadmapAgentSurfacesDocs = craneLib.buildPackage {
              inherit
                src
                cargoArtifacts
                buildInputs
                cargoVendorDir
                ;
              cargoToml = ./Cargo.toml;
              pname = "octet-roadmap-agent-surfaces-docs";
              version = "0.1.0";
              nativeBuildInputs = checkNativeBuildInputs;
              cargoExtraArgs = "-p cargo-octet";
              doNotPostBuildInstallCargoBinaries = true;
              buildPhaseCargoCommand = ":";
              cargoTestExtraArgs = "--test unison_roadmap roadmap_docs_transcript_covers_agent_surfaces -- --nocapture";
              installPhaseCommand = ''
                mkdir -p "$out"
              '';
              LD_LIBRARY_PATH = rustToolchainLibPath;
              RUSTUP_TOOLCHAIN = rustupToolchain;
              preCheck = ''
                export PATH="${dylintShimPath}:$PATH"
                export CARGO_OCTET_BIN="${cargoOctetUnwrapped}/bin/cargo-octet"
              '';
            };
            fingerprintDocsDownstreamSmokeCheck =
              pkgs.runCommand "octet-fingerprint-docs-downstream-smoke-check" { }
                ''
                  ${fingerprintDocsDownstreamSmoke}/bin/octet-fingerprint-docs-downstream-smoke
                  mkdir -p "$out"
                '';
            fingerprintDocsReviewGateDocsCheck =
              pkgs.runCommand "octet-fingerprint-docs-review-gate-docs-check" { }
                ''
                  cd ${self}
                  grep -F "For fingerprint docs/examples, use one review command:" docs/contributor-guide.md
                  grep -F "nix build .#checks.x86_64-linux.docs-examples --no-link -L" docs/contributor-guide.md
                  grep -F "CI/operator gate for executable" docs/contributor-guide.md
                  grep -F "focused flake checks or direct Cargo filters only after the aggregate gate" docs/contributor-guide.md
                  grep -F "2026-05-05: Function fingerprint docs and checks" CHANGELOG.md
                  grep -F "nix build .#checks.x86_64-linux.docs-examples --no-link -L" CHANGELOG.md
                  mkdir -p "$out"
                '';
            releaseCandidateSmokeCheck = pkgs.runCommand "octet-release-candidate-smoke" { } ''
              repo=${self}
              grep -F 'Candidate version: `0.1.0`' "$repo/docs/release-candidate.md"
              grep -F 'nix build .#checks.x86_64-linux.release-candidate-smoke -L --no-link' "$repo/docs/release-candidate.md"
              grep -F 'nix flake check -L' "$repo/docs/release-candidate.md"
              grep -F 'cargo-octet' "$repo/docs/release-candidate.md"
              grep -F 'cargo-tigerstyle' "$repo/docs/release-candidate.md"
              grep -F 'slotcar' "$repo/docs/release-candidate.md"
              grep -F 'tigerstyle' "$repo/docs/release-candidate.md"
              test -x ${cargoOctetUnwrapped}/bin/cargo-octet
              test -x ${cargoOctetUnwrapped}/bin/cargo-slotcar
              test -x ${cargoOctetUnwrapped}/bin/cargo-tigerstyle
              ${cargoOctetUnwrapped}/bin/cargo-octet --version | tee "$TMPDIR/cargo-octet.version"
              ${cargoOctetUnwrapped}/bin/cargo-slotcar --version | tee "$TMPDIR/cargo-slotcar.version"
              ${cargoOctetUnwrapped}/bin/cargo-tigerstyle --version | tee "$TMPDIR/cargo-tigerstyle.version"
              grep -Fx "cargo-octet 0.1.0" "$TMPDIR/cargo-octet.version"
              grep -Fx "cargo-octet 0.1.0" "$TMPDIR/cargo-slotcar.version"
              grep -Fx "cargo-octet 0.1.0" "$TMPDIR/cargo-tigerstyle.version"
              test -d ${octet}
              test -d ${cargoOctetUnwrapped}
              mkdir -p "$out"
            '';
            warningClean = craneLib.buildPackage {
              inherit
                src
                cargoArtifacts
                buildInputs
                cargoVendorDir
                ;
              cargoToml = ./Cargo.toml;
              pname = "octet-warning-clean";
              version = "0.1.0";
              nativeBuildInputs = checkNativeBuildInputs;
              doCheck = false;
              doNotPostBuildInstallCargoBinaries = true;
              buildPhaseCargoCommand = "cargo check --workspace --all-targets";
              installPhaseCommand = ''
                mkdir -p "$out"
              '';
              RUSTFLAGS = "-D warnings";
              LD_LIBRARY_PATH = rustToolchainLibPath;
              RUSTUP_TOOLCHAIN = rustupToolchain;
              preBuild = ''
                export PATH="${dylintShimPath}:$PATH"
              '';
            };
            warningCleanRegression =
              pkgs.runCommand "octet-warning-clean-regression"
                {
                  nativeBuildInputs = [
                    cargo-shim
                    rustup-shim
                    rustToolchain
                  ];
                  LD_LIBRARY_PATH = rustToolchainLibPath;
                  RUSTUP_TOOLCHAIN = rustupToolchain;
                }
                ''
                  fixture_version="0.1.0"
                  fixture_edition="2024"
                  scratch="$(mktemp -d)"
                  trap 'rm -rf "$scratch"' EXIT
                  mkdir -p "$scratch/src"
                  cat > "$scratch/Cargo.toml" <<EOF
                  [package]
                  name = "octet-warning-clean-negative-control"
                  version = "$fixture_version"
                  edition = "$fixture_edition"
                  EOF
                  cat > "$scratch/src/lib.rs" <<'EOF'
                  fn warning_clean_negative_control_dead_helper() {}
                  EOF

                  set +e
                  RUSTFLAGS="-D warnings" cargo check --manifest-path "$scratch/Cargo.toml" > "$scratch/stdout" 2> "$scratch/stderr"
                  status="$?"
                  set -e
                  test "$status" -ne 0
                  grep -F 'function `warning_clean_negative_control_dead_helper` is never used' "$scratch/stderr"
                  echo "warning_clean.regression_dead_code: PASS"
                  mkdir -p "$out"
                '';
            mkAeneasLibraryCheck =
              {
                pname,
                cargoPackage,
                cargoTestExtraArgs,
                extraPreCheck ? "",
              }:
              craneLib.buildPackage {
                inherit
                  src
                  cargoArtifacts
                  buildInputs
                  cargoVendorDir
                  pname
                  cargoTestExtraArgs
                  ;
                cargoToml = ./Cargo.toml;
                version = "0.1.0";
                nativeBuildInputs = checkNativeBuildInputs;
                cargoExtraArgs = "-p ${cargoPackage}";
                doNotPostBuildInstallCargoBinaries = true;
                buildPhaseCargoCommand = ":";
                installPhaseCommand = ''
                  mkdir -p "$out"
                '';
                LD_LIBRARY_PATH = rustToolchainLibPath;
                RUSTUP_TOOLCHAIN = rustupToolchain;
                preCheck = ''
                  export PATH="${dylintShimPath}:$PATH"
                  ${extraPreCheck}
                '';
              };
            aeneasLeanPureCompatibilityCheck = mkAeneasLibraryCheck {
              pname = "octet-aeneas-lean-pure-compatibility";
              cargoPackage = "octet-standards";
              cargoTestExtraArgs = "--lib aeneas -- --nocapture";
            };
            aeneasLeanLiveCompatibilityCheck = mkAeneasLibraryCheck {
              pname = "octet-aeneas-lean-live-compatibility";
              cargoPackage = "cargo-octet";
              cargoTestExtraArgs = "--lib live_provider_ -- --ignored --nocapture --test-threads=1";
              extraPreCheck = ''
                export OCTET_AENEAS_CHARON="${aeneasLeanProviderClosure}/share/octet/aeneas-lean/charon"
                export OCTET_AENEAS_AENEAS="${aeneasLeanProviderClosure}/share/octet/aeneas-lean/aeneas"
                export OCTET_AENEAS_LEAN="${aeneasLeanProviderClosure}/share/octet/aeneas-lean/lake"
                export OCTET_AENEAS_LEAN_LIBRARY="${aeneasLeanProviderClosure}/share/octet/aeneas-lean/aeneas-lean-library"
                export OCTET_AENEAS_NETWORK_SANDBOX="${aeneasLeanProviderClosure}/share/octet/aeneas-lean/bwrap"
              '';
            };
            charonSemanticPureCheck = mkAeneasLibraryCheck {
              pname = "octet-charon-semantic-pure";
              cargoPackage = "cargo-octet";
              cargoTestExtraArgs = "--lib charon_semantic -- --nocapture";
            };
            charonSemanticLiveCheck = mkAeneasLibraryCheck {
              pname = "octet-charon-semantic-live";
              cargoPackage = "cargo-octet";
              cargoTestExtraArgs = "--lib live_charon_semantic -- --ignored --nocapture --test-threads=1";
              extraPreCheck = ''
                export OCTET_AENEAS_CHARON="${aeneasLeanProviderClosure}/share/octet/aeneas-lean/charon"
                export OCTET_AENEAS_NETWORK_SANDBOX="${aeneasLeanProviderClosure}/share/octet/aeneas-lean/bwrap"
              '';
            };
            charonSemanticAnalysisCheck =
              pkgs.runCommand "octet-charon-semantic-analysis-check"
                {
                  nativeBuildInputs = with pkgs; [
                    b3sum
                    diffutils
                    jq
                    nickel
                  ];
                  LD_LIBRARY_PATH = rustToolchainLibPath;
                }
                ''
                  cd ${./.}
                  negative_case_count=30
                  generated_profile=$(mktemp)
                  generated_negative_cases=$(mktemp)
                  generated_source_token_receipt=$(mktemp)
                  generated_comparison_parent=$(mktemp)
                  nickel typecheck standards/charon-semantic/profile.ncl
                  nickel export --format json standards/charon-semantic/profile.ncl > "$generated_profile"
                  diff -u standards/charon-semantic/generated/profile.json "$generated_profile"
                  nickel typecheck standards/charon-semantic/negative-cases.ncl
                  nickel export --format json standards/charon-semantic/negative-cases.ncl > "$generated_negative_cases"
                  diff -u standards/charon-semantic/generated/negative-cases.json "$generated_negative_cases"
                  nickel typecheck standards/charon-semantic/comparison/legacy-source-token-receipt.ncl
                  nickel export --format json standards/charon-semantic/comparison/legacy-source-token-receipt.ncl \
                    > "$generated_source_token_receipt"
                  diff -u standards/charon-semantic/generated/legacy-source-token-receipt.json \
                    "$generated_source_token_receipt"
                  nickel typecheck standards/charon-semantic/comparison/comparison-parent.ncl
                  nickel export --format json standards/charon-semantic/comparison/comparison-parent.ncl \
                    > "$generated_comparison_parent"
                  diff -u standards/charon-semantic/generated/comparison-parent.json "$generated_comparison_parent"
                  expected_parent_identity="b3:$(b3sum "$generated_source_token_receipt" | cut -d ' ' -f 1)"
                  jq --exit-status --arg expected "$expected_parent_identity" \
                    '.identity == $expected and .authority == "source-token"' \
                    "$generated_comparison_parent" >/dev/null
                  jq --exit-status --argjson negative_case_count "$negative_case_count" \
                    '.schema_version == "octet-charon-semantic-negative-cases/v1"
                      and (.cases | length) == $negative_case_count
                      and ([.cases[].layer] | index("snapshot") != null)
                      and ([.cases[].layer] | index("comparison") != null)
                      and ([.cases[].layer] | index("provenance") != null)' \
                    "$generated_negative_cases" >/dev/null
                  test -d ${charonSemanticPureCheck}
                  test -d ${charonSemanticLiveCheck}
                  test -s standards/charon-semantic/fixtures/valid/artifacts/evidence-rail-receipt.json
                  mkdir -p "$out"
                  ${cargoOctetUnwrapped}/bin/cargo-octet octet artifact verify \
                    --artifact-dir standards/charon-semantic/fixtures/valid/artifacts \
                    --format json \
                    --emit-valence-provenance "$out/valence-handoff.json" \
                    > "$out/artifact-verification.json"
                  jq --exit-status '.status == "valid" and .diagnostics == []' \
                    "$out/artifact-verification.json" >/dev/null
                  diff -u standards/charon-semantic/generated/valence-handoff.json "$out/valence-handoff.json"
                  jq --exit-status '
                    .verification_status == "valid"
                      and ([.provenance_records[] | select(.source_id | startswith("octet.charon_semantic")) | .evidence_tier] | all(. == "transcript_assertion"))
                      and ([.provenance_records[].receipt.charon_semantic.verification_role?] | map(select(. != null)) | all(. == "recorded_only"))
                      and ([.provenance_records[].evidence_tier] | index("formal_proof") == null)
                  ' "$out/valence-handoff.json" >/dev/null
                '';
            aeneasLeanCompatibilityMatrixCheck =
              pkgs.runCommand "octet-aeneas-lean-compatibility-matrix-check" { }
                ''
                  test -d ${aeneasLeanPureCompatibilityCheck}
                  test -d ${aeneasLeanLiveCompatibilityCheck}
                  test -e ${proofRailManifestCheck}
                  test -s ${aeneasLeanProviderClosureCheck}/identity.json
                  mkdir -p "$out"
                '';
            cairnTraceabilityCheck =
              pkgs.runCommand "octet-cairn-traceability-check"
                {
                  nativeBuildInputs = with pkgs; [
                    coreutils
                    jq
                  ];
                }
                ''
                  cairn_bin=${cairn.packages.${system}.cairn}/bin/cairn
                  repository=${./.}
                  negative="$TMPDIR/negative-repository"
                  mkdir -p "$out" "$negative"

                  (
                    cd "$repository"
                    "$cairn_bin" tracey coverage --root . --json
                  ) > "$out/positive.json"
                  jq --exit-status '.valid == true and .missing == [] and .dangling == []' \
                    "$out/positive.json" >/dev/null

                  cp -R --no-preserve=mode "$repository/." "$negative"
                  substituteInPlace "$negative/cargo-octet/src/admit.rs" \
                    --replace-fail \
                    '// r[impl octet.effect_closure.cli.check_failure]' \
                    '// negative fixture removed the direct-check failure implementation marker'
                  substituteInPlace "$negative/tests/consumer_integration.rs" \
                    --replace-fail \
                    '// r[verify octet.effect_closure.cli.check_failure]' \
                    '// negative fixture removed the direct-check failure verification marker'

                  if (
                    cd "$negative"
                    "$cairn_bin" tracey coverage --root . --json
                  ) > "$out/negative.json" 2> "$out/negative.stderr"; then
                    echo 'traceability negative fixture unexpectedly passed' >&2
                    exit 1
                  fi
                  jq --exit-status \
                    '.valid == false
                     and .missing == ["octet.effect_closure.cli.check_failure"]
                     and .dangling == []' \
                    "$out/negative.json" >/dev/null
                '';
            cairnCobPilotCheck =
              pkgs.runCommand "octet-cairn-cob-pilot-check"
                {
                  nativeBuildInputs = with pkgs; [
                    coreutils
                    diffutils
                    gitMinimal
                    jq
                    gnugrep
                    nickel
                  ];
                }
                ''
                  fixture=${src}/fixtures/cairn-cob-pilot
                  scratch="$(mktemp -d)"
                  audit_store="$scratch/audit.git"
                  generated_manifest="$scratch/manifest.json"
                  bundle_heads="$scratch/bundle-heads.txt"
                  bundle_verify="$scratch/bundle-verify.txt"
                  base_ref='refs/heads/main'
                  base_oid="$(jq -er '.base_git_oid' "$fixture/manifest.json")"
                  namespace_ref="$(jq -er '.namespace_ref' "$fixture/manifest.json")"
                  namespace_oid="$(jq -er '.update_operation_id' "$fixture/manifest.json")"
                  expected_head_count=2

                  nickel typecheck "$fixture/manifest.ncl"
                  nickel export --format json "$fixture/manifest.ncl" > "$generated_manifest"
                  cmp "$fixture/manifest.json" "$generated_manifest"
                  nickel typecheck "$fixture/transport.ncl"
                  nickel export --format json "$fixture/transport.ncl" > "$scratch/transport.json"
                  cmp "$fixture/evidence/transport.json" "$scratch/transport.json"

                  git init --bare --initial-branch=fixture-audit "$audit_store"
                  git --git-dir="$audit_store" bundle verify "$fixture/octet-cob.bundle" > "$bundle_verify"
                  git bundle list-heads "$fixture/octet-cob.bundle" > "$bundle_heads"
                  test "$(wc -l < "$bundle_heads")" -eq "$expected_head_count"
                  grep -F "$base_oid $base_ref" "$bundle_heads" >/dev/null
                  grep -F "$namespace_oid $namespace_ref" "$bundle_heads" >/dev/null
                  git --git-dir="$audit_store" fetch --no-tags "$fixture/octet-cob.bundle" \
                    "+$base_ref:$base_ref" "+refs/namespaces/*:refs/namespaces/*"
                  test "$(git --git-dir="$audit_store" rev-parse "$base_ref")" = "$base_oid"
                  test "$(git --git-dir="$audit_store" rev-parse "$namespace_ref")" = "$namespace_oid"
                  test -z "$(git --git-dir="$audit_store" remote)"
                  git --git-dir="$audit_store" fsck --strict
                  test "$(git --git-dir="$audit_store" rev-list --all --count)" -eq 3
                  test "$(git --git-dir="$audit_store" rev-list --max-parents=0 --all)" = "$base_oid"
                  test -z "$(git --git-dir="$audit_store" ls-tree -r --name-only "$base_oid")"
                  mkdir -p "$out"
                  cp "$generated_manifest" "$bundle_heads" "$bundle_verify" "$out/"
                '';
            cairnCobConflictPilotCheck =
              pkgs.runCommand "octet-cairn-cob-conflict-pilot-check"
                {
                  nativeBuildInputs = with pkgs; [
                    coreutils
                    diffutils
                    gitMinimal
                    jq
                    gnugrep
                    nickel
                  ];
                }
                ''
                  fixture=${src}/fixtures/cairn-cob-conflict-pilot
                  scratch="$(mktemp -d)"
                  audit_store="$scratch/audit.git"
                  generated_manifest="$scratch/manifest.json"
                  bundle_heads="$scratch/bundle-heads.txt"
                  bundle_verify="$scratch/bundle-verify.txt"
                  base_ref='refs/heads/main'
                  base_oid="$(jq -er '.synthetic_base_git_oid' "$fixture/manifest.json")"
                  creator_ref="$(jq -er '.creator.ref_name' "$fixture/manifest.json")"
                  delegate_ref="$(jq -er '.delegate.ref_name' "$fixture/manifest.json")"
                  creator_operation="$(jq -er '.creator.operation_id' "$fixture/manifest.json")"
                  delegate_operation="$(jq -er '.delegate.operation_id' "$fixture/manifest.json")"
                  resolution_operation="$(jq -er '.resolution.operation_id' "$fixture/manifest.json")"
                  resolution_parents="$(jq -er '.resolution.direct_parents | join(" ")' "$fixture/manifest.json")"
                  expected_head_count=3

                  nickel typecheck "$fixture/manifest.ncl"
                  nickel export --format json "$fixture/manifest.ncl" > "$generated_manifest"
                  cmp "$fixture/manifest.json" "$generated_manifest"
                  nickel typecheck "$fixture/transport.ncl"
                  nickel export --format json "$fixture/transport.ncl" > "$scratch/transport.json"
                  cmp "$fixture/evidence/transport.json" "$scratch/transport.json"

                  git init --bare --initial-branch=fixture-audit "$audit_store"
                  git --git-dir="$audit_store" bundle verify "$fixture/octet-cob-conflict.bundle" > "$bundle_verify"
                  git bundle list-heads "$fixture/octet-cob-conflict.bundle" > "$bundle_heads"
                  test "$(wc -l < "$bundle_heads")" -eq "$expected_head_count"
                  grep -F "$base_oid $base_ref" "$bundle_heads" >/dev/null
                  grep -F "$resolution_operation $creator_ref" "$bundle_heads" >/dev/null
                  grep -F "$delegate_operation $delegate_ref" "$bundle_heads" >/dev/null
                  git --git-dir="$audit_store" fetch --no-tags "$fixture/octet-cob-conflict.bundle" \
                    "+$base_ref:$base_ref" "+refs/namespaces/*:refs/namespaces/*"
                  test "$(git --git-dir="$audit_store" rev-parse "$base_ref")" = "$base_oid"
                  test "$(git --git-dir="$audit_store" rev-parse "$creator_ref")" = "$resolution_operation"
                  test "$(git --git-dir="$audit_store" rev-parse "$delegate_ref")" = "$delegate_operation"
                  test -z "$(git --git-dir="$audit_store" remote)"
                  git --git-dir="$audit_store" merge-base --is-ancestor "$creator_operation" "$resolution_operation"
                  git --git-dir="$audit_store" merge-base --is-ancestor "$delegate_operation" "$resolution_operation"
                  test "$(git --git-dir="$audit_store" rev-list --parents -n 1 "$resolution_operation")" = \
                    "$resolution_operation $resolution_parents"
                  git --git-dir="$audit_store" fsck --strict
                  test "$(git --git-dir="$audit_store" rev-list --all --count)" -eq 5
                  test "$(git --git-dir="$audit_store" rev-list --max-parents=0 --all)" = "$base_oid"
                  test -z "$(git --git-dir="$audit_store" ls-tree -r --name-only "$base_oid")"
                  mkdir -p "$out"
                  cp "$generated_manifest" "$bundle_heads" "$bundle_verify" "$out/"
                '';
          in
          {
            octet = octet;
            slotcar = octet;
            tigerstyle = octet;

            nextest = craneLib.cargoNextest {
              inherit
                src
                cargoArtifacts
                buildInputs
                cargoVendorDir
                ;
              nativeBuildInputs = checkNativeBuildInputs;
              partitions = 1;
              partitionType = "count";
              CARGO_OCTET_BIN = "${cargoOctetUnwrapped}/bin/cargo-octet";
              LD_LIBRARY_PATH = rustToolchainLibPath;
              RUSTUP_TOOLCHAIN = rustupToolchain;
              preCheck = ''
                export PATH="${dylintShimPath}:$PATH"
                export CARGO_OCTET_BIN="${cargoOctetUnwrapped}/bin/cargo-octet"
              '';
            };

            ui = craneLib.buildPackage {
              inherit
                src
                cargoArtifacts
                buildInputs
                cargoVendorDir
                ;
              cargoToml = ./Cargo.toml;
              pname = "octet-ui";
              version = "0.1.0";
              nativeBuildInputs = checkNativeBuildInputs;
              doNotPostBuildInstallCargoBinaries = true;
              buildPhaseCargoCommand = ":";
              cargoTestExtraArgs = "--test ui -- --ignored --nocapture";
              installPhaseCommand = ''
                mkdir -p "$out"
              '';
              LD_LIBRARY_PATH = rustToolchainLibPath;
              RUSTUP_TOOLCHAIN = rustupToolchain;
              preCheck = ''
                export PATH="${dylintShimPath}:$PATH"
                export OCTET_UI_DRIVER="${uiDriver}/bin/octet-ui-driver"
                export OCTET_UI_LIBRARY="${octetUiLibrary}/lib/${octetUiLibraryName}"
                export TIGERSTYLE_UI_DRIVER="$OCTET_UI_DRIVER"
                export TIGERSTYLE_UI_LIBRARY="$OCTET_UI_LIBRARY"
              '';
            };

            fingerprint-lifecycle-docs = fingerprintLifecycleDocs;
            fingerprint-human-summary-docs = fingerprintHumanSummaryDocs;
            fingerprint-warning-summary-docs = fingerprintWarningSummaryDocs;
            fingerprint-json-output-docs = fingerprintJsonOutputDocs;
            fingerprint-source-scanner-docs = fingerprintSourceScannerDocs;
            release-operator-smoke-docs = releaseOperatorSmokeDocs;
            object-corpus-gate = objectCorpusGate;
            object-corpus-compatibility-gate = objectCorpusCompatibilityGate;
            object-corpus-caveat-taxonomy-gate = objectCorpusCaveatTaxonomyGate;
            object-corpus-fixture-freshness = objectCorpusFixtureFreshnessGate;
            supplement-compatibility-gate = supplementCompatibilityGate;
            valence-evidence-index = valenceEvidenceIndexGate;
            evidence-codebase-index = evidenceCodebaseIndexGate;
            structured-migration-sessions = structuredMigrationSessionsGate;
            # r[verify octet.nickel_toolchain.cohort]
            nickel-cohort-exact =
              pkgs.runCommand "octet-nickel-cohort-exact"
                {
                  nativeBuildInputs = [ nickel ];
                }
                ''
                  test '${nickel-1-17-0.rev}' = '1320a983e6c3d1e2fb53dd2464b084b4903b1426'
                  test "$(command -v nickel)" = '${nickel}/bin/nickel'
                  test "$(nickel --version)" = 'nickel-lang-cli nickel 1.17.0 (rev 1320a98)'
                  if grep -q 'pkgs\.nickel' ${self}/flake.nix; then
                    echo 'older, floating, ambient, or mixed Nickel evaluator reference found' >&2
                    exit 1
                  fi
                  mkdir -p "$out"
                '';
            # r[verify octet.nickel_toolchain.compatibility]
            # r[verify octet.nickel_toolchain.evidence]
            nickel-toolchain-fixtures =
              pkgs.runCommand "octet-nickel-toolchain-fixtures"
                {
                  nativeBuildInputs = [ nickel ];
                }
                ''
                  fixtures=${src}/standards/nickel-toolchain/fixtures
                  for fixture in "$fixtures"/positive/*.ncl; do
                    nickel typecheck "$fixture"
                    nickel export --format json "$fixture" > /dev/null
                  done
                  for fixture in "$fixtures"/negative/*.ncl; do
                    if nickel eval "$fixture" >negative.out 2>negative.err; then
                      echo "negative Nickel fixture unexpectedly passed: $fixture" >&2
                      exit 1
                    fi
                    test -s negative.err
                  done
                  mkdir -p "$out"
                '';
            valence-evidence-release = valenceEvidenceReleaseGate;
            valence-core-package-readiness = valenceCorePackageReadiness;
            nickel-export-cutover-readiness = nickelExportCutoverReadiness;
            thir-supplement-inventory-gate = thirSupplementInventoryGate;
            mir-supplement-fixture-gate = mirSupplementFixtureGate;
            mir-supplement-inventory-gate = mirSupplementInventoryGate;
            fresh-build-object-corpus-determinism = freshBuildObjectCorpusDeterminism;
            valence-object-evidence = valenceObjectEvidenceGate;
            roadmap-agent-surfaces-docs = roadmapAgentSurfacesDocs;
            release-candidate-smoke = releaseCandidateSmokeCheck;
            warning-clean = warningClean;
            warning-clean-regression = warningCleanRegression;

            fingerprint-docs-downstream-smoke = fingerprintDocsDownstreamSmokeCheck;
            fingerprint-docs-review-gate-docs = fingerprintDocsReviewGateDocsCheck;

            docs-examples = pkgs.runCommand "octet-docs-examples" { } ''
              test -d ${fingerprintLifecycleDocs}
              test -d ${fingerprintHumanSummaryDocs}
              test -d ${fingerprintWarningSummaryDocs}
              test -d ${fingerprintJsonOutputDocs}
              test -d ${fingerprintSourceScannerDocs}
              test -d ${releaseOperatorSmokeDocs}
              test -d ${roadmapAgentSurfacesDocs}
              test -d ${fingerprintDocsDownstreamSmokeCheck}
              test -d ${fingerprintDocsReviewGateDocsCheck}
              mkdir -p "$out"
            '';

            clippy = craneLib.cargoClippy {
              inherit
                src
                cargoArtifacts
                nativeBuildInputs
                buildInputs
                cargoVendorDir
                ;
              cargoClippyExtraArgs = "--all-targets -- -D warnings -W clippy::pedantic ${productionSafetyClippyArgs}";
            };

            fmt = craneLib.cargoFmt {
              src = fmtSrc;
            };

            policy-registry = policyRegistryCheck;
            cargo-octet-self-check = cargoOctetSelfCheck;
            cairn-traceability = cairnTraceabilityCheck;
            cairn-cob-pilot = cairnCobPilotCheck;
            cairn-cob-conflict-pilot = cairnCobConflictPilotCheck;
            ast-grep-structural-rail = astGrepStructuralRailCheck;
            wasm-artifact-rail = wasmArtifactRailCheck;
            dynamic-evidence-profiles = dynamicEvidenceProfilesCheck;
            sarif-profiles = sarifProfilesCheck;
            coverage-profiles = coverageProfilesCheck;
            benchmark-profiles = benchmarkProfilesCheck;
            hegel-profiles = hegelProfilesCheck;
            mirai-semantic-pilot = miraiPilotCheck;
            mutation-profiles = mutationProfilesCheck;
            strict-profile-catalog = strictProfileCatalogCheck;
            nominal-domain-catalog = nominalDomainCatalogCheck;
            project-architecture-policy = projectArchitecturePolicyCheck;
            assurance-profile-onixos-lifecycle = assuranceProfileOnixOSLifecycleCheck;
            clippy-restriction-profiles = clippyRestrictionProfilesCheck;
            effect-closure-catalog = effectClosureCatalogCheck;
            downstream-proc-macro = downstreamProcMacroCheck;
            proof-rail-manifest = proofRailManifestCheck;
            aeneas-lean-provider-closure = aeneasLeanProviderClosureCheck;
            aeneas-lean-compatibility-matrix = aeneasLeanCompatibilityMatrixCheck;
            aeneas-consumer-contract = aeneasConsumerContractCheck;
            charon-semantic-analysis = charonSemanticAnalysisCheck;
            verus-toolchain-profile = verusToolchainProfileCheck;
            production-verus-negative-proof = productionVerusNegativeProofCheck;
            production-verus-negative-prove = productionVerusNegativeProveCheck;
            production-verus-multi-rail-prove = productionVerusMultiRailProveCheck;
            standards-fixtures = standardsFixtureCheck;
            octet-standards-nix-closure-manifest =
              pkgs.runCommand "octet-standards-nix-closure-manifest-check" { }
                ''
                  test -s ${octetStandardsNixClosureManifest}/proof-nix-closure.txt
                  test -s ${octetStandardsNixClosureManifest}/octet-standards-closure.graph
                  test -s ${octetStandardsNixClosureManifest}/octet-standards-closure.graph.canonical
                  test -s ${octetStandardsNixClosureManifest}/octet-standards-closure.graph.blake3
                  grep -F "proof-verifier-nix-derivation:" ${octetStandardsNixClosureManifest}/proof-nix-closure.txt >/dev/null
                  grep -F "proof-verifier-nix-output: ${octetStandards}" ${octetStandardsNixClosureManifest}/proof-nix-closure.txt >/dev/null
                  grep -F "proof-verifier-nix-closure: exportReferencesGraph:octet-standards-closure" ${octetStandardsNixClosureManifest}/proof-nix-closure.txt >/dev/null
                  graph_digest=$(cat ${octetStandardsNixClosureManifest}/octet-standards-closure.graph.blake3)
                  grep -F "proof-verifier-nix-closure-graph-blake3: $graph_digest" ${octetStandardsNixClosureManifest}/proof-nix-closure.txt >/dev/null
                  mkdir -p "$out"
                '';
            octet-production-verus-nix-closure-manifest =
              pkgs.runCommand "octet-production-verus-nix-closure-manifest-check" { }
                ''
                  test -s ${octetProductionVerusNixClosureManifest}/proof-nix-closure.txt
                  test -s ${octetProductionVerusNixClosureManifest}/octet-production-verus-closure.graph
                  test -s ${octetProductionVerusNixClosureManifest}/octet-production-verus-closure.graph.canonical
                  test -s ${octetProductionVerusNixClosureManifest}/octet-production-verus-closure.graph.blake3
                  grep -F "proof-verifier-nix-output: ${octetProductionVerus}" ${octetProductionVerusNixClosureManifest}/proof-nix-closure.txt >/dev/null
                  grep -F "proof-verifier-nix-closure: exportReferencesGraph:octet-production-verus-closure" ${octetProductionVerusNixClosureManifest}/proof-nix-closure.txt >/dev/null
                  graph_digest=$(cat ${octetProductionVerusNixClosureManifest}/octet-production-verus-closure.graph.blake3)
                  grep -F "proof-verifier-nix-closure-graph-blake3: $graph_digest" ${octetProductionVerusNixClosureManifest}/proof-nix-closure.txt >/dev/null
                  mkdir -p "$out"
                '';
            slotcar-standards-nix-closure-manifest = self.checks.${system}.octet-standards-nix-closure-manifest;
            policy-artifact-consumer-fixture = policyArtifactConsumerFixtureCheck;
          };

        devShells.default = craneLib.devShell {
          inherit buildInputs;

          packages = [
            cargo-shim
            rustup-shim
            pkgs.cargo-nextest
            pkgs.cargo-watch
            pkgs.cargo-llvm-cov
            pkgs.cargo-mutants
            pkgs.ast-grep
            cairn.packages.${system}.cairn
            pkgs.jq
            nickel
            pkgs.rust-analyzer
            octetStandards
            octetVerus
            octetProductionVerus
          ]
          ++ pkgs.lib.optionals (verusfmtAsset != null) [ octetVerusfmt ];

          inputsFrom = [ octet ];

          LD_LIBRARY_PATH = rustToolchainLibPath;
          RUSTUP_TOOLCHAIN = rustupToolchain;
          OCTET_COVERAGE_CARGO = "${cargo-shim}/bin/cargo";
          OCTET_COVERAGE_PATH = "${cargoOctetRuntimePath}/bin";
          OCTET_COVERAGE_CARGO_CONFIG = "${cargoVendorDir}/config.toml";
          OCTET_COVERAGE_LD_LIBRARY_PATH = rustToolchainLibPath;
          OCTET_MUTATION_CARGO = "${cargo-shim}/bin/cargo";
          OCTET_MUTATION_PATH = "${cargoOctetRuntimePath}/bin";
          OCTET_MUTATION_CARGO_CONFIG = "${cargoVendorDir}/config.toml";
          OCTET_MUTATION_LD_LIBRARY_PATH = rustToolchainLibPath;
          OCTET_HEGEL_CARGO = "${cargo-shim}/bin/cargo";
          OCTET_HEGEL_PATH = "${cargoOctetRuntimePath}/bin";
          OCTET_HEGEL_CARGO_CONFIG = "${cargoVendorDir}/config.toml";
          OCTET_HEGEL_LD_LIBRARY_PATH = rustToolchainLibPath;
          shellHook = ''
            export PATH="${dylintShimPath}:$PATH"
          '';
        };
      }
    );
}
