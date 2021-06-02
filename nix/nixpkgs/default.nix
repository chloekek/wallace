let
    pinned   = fromTOML (builtins.readFile ./pinned.toml);
    tarball  = fetchTarball pinned;
    config   = { };
    overlays = map import [
        ../wallace
    ];
in
    import tarball { inherit config overlays; }
