self: super:

{
    wallace = {

        # Pinned versions of third-party dependencies.
        pinned = {
            inherit (self.rust_1_49.packages.stable)
                cargo;
        };

    };
}
