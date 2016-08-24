//! # imag
//!
//! This is the _developer_ documentation for the imag personal information management suite for the
//! commandline.
//!
//! For the user documentation, have a look
//! [at the 'doc' subtree in the repository](http://git.imag-pim.org/imag/tree/doc)
//! which can be compiled to HTML or PDF using [pandoc](pandoc.org) (and might be a bit outdated as
//! imag is not yet released for use).
//!
//! ## General
//!
//! _Some_ things from the user documentation might be helpful for developers as well, so make sure
//! to at least skim over it if you want to contribute to the imag source.
//!
//! Also make sure you had a look at
//! [the CONTRIBUTING](http://git.imag-pim.org/imag/tree/CONTRIBUTING.md)
//! file and [the developers certificate of origin](http://developercertificate.org/), which we also
//! have in the `CONTRIBUTING` file, by the way.
//!
//! ## Contributing
//!
//! All things relevant for contributing are descripbed in
//! [the CONTRIBUTING file](http://git.imag-pim.org/imag/tree/CONTRIBUTING.md),
//! but here are some additional notes:
//!
//! * We have a `editorconfig` file in the repository. Would be nice if you'd
//!   [use it](http://editorconfig.org/)
//! * We have [default.nix](http://git.imag-pim.org/imag/tree/default.nix) file, which can be used
//!   to install dependencies in `nix-shell` environments. If you have a installation of the nix
//!   package manager, feel free to use this opportunity to be _pure_.
//! * If you want to play around with imag, use the
//!   [imagrc.toml](http://git.imag-pim.org/imag/tree/imagrc.toml)
//!   file from the repository, we try to keep it up to date.
//! * You can use [the Makefile](http://git.imag-pim.org/imag/tree/Makefile) to build things (if you
//!   have all dependencies and cargo/rustc installed, of course).
//! * It is a real advantage to use `cargo-check` when developing things - it speeds you up, beleive
//!   me!
//!

